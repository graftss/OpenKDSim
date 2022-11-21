use std::{cell::RefCell, rc::Rc};

use gl_matrix::{
    common::{Mat4, Vec3, Vec4},
    mat4,
};

use crate::{
    gamestate::GameState,
    global::GlobalState,
    macros::{max_to_none, new_mat4_copy, new_mat4_id},
    mono_data::MonoDataPropPtrs,
    name_prop_config::NamePropConfig,
    util::scale_sim_transform,
};

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
enum PropLinkAction {}

#[derive(Debug, Default)]
struct PropSubobject {
    /// The next subobject in the linked list, if one exists.
    /// offset: 0x10
    pub next: Option<Box<PropSubobject>>,

    /// (??) The position of this subobject (presumably relative to the prop).
    /// offset: 0x18
    pub pos: Vec4,

    /// (??) The Euler agnles of this subobject
    /// offset: 0x28
    pub rot_vec: Vec4,
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

pub type PropScript = fn(prop: Prop) -> ();

#[derive(Debug, Default)]
pub struct PropNode {
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

    /// (??) If true, the prop won't be displayed.
    /// offset: 0xa
    display_off: bool,

    /// The alpha level of the prop (fades out as it gets further from the player camera).
    /// offset: 0x14
    alpha: f32,

    move_type: u16,

    /// (??) The prop is intangible until the player loads this area.
    /// offset: 0x1a
    hit_on_area: u8,

    /// (??) encodes the behavior of this prop relative to its child/parent
    /// offset: 0x1b
    link_action: Option<u16>,

    /// Encodes motion behavior innate to this prop's name index.
    /// offset: 0x1c
    innate_motion_type: u8,

    /// The state index of the innate motion action.
    /// offset: 0x1d
    innate_motion_state: u8,

    /// True if the prop's motion action follows a path.
    /// offset: 0x20
    has_path_motion_action: bool,

    /// If true, the prop cannot wobble.
    /// offset: 0x21
    disable_wobble: bool,

    /// True if the prop's motion action applies any translation.
    /// offset: 0x22
    has_moving_motion: bool,

    /// The next sibling of this prop in its family tree.
    /// offset: 0x28
    next_sibling: Option<Prop>,

    /// The first child of this prop in its family tree.
    /// offset: 0x30
    first_child: Option<Prop>,

    /// The area in which this prop loaded.
    /// offset: 0x38
    init_area: u8,

    /// The position at which this prop loaded.
    /// offset: 0x40
    init_pos: Vec4,

    /// The prop's rotation as a matrix.
    /// offset: 0x50
    rotation_mat: Mat4,

    /// The prop's position.
    /// offset: 0x90
    pos: Vec4,

    /// The prop's rotation as Euler angles.
    /// offset: 0xa0
    rotation_vec: Vec4,

    /// The prop's scale. (unused in simulation)
    /// offset: 0xb0
    scale: Vec4,

    /// The prop's position on the previous tick.
    /// offset: 0xc0
    last_pos: Vec4,

    /// The prop's rotation as Euler angles on the previous tick.
    /// offset: 0xd0
    last_rotation_vec: Vec4,

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
    parent_prop: Option<Prop>,

    /// (??) name taken from unity code
    /// offset: 0x580
    extra_action_type: Option<u16>,

    /// The unique name id of this prop, if it has one.
    /// This can either be from being a "named" object, or from being a twinned object in Gemini.
    /// offset: 0x581
    unique_name_id: Option<u16>,

    /// The area at which this prop is destroyed.
    /// offset: 0x582
    display_off_area: u16,

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
    tree_group_id: Option<u16>,

    /// True if the prop isn't moving.
    /// offset: 0x58e
    stationary: bool,

    /// If true, the prop can't wobble when the katamari collides with it.
    /// Temporarily set by motion actions such as hops.
    /// offset: 0x58f
    force_no_wobble: bool,

    /// The radius of the prop's bounding sphere.
    /// offset: 0x910
    radius: f32,

