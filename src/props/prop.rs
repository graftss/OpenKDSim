use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::{Rc, Weak},
};

use gl_matrix::{
    common::{Mat4, Vec3},
    mat4, vec3,
};

use crate::{
    collision::{mesh::Mesh, util::max_transformed_y},
    constants::{FRAC_1_3, FRAC_PI_750, UNITY_TO_SIM_SCALE, _4PI},
    macros::{max_to_none, new_mat4_copy, set_translation, temp_debug_log, vec3_from},
    mono_data::{PropAabbs, PropMonoData},
    player::Player,
    props::config::NamePropConfig,
    util::scale_sim_transform,
};

const FLAG_HAS_PARENT: u8 = 0x2;
const FLAG_INTANGIBLE_CHILD: u8 = 0x4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropGlobalState {
    /// Normal unattached state.
    Unattached = 1,

    /// Normal attached state.
    Attached = 2,

    /// Briefly, just after being knocked airborne, while intangible to the katamari.
    AirborneIntangible = 4,
}

impl Default for PropGlobalState {
    fn default() -> Self {
        Self::Unattached
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropLinkAction {
    /// No special link action.
    None,

    /// (???)
    MaybeFollowParent,

    /// A child prop with this `LinkAction` destroys itself when its
    /// parent prop is attached.
    DestroyWhenParentAttached,

    /// A child prop with this `LinkAction` is intangible.
    IntangibleChild,

    /// (??)
    MaybeIgnoreParent,

    /// (???)
    MaybeUseParent,

    /// A child prop with this `LinkAction` switches to its alt motion action
    /// after its parent is attached.
    ReactWhenParentAttached,

    Unknown(u16),
}

impl From<u16> for PropLinkAction {
    fn from(value: u16) -> Self {
        match value {
            0 => PropLinkAction::None,
            1 => PropLinkAction::MaybeFollowParent,
            2 => PropLinkAction::DestroyWhenParentAttached,
            3 => PropLinkAction::IntangibleChild,
            4 => PropLinkAction::MaybeIgnoreParent,
            5 => PropLinkAction::MaybeUseParent,
            6 => PropLinkAction::ReactWhenParentAttached,
            _ => PropLinkAction::Unknown(value),
        }
    }
}

#[derive(Debug, Default)]
struct PropSubobject {
    /// The next subobject in the linked list, if one exists.
    /// offset: 0x10
    pub next: Option<Box<PropSubobject>>,

    /// (??) The position of this subobject (presumably relative to the prop).
    /// offset: 0x18
    pub pos: Vec3,

    /// (??) The Euler agnles of this subobject
    /// offset: 0x28
    pub rot_vec: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum PropUnattachedState {
    /// The default state that most props are in most of the time.
    Normal = 0,

    State1 = 1,
    State2 = 2,
    State3 = 3,
    State4 = 4,

    /// Active after an airborne prop bounces off a floor once.
    AirborneBounced = 5,
}

impl Default for PropUnattachedState {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum PropAnimationType {
    Waiting = 0,
    MovingForward = 1,
    Animation2 = 2,
    PathFleeing = 3,
    Animation4 = 4,
    PutterSwing = 5,
}

impl Default for PropAnimationType {
    fn default() -> Self {
        Self::Waiting
    }
}

impl Into<u8> for PropAnimationType {
    fn into(self) -> u8 {
        match self {
            Self::Waiting => 0,
            Self::MovingForward => 1,
            Self::Animation2 => 2,
            Self::PathFleeing => 3,
            Self::Animation4 => 4,
            Self::PutterSwing => 5,
        }
    }
}

// holy cannoli what were they thinking
#[derive(Debug)]
pub struct AddPropArgs {
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub rot_w: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
    pub name_idx: u16,
    pub loc_pos_type: u16,
    pub random_group_id: u16,
    pub mono_move_type: u16,
    pub mono_hit_on_area: u16,
    pub link_action: u16,
    pub extra_action_type: u16,
    pub unique_name_id: u16,
    pub disp_off_area_no: u16,
    pub vs_drop_flag: u16,
    pub comment_id: u16,
    pub comment_group_id: u16,
    pub twin_id: u16,
    pub shake_off_flag: u16,
}

impl AddPropArgs {
    pub fn transform_coords_to_sim(&mut self) {
        self.pos_x *= UNITY_TO_SIM_SCALE;
        self.pos_y *= UNITY_TO_SIM_SCALE;
        self.pos_z *= UNITY_TO_SIM_SCALE;
    }
}

pub type PropScript = fn(prop: PropRef) -> ();

/// The six different ways in which a prop's transform can be computed.
/// Each state corresponds to one the six callbacks starting at offset 0x69e48,
/// in the order defined in this enum.
enum UnattachedTransformState {
    Normal,
    Stalled,
    StationaryChild,
    StationaryChildStalled,
    MovingChild,
    MovingChildStalled,
}

#[derive(Debug, Default)]
pub struct Prop {
    /// The unique id of this prop.
    /// offset: 0x0
    ctrl_idx: u16,

    /// The object type of this prop.
    /// offset: 0x2
    name_idx: u16,

    /// (??) some flags
    /// offset: 0x6
    flags: u8,

    /// offset: 0x7
    global_state: PropGlobalState,

    /// (??) some more flags
    /// offset: 0x8
    flags2: u8,

    /// If true, the prop won't update.
    /// offset: 0x9
    force_disabled: bool,

    /// If false, the prop won't be displayed, but it will still be tangible.
    /// offset: 0xa
    display_on: bool,

    /// The alpha level of the prop (fades out as it gets further from the player camera).
    /// offset: 0x14
    alpha: f32,

    /// (??) The type of movement that this prop performs.
    /// offset: 0x16
    move_type: Option<u16>,

    /// The prop is intangible until the player loads this area.
    /// offset: 0x1a
    hit_on_area: Option<u16>,

    /// (??) encodes the behavior of this prop relative to its child/parent
    /// offset: 0x1b
    link_action: Option<PropLinkAction>,

    /// Encodes motion behavior innate to this prop's name index.
    /// offset: 0x1c
    innate_motion_type: Option<u8>,

    /// The state index of the innate motion action.
    /// offset: 0x1d
    innate_motion_state: u8,

    /// True if the prop's motion action follows a path.
    /// offset: 0x1f
    is_following_path: bool,

    /// If true, the prop cannot wobble.
    /// offset: 0x20
    disable_wobble: bool,

    /// True if the prop's motion action applies any translation.
    /// offset: 0x21
    has_motion: bool,

    /// The next sibling of this prop in its family tree.
    /// offset: 0x28
    pub next_sibling: Option<PropRef>,

    /// The first child of this prop in its family tree.
    /// offset: 0x30
    pub first_child: Option<PropRef>,

    /// The area in which this prop loaded.
    /// offset: 0x38
    init_area: u8,

    /// The position at which this prop loaded.
    /// offset: 0x40
    init_pos: Vec3,

    /// The prop's rotation as a matrix.
    /// offset: 0x50
    rotation_mat: Mat4,

    /// The prop's position.
    /// offset: 0x90
    pos: Vec3,

    /// The prop's rotation as Euler angles.
    /// offset: 0xa0
    rotation_vec: Vec3,

    /// The prop's scale. (unused in simulation)
    /// offset: 0xb0
    scale: Vec3,

    /// The prop's position on the previous tick.
    /// offset: 0xc0
    last_pos: Vec3,

    /// The prop's rotation as Euler angles on the previous tick.
    /// offset: 0xd0
    last_rotation_vec: Vec3,

    /// The prop's transform matrix while unattached from the katamari.
    /// offset: 0x110
    unattached_transform: Mat4,

    /// The prop's initial rotation matrix when it loaded.
    /// offset: 0x150
    init_rotation_mat: Mat4,

    /// (??) the transform when moving?
    /// offset: 0x190
    motion_transform: Mat4,

    /// A pointer to the prop's first subobject.
    /// offset: 0x558
    first_subobject: Option<Box<PropSubobject>>,

    /// (??) a script that seems to only be used for policeman gunshots in reroll
    /// offset: 0x560
    script_0x560: Option<Box<PropScript>>,

    /// The prop's motion script, if it has one.
    /// offset: 0x568
    motion_script: Option<Box<PropScript>>,

    /// The prop's innate script, if it has one.
    /// offset: 0x570
    innate_script: Option<Box<PropScript>>,

    /// The prop's parent, if it has one.
    /// offset: 0x578
    pub parent: Option<WeakPropRef>,

    /// (??) name taken from unity code
    /// offset: 0x580
    extra_action_type: Option<u16>,

    /// The unique name id of this prop, if it has one.
    /// This can either be from being a "named" object, or from being a twinned object in Gemini.
    /// offset: 0x581
    unique_name_id: Option<u16>,

    /// The area at which this prop is destroyed.
    /// offset: 0x582
    display_off_area: Option<u8>,

    /// (??)
    /// offset: 0x583
    vs_drop_flag: bool,

    /// The prop's concrete motion action.
    /// offset: 0x584
    motion_action_type: Option<u16>,

    /// (??) The prop's behavior type, which encodes the primary motion action, the alternate motion
    /// action, and the (???)
    /// offset: 0x585
    behavior_type: Option<u16>,

    /// The prop's alternate concrete motion action, which can be triggered by various events
    /// (e.g. katamari gets close, or katamari collects this prop's parent, etc.)
    /// offset: 0x586
    alt_motion_action_type: Option<u16>,

    /// (??) The prop's state while it's unattached.
    /// offset: 0x588
    unattached_state: PropUnattachedState,

    /// The prop's animation type.
    /// offset: 0x589
    animation_type: PropAnimationType,

    /// The ID of the king message that's played when this prop is collected, if any.
    /// offset: 0x58a
    comment_id: Option<u16>,

    /// The ID of the king message that's played when all props sharing this group ID are
    /// collected, if any.
    /// offset: 0x58b
    comment_group_id: Option<u16>,

    /// The unique ID of this prop's tree. All props in the same tree have the same ID.
    /// offset: 0x58c
    tree_id: Option<u16>,

    /// True if the prop isn't moving.
    /// offset: 0x58e
    stationary: bool,

    /// If true, the prop can't wobble when the katamari collides with it.
    /// Temporarily set by motion actions such as hops.
    /// offset: 0x58f
    force_no_wobble: bool,

    /// The AABB of this prop encoded as a collision mesh.
    /// offset: 0x5e0
    aabb_mesh: Mesh,

    /// The 8 corner points of the prop's AABB.
    /// offset: 0x870
    aabb_vertices: Vec<Vec3>,

    /// The radius of the prop's bounding sphere.
    /// offset: 0x910
    radius: f32,

    /// The volume of the prop's AABB (in m^3).
    /// offset: 0x918
    aabb_vol_m3: f32,

    /// The volume of the prop used when comparing to the katamari's volume for the purposes
    /// of attaching the prop (in m^3).
    /// offset: 0x91c
    compare_vol_m3: f32,

    /// The *base* volume added to the katamari when this prop is attached (in m^3).
    /// This value will still be scaled by the mission's penalty.
    /// offset: 0x920
    attach_vol_m3: f32,

    /// Half the maximum AABB side length; used to quickly decide if the katamari
    /// is close enough to bother doing a full collision test.
    /// offset: 0x928
    aabb_radius: f32,

    /// The exact katamari diameter needed to collect this object (in cm), obtained by
    /// comparing the prop's AABB volume to the katamari's volume.
    /// The prop's true collection diameter is obtained by truncating this value to an integral value in mm.
    /// offset: 0x930
    exact_attach_diam_cm: f32,

    /// The minimum katamari diameter needed to collect this object (in mm).
    /// offset: 0x934
    attach_diam_mm: i32,

    /// The sizes of the prop's AABB.
    /// offset: 0x938
    aabb_size: Vec3,

    /// The true volume added to the katamari, computed *at the time the prop is attached*.
    /// This value is saved so that if the prop is lost, the volume can be recovered and subtracted
    /// from the katamari's volume.
    /// offset: 0x944
    onattach_added_vol: f32,

    /// The number of ticks remaining on the mission timer when the prop was attached.
    /// offset: 0x950
    onattach_remain_ticks: i32,

    /// The real-time game time when the prop was attached (in ms).
    /// offset: 0x954
    onattach_game_time_ms: i32,

    /// Information about this type of prop from its `PropMonoData`.
    /// Namely, its AABB, collision mesh, and vault points.
    mono_data: Option<Rc<PropMonoData>>,

    /// (??) The additional transform applied to the prop while it is attached to the katamari.
    /// offset: 0x968
    attached_transform: Mat4,

    /// While attached, the offset from the katamari center to the prop's center.
    /// offset: 0x9e8
    kat_center_offset: Vec3,

    /// (??) The velocity of the katamari when it collided with the prop.
    /// offset: 0x9f8
    kat_collision_vel: Vec3,

    /// The prop's distance to player 0 on the previous frame.
    /// offset: 0xa08
    last_dist_to_p0: f32,

    /// The prop's distance to player 1 on the previous frame.
    /// offset: 0xa0c
    last_dist_to_p1: f32,

    /// The prop's distance to player 0.
    /// offset: 0xa10
    dist_to_p0: f32,

    /// The prop's distance to player 1.
    /// offset: 0xa14
    dist_to_p1: f32,

    /// The remaining "bonk force" needed to detach this prop from the katamari.
    /// Initialized to the prop's unscaled volume when the prop is attached.
    /// offset: 0xa18
    remain_knockoff_volume: f32,

    /// A multiple of the prop's volume that seems to be used somewhere. Hell if I know
    /// offset: 0xa24
    weird_vol_multiple: f32,

    /// If this prop is attached, points to the prop that was attached before this (if one exists).
    /// offset: 0xa28
    collected_before: Option<WeakPropRef>,

    /// If this prop is attached, points to the prop that was attached after this (if one exists).
    /// offset: 0xa30
    collected_next: Option<WeakPropRef>,

    /// When attached, the index of the katamari collision ray nearest to this prop.
    /// offset: 0xa88
    nearest_kat_ray: Option<u16>,

    /// If >0, this prop is intangible. Decrements by 1 each tick.
    /// offset: 0xa8a
    intangible_ticks: u16,

    /// The cooldown on when this prop can scream, when >0. If 0, the prop is ready to scream again
    /// upon collision with the katamari.
    /// offset: 0xa8f
    scream_cooldown_ticks: u8,

    /// True if this prop has a twin prop on the Gemini mission.
    /// offset: 0xb10
    has_twin: bool,

    /// The twin ID of this prop, if any. Both twins should have the same unique ID.
    twin_id: Option<u16>,

    /// If this prop has a Gemini twin, points to the twin prop.
    /// offset: 0xb18
    twin_prop: Option<WeakPropRef>,

    /// The transform matrix of this prop when it was loaded.
    /// offset: 0xb24
    init_transform: Mat4,

    /// The vector that the prop moved between the previous frame and this frame.
    /// offset: 0xb64
    delta_pos_unit: Vec3,

    /// The Euler angles of this prop when it was loaded.
    /// offset: 0xbb8
    init_rotation_vec: Vec3,
}

pub type PropRef = Rc<RefCell<Prop>>;
pub type WeakPropRef = Weak<RefCell<Prop>>;

impl Display for Prop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Prop(ctrl={})", self.ctrl_idx)
    }
}

// impl Debug for Prop {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Prop")
//             .field("ctrl_idx", &self.ctrl_idx)
//             .finish()
//     }
// }

impl Prop {
    pub fn print_links(&self, label: &str) {
        println!(
            "{} (tree={:?}):\n  child: {:?}\n  next_sibling: {:?}\n  parent: {:?}",
            label,
            self.tree_id,
            self.first_child,
            self.next_sibling,
            self.parent.as_ref().map(|p| p.upgrade().unwrap())
        );
    }
}

impl Prop {
    pub fn new(
        ctrl_idx: u16,
        args: &AddPropArgs,
        area: u8,
        mono_data: &Rc<PropMonoData>,
    ) -> PropRef {
        Rc::new(RefCell::new(Prop::new_node(
            ctrl_idx, args, area, mono_data,
        )))
    }

    /// Create a new `Prop` object.
    /// Mostly follows the function `prop_init`.
    /// offset: 0x4e950
    pub fn new_node(
        ctrl_idx: u16,
        args: &AddPropArgs,
        area: u8,
        mono_data: &Rc<PropMonoData>,
    ) -> Self {
        let name_idx = args.name_idx;
        let config = NamePropConfig::get(name_idx.into());
        let mono_data = mono_data.clone();

        // initialize rotation matrix
        let id = mat4::create();
        let mut rotation_mat = mat4::create();
        let rot_axis = [args.rot_x, args.rot_y, args.rot_z];
        let rot_angle = args.rot_w;
        mat4::rotate(&mut rotation_mat, &id, -rot_angle, &rot_axis);

        // save the initial rotation
        new_mat4_copy!(init_rotation_mat, rotation_mat);

        // initialize unattached transform to the initial rotation mat
        new_mat4_copy!(unattached_transform, rotation_mat);
        new_mat4_copy!(init_transform, rotation_mat);

        // TODO
        // lines 104-107 of `prop_init` (init comment)
        // lines 108-149 of `prop_init` (init motion)
        // lines 150-162 of `prop_init` (init random group)
        // lines 163-190 of `prop_init` (init twin)
        // lines 348-349 (find first subobject)
        // lines 350-357 (init motion scripts)
        // lines 368-371, 392-401 (init wobble state)
        // lines 373-384 (init generated prop??)
        // line 385
        // lines 386-391 (init fish)

        let mut result = Prop {
            ctrl_idx,
            name_idx,
            flags: 0,
            global_state: PropGlobalState::Unattached,
            flags2: 0,
            force_disabled: false,
            display_on: false,
            alpha: 1.0,
            move_type: max_to_none!(u16, args.mono_move_type),
            hit_on_area: max_to_none!(u16, args.mono_hit_on_area),
            link_action: max_to_none!(u16, args.link_action).map(|v| v.into()),
            innate_motion_type: max_to_none!(u8, config.innate_motion_type),
            innate_motion_state: 0,
            is_following_path: false,
            disable_wobble: args.shake_off_flag != 0,
            has_motion: false,
            next_sibling: None,
            first_child: None,
            init_area: area,
            init_pos: [args.pos_x, args.pos_y, args.pos_z],
            rotation_mat: rotation_mat,
            pos: [args.pos_x, args.pos_y, args.pos_z],
            rotation_vec: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            last_pos: [args.pos_x, args.pos_y, args.pos_z],
            last_rotation_vec: [args.rot_x, args.rot_y, args.rot_z],
            init_rotation_vec: [args.rot_x, args.rot_y, args.rot_z],
            unattached_transform: unattached_transform,
            init_rotation_mat: init_rotation_mat,
            init_transform: init_transform,
            motion_transform: [0.0; 16],
            extra_action_type: max_to_none!(u16, args.extra_action_type),
            unique_name_id: max_to_none!(u16, args.unique_name_id),
            display_off_area: max_to_none!(u8, args.disp_off_area_no as u8),
            vs_drop_flag: args.vs_drop_flag != 0,
            unattached_state: PropUnattachedState::Normal,
            animation_type: PropAnimationType::Waiting,
            comment_id: max_to_none!(u16, args.comment_id),
            comment_group_id: max_to_none!(u16, args.comment_group_id),
            stationary: true,
            force_no_wobble: false,
            onattach_added_vol: 0.0,
            onattach_remain_ticks: 0,
            onattach_game_time_ms: 0,
            attached_transform: [0.0; 16],
            kat_center_offset: [0.0; 3],
            kat_collision_vel: [0.0; 3],
            last_dist_to_p0: 0.0,
            last_dist_to_p1: 0.0,
            dist_to_p0: 0.0,
            dist_to_p1: 0.0,
            remain_knockoff_volume: 0.0,
            collected_before: None,
            collected_next: None,
            has_twin: args.twin_id != u16::MAX,
            twin_id: max_to_none!(u16, args.twin_id),
            nearest_kat_ray: None,
            intangible_ticks: 0,
            scream_cooldown_ticks: 0,
            parent: None,

            // initialized in `self.init_aabb_and_volume()`
            aabb_mesh: Mesh::default(),
            aabb_vertices: vec![],
            aabb_size: [0.0; 3],
            aabb_vol_m3: 0.0,
            compare_vol_m3: 0.0,
            attach_vol_m3: 0.0,
            exact_attach_diam_cm: 0.0,
            attach_diam_mm: 0,
            radius: 0.0,
            aabb_radius: 0.0,
            weird_vol_multiple: 0.0,

            first_subobject: None, // TODO
            script_0x560: None,    // TODO
            motion_script: None,   // TODO
            innate_script: None,
            tree_id: None,                // TODO
            motion_action_type: None,     // TODO
            behavior_type: None,          // TODO
            alt_motion_action_type: None, // TODO
            twin_prop: None,
            mono_data: None,

            delta_pos_unit: [0.0; 3],
        };

        if let Some(aabbs) = &mono_data.aabbs {
            result.init_aabb_and_volume(aabbs, config);
        }

        // move random-spawn props vertically so that they're resting on the ground
        if args.loc_pos_type != 0 {
            let y_offset = result.max_aabb_y();
            result.pos[1] += y_offset;
            result.init_pos[1] += y_offset;
        }

        // note the conditional call to `prop_init_tree_links` here in the original sim,
        // but the condition to call it appears to never be true in reroll.

        result.mono_data = Some(mono_data);

        result
    }

    /// Initialize the prop's AABB and volume
    fn init_aabb_and_volume(&mut self, aabbs: &PropAabbs, config: &NamePropConfig) {
        // TODO: refactor this as a simulation param
        let VOL_RATIO_FOR_PICKUP = 0.1;

        if config.is_dummy_hit {
            // TODO: `prop_init_aabb_and_volume:222-250`
            return;
        }

        // TODO: `prop_init_aabb_and_volume:90-115` (compute an AABB that includes the AABB's of all subobjects,
        //       rather than just the prop's own AABB)
        // (but this behavior is in `aabbs.get_root_aabb` in this implementation)
        let root_aabb = aabbs.get_root_aabb();
        self.aabb_vertices = root_aabb.compute_vertices();
        self.aabb_mesh = root_aabb.compute_mesh(&self.aabb_vertices);
        self.aabb_size = root_aabb.size();
        self.radius = root_aabb.compute_radius();

        // compute various volumes
        // for the purposes of finding a prop's volume, each side length of
        // its AABB is treated as being at least 0.1.
        let mut vol_sizes_m = vec3::create();
        let mut min_vol_size_m = f32::INFINITY;
        for i in 0..3 {
            let real_size = self.aabb_size[i];
            vol_sizes_m[i] = if real_size < 0.1 { 0.1 } else { real_size };

            vol_sizes_m[i] *= 0.01;

            // maintain the maximum volume-box side length to be used for the katamari
            // collision radius.
            if vol_sizes_m[i] < min_vol_size_m {
                min_vol_size_m = vol_sizes_m[i];
                println!("mvsm={}", min_vol_size_m);
            }
        }

        self.aabb_vol_m3 = vol_sizes_m[0] * vol_sizes_m[1] * vol_sizes_m[2];
        self.compare_vol_m3 = self.aabb_vol_m3 * config.compare_vol_mult;
        self.attach_vol_m3 = self.compare_vol_m3 * config.attach_vol_mult;
        self.aabb_radius = min_vol_size_m * 100.0 * 0.5;
        self.weird_vol_multiple = self.compare_vol_m3 / FRAC_PI_750;

        // compute katamari diameter needed to attach this prop
        let attach_rad_m = (self.compare_vol_m3 / VOL_RATIO_FOR_PICKUP * 3.0 / _4PI).powf(FRAC_1_3);
        self.exact_attach_diam_cm = attach_rad_m * 100.0 + attach_rad_m * 100.0;
        self.attach_diam_mm = (self.exact_attach_diam_cm * 10.0) as i32;
    }

    pub fn get_name_idx(&self) -> u16 {
        self.name_idx
    }

    pub fn get_ctrl_idx(&self) -> u16 {
        self.ctrl_idx
    }

    pub fn is_initialized(&self) -> bool {
        self.name_idx != u16::MAX
    }

    /// Traverses the prop's linked list of subobjects to compute the list's length.
    pub fn count_subobjects(&self) -> i32 {
        let mut result = 0;
        let mut try_next_subobj = &self.first_subobject;

        loop {
            if let Some(next_subobj) = try_next_subobj {
                result += 1;
                try_next_subobj = &next_subobj.next;
            } else {
                break;
            }
        }

        return result;
    }

    /// Traverse the prop's linked list of subobject to find the subobject at index `subobj_idx`.
    fn get_subobject(&self, subobj_idx: i32) -> &Option<Box<PropSubobject>> {
        let mut next_idx = 0;
        let mut try_next_subobj = &self.first_subobject;

        loop {
            if let Some(next_subobj) = try_next_subobj {
                if next_idx == subobj_idx {
                    return try_next_subobj;
                } else {
                    next_idx += 1;
                    try_next_subobj = &next_subobj.next;
                }
            } else {
                return &None;
            }
        }
    }

    /// Mimicks the `GetSubobjectPosition` API function.
    pub fn get_subobject_position(
        &self,
        subobj_idx: i32,
        pos_x: &mut f32,
        pos_y: &mut f32,
        pos_z: &mut f32,
        rot_x: &mut f32,
        rot_y: &mut f32,
        rot_z: &mut f32,
    ) {
        if let Some(subobj) = self.get_subobject(subobj_idx) {
            *pos_x = subobj.pos[0];
            *pos_y = subobj.pos[1];
            *pos_z = subobj.pos[2];
            *rot_x = subobj.rot_vec[0];
            *rot_y = subobj.rot_vec[1];
            *rot_z = subobj.rot_vec[2];
        }
    }

    pub fn is_attached(&self) -> bool {
        self.global_state == PropGlobalState::Attached
    }

    pub fn get_pos(&self, out: &mut Vec3) {
        vec3::copy(out, &self.pos);
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    /// Writes the active transform to `out`.
    /// This can either be the unattached transform or the attached transform.
    pub unsafe fn unsafe_copy_transform(&self, out: *mut Mat4) {
        let mut transform = if self.is_attached() {
            self.attached_transform.clone()
        } else {
            self.unattached_transform.clone()
        };

        scale_sim_transform(&mut transform);

        mat4::copy(&mut *out, &transform);
    }

    /// Mimicks the `MonoGetVolume` API function.
    pub fn get_volume(&self, volume: &mut f32, collect_diam: &mut i32) {
        *volume = self.compare_vol_m3;
        *collect_diam = self.attach_diam_mm;
    }

    pub fn set_disabled(&mut self, force_disabled: i32) {
        self.force_disabled = force_disabled != 0;
    }

    pub fn is_disabled(&self) -> bool {
        self.force_disabled
    }

    pub fn set_no_parent(&mut self) {
        self.flags &= 0xfd;
        self.parent = None;
        self.tree_id = None;
    }

    pub fn set_parent(&mut self, parent: WeakPropRef, tree_group_id: u16) {
        self.flags |= FLAG_HAS_PARENT;
        self.parent = Some(parent.clone());

        if self.link_action == Some(PropLinkAction::IntangibleChild) {
            self.flags |= FLAG_INTANGIBLE_CHILD;
        }

        self.tree_id = Some(tree_group_id);

        // TODO: line 53: `prop_update_rotation_from_parent(self)`

        mat4::identity(&mut self.motion_transform);
    }

    /// Add `child` as a child of this prop by add it to the end
    /// of the sibling list.
    pub fn add_child(&mut self, child: PropRef) {
        if let Some(first_child) = &self.first_child {
            first_child.clone().borrow_mut().add_sibling(child);
        } else {
            self.first_child = Some(child);
        }
    }

    /// Traverse this prop's sibling list, adding `sibling` to the end.
    pub fn add_sibling(&mut self, sibling: PropRef) {
        if let Some(next_sibling) = &self.next_sibling {
            next_sibling.clone().borrow_mut().add_sibling(sibling);
        } else {
            self.next_sibling = Some(sibling);
        }
    }

    /// Used by the `GetPropAttached` API function.
    /// Writes 3 bytes to `out`.
    pub unsafe fn get_attach_status(&self, out: *mut u8, kat_diam_int: i32) {
        let mut status: u8 = 0;

        // status & 1: prop is attached
        if self.is_attached().into() {
            status |= 0x1;
        }

        // status & 2: prop is collectible
        if self.attach_diam_mm <= kat_diam_int {
            status |= 0x2;
        }

        // status & 4: prop has a subobject
        if self.first_subobject.is_some() {
            status |= 0x4;
        }

        // status & 8: prop is disabled
        if self.force_disabled {
            status |= 0x8;
        }

        // status & 0x10: prop is hidden
        if !self.display_on {
            status |= 0x10;
        }

        // TODO
        // status & 0x20: prop changes name when attached
        if false {
            status |= 0x20;
        }

        // `out[0]` is the `status` flags.
        *out.offset(0) = status;

        // `out[1]` is the animation type
        *out.offset(1) = self.animation_type.into();

        // `out[2]` if the prop alpha (quantized to a `u8` value)
        *out.offset(2) = (self.alpha * 255.0) as u8;
    }

    /// Returns `true` if this prop should be destroyed when the area `area` loads.
    pub fn check_destroy_on_area_load(&self, area: u8) -> bool {
        let should_destroy = self
            .display_off_area
            .map_or(false, |destroy_area| destroy_area == area)
            && self.global_state != PropGlobalState::Attached;

        // TODO: if `should_destroy` is true, call `prop_destroy`

        should_destroy
    }
}

impl Prop {
    /// Computes the highest point (in local space) on this prop's AABB
    /// after transforming the AABB with its current rotation matrix.
    pub fn max_aabb_y(&self) -> f32 {
        max_transformed_y(&self.aabb_vertices, &self.rotation_mat)
    }
}

impl Prop {
    /// Update a prop when not in the ending mission, i.e. the "normal" prop update logic.
    /// offset: 0x50050 (note: that offset's function loops over all props, and this function is
    ///                  one iteration of that loop.)
    pub fn update_nonending(&mut self, player: &Player) {
        if self.force_disabled {
            return;
        }

        vec3::copy(&mut self.last_pos, &self.pos);
        vec3::copy(&mut self.last_rotation_vec, &self.rotation_vec);

        match self.global_state {
            PropGlobalState::Unattached => self.update_unattached(),
            PropGlobalState::Attached => self.update_attached(),
            PropGlobalState::AirborneIntangible => self.update_airborne_intangible(),
        }

        self.update_child_link();

        if let Some(_script) = self.motion_script.as_ref() {
            // TODO_PROP_MOTION: `props_update_nonending:55-68`
        }

        if let Some(_script) = self.innate_script.as_ref() {
            // TODO_PROP_MOTION: call `innate_script`
        }

        self.cache_distance_to_players(player);

        let delta_pos = vec3_from!(-, self.pos, self.last_pos);
        vec3::normalize(&mut self.delta_pos_unit, &delta_pos);

        if self.global_state != PropGlobalState::Attached {
            // TODO_LINKS: `props_update_nonending:96-133` (different transform logic for linked props)
            // if (self.flags & 2) != 0 {
            //     // if prop is wobbling (??)
            // }
            // TODO_LINKS: this value should depend on the above code
            let transform_state = UnattachedTransformState::Normal;

            match transform_state {
                UnattachedTransformState::Normal => self.update_transform_normal(),
                UnattachedTransformState::Stalled => self.update_transform_stalled(),
                UnattachedTransformState::StationaryChild => {
                    self.update_transform_stationary_child()
                }
                UnattachedTransformState::StationaryChildStalled => {
                    self.update_transform_stationary_child_stalled()
                }
                UnattachedTransformState::MovingChild => self.update_transform_moving_child(),
                UnattachedTransformState::MovingChildStalled => {
                    self.update_transform_moving_child_stalled()
                }
            }
        }
    }

    // TODO_PROPS
    /// Update logic for a prop that's not attached to the katamari.
    /// offset: 0x50f10
    fn update_unattached(&mut self) {
        // match self.unattached_state {
        //     PropUnattachedState::Normal => todo!(),
        //     PropUnattachedState::State1 => todo!(),
        //     PropUnattachedState::State2 => todo!(),
        //     PropUnattachedState::State3 => todo!(),
        //     PropUnattachedState::State4 => todo!(),
        //     PropUnattachedState::AirborneBounced => todo!(),
        // }
    }

    /// Update logic for a prop that's attached to the katamari.
    /// offset: 0x50f30
    fn update_attached(&mut self) {
        self.flags &= 0xdf;
        self.flags2 &= 0xfe;
        self.move_type = None;
        self.has_motion = false;
        self.unattached_state = PropUnattachedState::Normal;
        self.animation_type = PropAnimationType::Animation2;
        self.is_following_path = false;
    }

    /// TODO_PROPS
    /// Update logic for a prop that's airborne.
    /// offset: 0x50eb0
    fn update_airborne_intangible(&mut self) {}

    /// Update logic for props which have a parent.
    /// offset: 0x2e030
    fn update_child_link(&mut self) {
        if self.parent.is_none() {
            self.flags &= 0xfd;
            return;
        }

        // TODO_PROPS: `prop_update_link:20-`
    }

    /// Compute the distance from this prop to other players and cache those
    /// distances on the prop for later use.
    /// offset: 0x50290
    fn cache_distance_to_players(&mut self, player: &Player) {
        self.last_dist_to_p0 = self.dist_to_p0;
        self.dist_to_p0 = vec3::distance(&self.pos, player.katamari.get_center());

        // TODO_VS: `prop_cache_distance_to_players:18+` (cache distance to other players)
    }
}

/// Subroutines to update a prop's unattached transform depending on its `PropTransformState` state.
impl Prop {
    /// Update a prop's transform when it is not a linked child and not stalled.
    /// offset: 0x51d90
    fn update_transform_normal(&mut self) {
        let mut temp1 = mat4::create();
        let mut temp2 = mat4::create();

        mat4::rotate_z(&mut temp2, &temp1, self.rotation_vec[2]);
        mat4::rotate_x(&mut temp1, &temp2, self.rotation_vec[0]);
        mat4::rotate_y(&mut temp2, &temp1, self.rotation_vec[1]);

        mat4::multiply(&mut self.unattached_transform, &self.rotation_mat, &temp2);

        set_translation!(self.unattached_transform, self.pos);
    }

    /// Update a prop's transform when it is not a linked child and stalled.
    /// offset: 0x51ac0
    fn update_transform_stalled(&mut self) {
        // TODO_STALLS
    }

    /// Update a prop's transform when it is a linked child, not moving, and not stalled.
    /// offset: 0x518a0
    fn update_transform_stationary_child(&mut self) {
        // TODO_LINKS
    }

    /// Update a prop's transform when it is a linked child, not moving, and stalled.
    /// offset: 0x51580
    fn update_transform_stationary_child_stalled(&mut self) {
        // TODO_LINKS
        // TODO_STALLS
    }

    /// Update a prop's transform when it is a linked child, moving, and not stalled.
    /// offset: 0x51f60
    fn update_transform_moving_child(&mut self) {
        // TODO_LINKS
    }

    /// Update a prop's transform when it is a linked child, moving, and stalled.
    /// offset: 0x52230
    fn update_transform_moving_child_stalled(&mut self) {
        // TODO_LINKS
        // TODO_STALLS
    }
}