    /// The volume of the prop's AABB (in m^3).
    /// offset: 0x918
    aabb_vol_m3: f32,

    /// (??) The volume of the prop used when comparing to the katamari's volume (in m^3).
    /// offset: 0x91c
    compare_vol_m3: f32,

    /// The *base* volume added to the katamari when this prop is attached (in m^3).
    /// This value will still be scaled by the mission's penalty.
    /// offset: 0x920
    added_vol_m3: f32,

    /// The exact katamari diameter needed to collect this object (in cm), obtained by
    /// comparing the prop's AABB volume to the katamari's volume.
    /// The prop's true collection diameter is obtained by truncating this value to an integral value in mm.
    /// offset: 0x930
    exact_collect_diam_cm: f32,

    /// The minimum katamari diameter needed to collect this object (in mm).
    /// offset: 0x934
    collect_diam_mm: i32,

    /// The sizes of the prop's AABB
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

    /// The prop's triangle mesh used to collide the prop with the katamari while the
    /// katamari is too small to attach the prop.
    /// The mesh is stored in the "mono data" glob.
    /// offset: 0x960
    collision_mesh: (),

    /// (??) The additional transform applied to the prop while it is attached to the katamari.
    /// offset: 0x968
    attached_transform: Mat4,

    /// While attached, the offset from the katamari center to the prop's center.
    /// offset: 0x9e8
    kat_center_offset: Vec4,

    /// (??) The velocity of the katamari when it collided with the prop.
    /// offset: 0x9f8
    kat_collision_vel: Vec4,

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

    /// If this prop is attached, points to the prop that was attached before this (if one exists).
    /// offset: 0xa28
    collected_before: Option<Prop>,

    /// If this prop is attached, points to the prop that was attached after this (if one exists).
    /// offset: 0xa30
    collected_next: Option<Prop>,

    mono_data_ptrs: MonoDataPropPtrs,

    /// True if this prop has a twin prop on the Gemini mission.
    /// offset: 0xb10
    has_twin: bool,

    /// The twin ID of this prop, if any. Both twins should have the same unique ID.
    twin_id: Option<u16>,

    /// If this prop has a Gemini twin, points to the twin prop.
    /// offset: 0xb18
    twin_prop: Option<Prop>,

    /// The transform matrix of this prop when it was loaded.
    /// offset: 0xb24
    init_transform: Mat4,

    /// The Euler angles of this prop when it was loaded.
    /// offset: 0xbb8
    init_rotation_vec: Vec4,
}

pub type Prop = Rc<RefCell<PropNode>>;

impl PropNode {
    pub fn new(state: &mut GameState, args: &AddPropArgs) -> Self {
        let config = NamePropConfig::get(args.name_idx.into());

        // initialize rotation matrix to identity
        new_mat4_id!(rotation_mat);
        // TODO: rotate `rotation_mat` by rotation angles in `args`

        // save the initial rotation
        new_mat4_copy!(init_rotation_mat, rotation_mat);

        // initialize unattached transform to the initial rotation mat
        new_mat4_copy!(unattached_transform, rotation_mat);
        new_mat4_copy!(init_transform, unattached_transform);

        // TODO
        // lines 104-107 of `prop_init` (init comment)
        // lines 108-149 of `prop_init` (init motion)
        // lines 150-162 of `prop_init` (init random group)
        // lines 163-190 of `prop_init` (init twin)
        // lines 348-349 (find first subobject)
        // lines 350-357 (init motion scripts)
        // lines 358-364 (init aabb)
        // lines 365-367 (init links to other props)
        // lines 368-371, 392-401 (init wobble state)
        // lines 373-384 (init generated prop??)
        // line 385
        // lines 386-391 (init fish)

        let mono_data_ptrs = state
            .mono_data
            .props
            .get(args.name_idx as usize)
            .unwrap()
            .clone();

        PropNode {
            ctrl_idx: state.global.get_next_ctrl_idx().try_into().unwrap(),
            name_idx: args.name_idx,
            flags: 0,
            global_state: PropGlobalState::Unattached,
            flags2: 0,
            force_disabled: false,
            display_off: false,
            alpha: 1.0,
            move_type: args.mono_move_type,
            hit_on_area: args.mono_hit_on_area.try_into().unwrap(),
            link_action: max_to_none!(u16, args.link_action),
            innate_motion_type: config.innate_motion_type,
            innate_motion_state: 0,
            has_path_motion_action: false,
            disable_wobble: args.shake_off_flag != 0,
            has_moving_motion: false,
            next_sibling: None,
            first_child: None,
            init_area: state.global.area.unwrap(),
            init_pos: [args.pos_x, args.pos_y, args.pos_z, 1.0],
            rotation_mat: rotation_mat,
            pos: [args.pos_x, args.pos_y, args.pos_z, 1.0],
            rotation_vec: [args.rot_x, args.rot_y, args.rot_z, args.rot_w],
            scale: [1.0, 1.0, 1.0, 1.0],
            last_pos: [args.pos_x, args.pos_y, args.pos_z, 1.0],
            last_rotation_vec: [args.rot_x, args.rot_y, args.rot_z, args.rot_w],
            init_rotation_vec: [args.rot_x, args.rot_y, args.rot_z, args.rot_w],
            unattached_transform: unattached_transform,
            init_rotation_mat: init_rotation_mat,
            init_transform: init_transform,
            motion_transform: [0.0; 16],
            extra_action_type: max_to_none!(u16, args.extra_action_type),
            unique_name_id: max_to_none!(u16, args.unique_name_id),
            display_off_area: args.disp_off_area_no,
            vs_drop_flag: args.vs_drop_flag != 0,
            unattached_state: PropUnattachedState::Normal,
            animation_type: PropAnimationType::Waiting,
            comment_id: max_to_none!(u16, args.comment_id),
            comment_group_id: max_to_none!(u16, args.comment_group_id),
            tree_group_id: None, // TODO
            stationary: true,
            force_no_wobble: false,
            onattach_added_vol: 0.0,
            onattach_remain_ticks: 0,
            onattach_game_time_ms: 0,
            attached_transform: [0.0; 16],
            kat_center_offset: [0.0; 4],
            kat_collision_vel: [0.0; 4],
            last_dist_to_p0: 0.0,
            last_dist_to_p1: 0.0,
            dist_to_p0: 0.0,
            dist_to_p1: 0.0,
            remain_knockoff_volume: 0.0,
            collected_before: None,
            collected_next: None,
            has_twin: args.twin_id != u16::MAX,
            twin_id: max_to_none!(u16, args.twin_id),
            mono_data_ptrs: mono_data_ptrs,

            // TODO
            first_subobject: None, // TODO
            script_0x560: None,    // TODO
            motion_script: None,   // TODO
            innate_script: None,
            parent_prop: None,            // TODO
            motion_action_type: None,     // TODO
            behavior_type: None,          // TODO
            alt_motion_action_type: None, // TODO
            radius: 0.0,
            aabb_vol_m3: 0.0,
            compare_vol_m3: 0.0,
            added_vol_m3: 0.0,
            exact_collect_diam_cm: 0.0,
            collect_diam_mm: 0,
            aabb_size: [0.0; 3],
            collision_mesh: (),
            twin_prop: None,
        }
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
        *collect_diam = self.collect_diam_mm;
    }

    pub fn set_disabled(&mut self, force_disabled: i32) {
        self.force_disabled = force_disabled != 0;
    }

    pub fn set_parent(&mut self, parent: Option<Box<Prop>>) {
        // if let Some(parent_box) = parent {
        //     self.flags |= 0x2;
        //     self.parent_prop = *parent;
        //     parent_box.disable_wobble = true;
        // } else {
        //     self.flags &= 0xfd;
        //     self.parent_prop = None;
        //     self.tree_group_id = None;
        // }
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
        if self.collect_diam_mm <= kat_diam_int {
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
        if self.display_off {
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
}
