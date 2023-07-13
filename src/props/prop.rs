use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
};

use gl_matrix::{
    common::{Mat4, Vec3},
    mat4::{self},
    vec3,
};
use serde::{Deserialize, Serialize};

use crate::{
    collision::{mesh::Mesh, util::max_transformed_y},
    constants::{FRAC_1_3, FRAC_PI_750, UNITY_TO_SIM_SCALE, VEC3_ZERO, _4PI},
    debug::DEBUG_CONFIG,
    global::GlobalState,
    macros::{
        max_to_none, modify_translation, new_mat4_copy, scale_translation, set_translation,
        temp_debug_log, vec3_from,
    },
    mission::state::MissionState,
    mono_data::{MonoData, PropAabbs, PropMonoData},
    player::{katamari::Katamari, Player},
    props::config::NamePropConfig,
    util::scale_sim_transform,
};

use super::{
    comments::KingCommentState,
    motion::{behavior::PropBehavior, RotationAxis},
    random::RandomPropsState,
    PropsState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropGlobalState {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PropUnattachedState {
    /// The default state that most props are in most of the time.
    Normal = 0,

    InTrajectory = 1,
    State2 = 2,
    State3 = 3,

    /// Active when a spherical object (capable of inelastic collisions while unattached)
    /// is rolling with force from such an inelastic collision.
    InelasticRoll = 4,

    /// Active after an airborne prop bounces off a floor once.
    AirborneBounced = 5,
}

impl Default for PropUnattachedState {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

bitflags::bitflags! {
    /// Definition of the 0x6 offset field of `Prop`, which is a 1-byte bitfield.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct PropFlags1: u8 {
        /// True when the prop has a parent.
        const HasParent = 0x2;

        /// (??)
        const IntangibleChild = 0x4;

        /// (??) related to umbrellas, toy dispensers, and other props?
        /// maybe encodes that something happens when the katamari bonks it?
        const Unknown_0x8 = 0x8;

        /// (??) probably related to intangible airborne props
        const AirborneFlag_0x10 = 0x10;

        /// (??) probably related to intangible airborne props
        const AirborneWithReactiveChild = 0x20;

        /// (??) True when the prop (1) has the "hop" motion action and (2) is mid-hop in the air.
        const Hop = 0x40;

        /// (??) probably related to intangible airborne props
        const DetachedWhileKatamariStuck = 0x80;
    }
}

bitflags::bitflags! {
    /// Definition of the 0x8 offset field of `Prop`, which is a 1-byte bitfield.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct PropFlags2: u8 {
        /// True if the prop is following its parent prop.
        const FollowParent = 0x1;

        // True if the prop is wobbling after being hit by the katamari
        const Wobble = 0x2;

        /// (??) set in `apply_trajectory`
        const UnderTrajectory = 0x4;

        /// True if the prop is fleeing from the katamari.
        const Flee = 0x8;

        /// (??) True if the prop has the "spinning fight" behavior seen in e.g.
        /// "Judo Contest" and "Sumo Bout" objects.
        const SpinningFight = 0x10;

        /// (??) Seems like something to do with parent/child links.
        const Unknown0x80 = 0x80;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropTrajectoryType {
    Normal,

    /// The trajectory taken by unattached props which are hit by the katamari and sent flying into
    /// the "airborne intangible" state.
    HitAirborne,

    /// The trajectory taken by drinks ejected from bonked vending machines.
    VendingMachine,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Prop {
    /// The unique id of this prop.
    /// offset: 0x0
    ctrl_idx: u16,

    /// The object type of this prop.
    /// offset: 0x2
    name_idx: u16,

    /// A set of flags encoding special prop behaviors.
    /// offset: 0x6
    flags: PropFlags1,

    /// offset: 0x7
    pub global_state: PropGlobalState,

    /// A second set of flags encoding special prop behaviors.
    /// offset: 0x8
    flags2: PropFlags2,

    /// If true, the prop won't update.
    /// offset: 0x9
    disabled: bool,

    /// If false, the prop won't be displayed, but it will still be tangible.
    /// offset: 0xa
    display_on: bool,

    /// (??) Presumably, true when the prop is attached to a katamari, but this is redundant
    /// offset: 0xd
    is_attached: bool,

    /// The alpha level of the prop (fades out as it gets further from the player camera).
    /// offset: 0x14
    alpha: f32,

    /// (??) The type of movement that this prop performs.
    /// offset: 0x16
    move_type: Option<u16>,

    /// The prop is intangible until the player loads this area.
    /// offset: 0x1a
    hit_on_area: Option<u8>,

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
    /// NOTE: the original simulation keeps a pointer to the prop, but we only store
    /// the control index here because it's safer/rustier.
    /// offset: 0x28
    pub next_sibling: Option<u16>,

    /// The first child of this prop in its family tree.
    /// NOTE: the original simulation keeps a pointer to the prop, but we only store
    /// the control index here because it's safer/rustier.
    /// offset: 0x30
    pub first_child: Option<u16>,

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
    pub pos: Vec3,

    /// The prop's rotation as Euler angles.
    /// offset: 0xa0
    rotation_vec: Vec3,

    /// The prop's scale. (unused in simulation)
    /// offset: 0xb0
    scale: Vec3,

    /// The prop's position on the previous tick.
    /// offset: 0xc0
    pub last_pos: Vec3,

    /// The prop's rotation as Euler angles on the previous tick.
    /// offset: 0xd0
    last_rotation_vec: Vec3,

    /// The prop's velocity while under a trajectory, which can be imparted either
    /// by the katamari (when hit by the katamari or detached from the katamari),
    /// or by a prop behavior (e.g. launched from a vending machine or russian doll).
    /// offset: 0xe0
    trajectory_velocity: Vec3,

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
    // TODO_SUBOBJ: make this a vector of subobjects
    #[serde(skip)]
    first_subobject: Option<Box<PropSubobject>>,

    /// (??) a script that seems to only be used for policeman gunshots in reroll
    /// offset: 0x560
    // TODO_PROP_MOTION: make this an enum that determines which script to call
    #[serde(skip)]
    script_0x560: Option<Box<PropScript>>,

    /// The prop's motion script, if it has one.
    /// offset: 0x568
    // TODO_PROP_MOTION: make this an enum that determines which script to call
    #[serde(skip)]
    motion_script: Option<Box<PropScript>>,

    /// The prop's innate script, if it has one.
    /// offset: 0x570
    // TODO_PROP_MOTION: make this an enum that determines which script to call
    #[serde(skip)]
    innate_script: Option<Box<PropScript>>,

    /// The prop's parent, if it has one.
    /// NOTE: the original simulation keeps a pointer to the prop, but we only store
    /// the control index here because it's safer/rustier.
    /// offset: 0x578
    pub parent: Option<u16>,

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
    behavior_type: Option<PropBehavior>,

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

    /// True if the prop isn't moving (used when the prop has a move type that makes it
    /// occasionally stop moving, e.g. random roaming movement).
    /// offset: 0x58e
    stationary: bool,

    /// If true, the prop can't wobble when the katamari collides with it.
    /// Temporarily set by motion actions such as hops.
    /// offset: 0x58f
    force_no_wobble: bool,

    /// The AABB of this prop encoded as a collision mesh.
    /// offset: 0x5e0
    #[serde(skip)]
    pub aabb_mesh: Option<Rc<Mesh>>,

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

    /// The base volume added to the katamari when this prop is attached (in m^3).
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
    // TODO_SERIAL: set this after load
    #[serde(skip)]
    mono_data: Option<Rc<PropMonoData>>,

    /// The mesh used for non-collection collisions with this prop.
    /// offset: 0x960
    #[serde(skip)]
    // TODO_SERIAL: set this after load
    collision_mesh: Option<Rc<Mesh>>,

    /// (??) The additional transform applied to the prop while it is attached to the katamari.
    /// offset: 0x968
    pub init_attached_transform: Mat4,

    /// (??)
    /// offset: 0x9a8
    pub attached_transform: Mat4,

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
    attach_life: f32,

    /// (??) True if the prop is near a player
    /// offset: 0xa20
    pub near_player: bool,

    /// A multiple of the prop's volume that seems to be used somewhere. Hell if I know
    /// offset: 0xa24
    weird_vol_multiple: f32,

    /// If this prop is attached, points to the prop that was attached before this (if one exists).
    /// NOTE: this linked list of collected props is replaced by the vector `Katamari::collected_props`
    /// offset: 0xa28
    // last_collected_prop: Option<WeakPropRef>,

    /// If this prop is attached, points to the prop that was attached after this (if one exists).
    /// offset: 0xa30
    /// NOTE: this linked list of collected props is replaced by the vector `Katamari::collected_props`
    // next_collected_prop: Option<WeakPropRef>,

    /// True if this prop's collision mesh contacts a katamari.
    /// offset: 0xa41
    hit_katamari: bool,

    /// If none, this prop's collision mesh doesn't contact any katamaris.
    /// If some, the player index of the katamari meeting this prop's collision mesh.
    /// offset: 0xa42
    contact_katamari_idx: Option<u8>,

    /// When attached, the index of the katamari collision ray nearest to this prop.
    /// offset: 0xa88
    nearest_kat_ray_idx: Option<u16>,

    /// If >0, this prop is intangible to the katamari. Decrements by 1 each tick.
    /// offset: 0xa8a
    pub intangible_timer: u16,

    /// If true, this prop is intangible to the katamari.
    /// offset: 0xa8e
    pub force_intangible: bool,

    /// The cooldown on when this prop can scream, when >0. If 0, the prop is ready to scream again
    /// upon collision with the katamari.
    /// offset: 0xa8f
    pub scream_cooldown_timer: u8,

    /// True if this prop has a twin prop on the Gemini mission.
    /// offset: 0xb10
    has_twin: bool,

    /// The twin ID of this prop, if any. Both twins should have the same unique ID.
    twin_id: Option<u16>,

    /// If this prop has a Gemini twin, points to the twin prop.
    /// NOTE: the original simulation keeps a pointer to the prop, but we only store
    /// the control index here because it's safer/rustier.
    /// offset: 0xb18
    twin_prop: Option<u16>,

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
pub type MeshRef = Rc<RefCell<Mesh>>;

impl Display for Prop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Prop(ctrl={})", self.ctrl_idx)
    }
}

impl Prop {
    pub fn new(
        ctrl_idx: u16,
        args: &AddPropArgs,
        area: u8,
        mono_data: &MonoData,
        global: &mut GlobalState,
        comments: &mut KingCommentState,
        random: &mut RandomPropsState,
    ) -> PropRef {
        Rc::new(RefCell::new(Prop::new_node(
            ctrl_idx, args, area, mono_data, global, comments, random,
        )))
    }

    /// Create a new `Prop` object.
    /// Mostly follows the function `prop_init`.
    /// offset: 0x4e950
    pub fn new_node(
        ctrl_idx: u16,
        args: &AddPropArgs,
        area: u8,
        mono_data: &MonoData,
        global: &mut GlobalState,
        comments: &mut KingCommentState,
        random: &mut RandomPropsState,
    ) -> Self {
        // if the prop belongs to a random group, determine its name index by sampling
        // the random group
        let name_idx = if args.loc_pos_type != 0 {
            let random_name_idx = random.sample_group(global, args.random_group_id as usize);
            random_name_idx.unwrap_or(args.name_idx)
        } else {
            args.name_idx
        };

        let prop_mono_data = &mono_data.props[name_idx as usize];

        let config = NamePropConfig::get(name_idx.into());

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

        // lines 108-149 of `prop_init` (init motion)
        // lines 163-190 of `prop_init` (init twin/catch_count_b)

        // lines 348-349 (find first subobject)
        // lines 350-357 (init motion scripts)
        // lines 368-371, 392-401 (init wobble state)
        // lines 373-384 (init generated prop??)
        // line 385
        // lines 386-391 (init fish)

        let mut result = Prop {
            ctrl_idx,
            name_idx,
            flags: PropFlags1::default(),
            global_state: PropGlobalState::Unattached,
            flags2: PropFlags2::default(),
            disabled: false,
            display_on: false,
            alpha: 1.0,
            move_type: max_to_none!(u16, args.mono_move_type),
            hit_on_area: (max_to_none!(u16, args.mono_hit_on_area)).map(|a| a as u8),
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
            init_attached_transform: [0.0; 16],
            kat_center_offset: [0.0; 3],
            kat_collision_vel: [0.0; 3],
            last_dist_to_p0: 0.0,
            last_dist_to_p1: 0.0,
            dist_to_p0: 0.0,
            dist_to_p1: 0.0,
            attach_life: 0.0,
            has_twin: args.twin_id != u16::MAX,
            twin_id: max_to_none!(u16, args.twin_id),
            nearest_kat_ray_idx: None,
            intangible_timer: 0,
            scream_cooldown_timer: 0,
            parent: None,

            // initialized in `self.init_aabb_and_volume()`
            aabb_mesh: None,
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
            near_player: false,
            force_intangible: false,
            hit_katamari: false,
            contact_katamari_idx: None,
            is_attached: false,
            attached_transform: [0.0; 16],
            collision_mesh: None,
            trajectory_velocity: [0.0; 3],
        };

        if let Some(aabbs) = &prop_mono_data.aabbs {
            result.init_aabb_and_volume(aabbs, config);
        }

        result.collision_mesh = match &prop_mono_data.collision_mesh {
            mesh @ Some(_) => mesh.as_ref().map(|m| m.clone()),
            None => result.aabb_mesh.as_ref().map(|mesh| mesh.clone()),
        };

        // move random-spawn props vertically so that they're resting on the ground
        if args.loc_pos_type != 0 {
            let y_offset = result.max_aabb_y();
            result.pos[1] += y_offset;
            result.init_pos[1] += y_offset;
        }

        // handle king comment (unused since unity also handles it, apparently)
        if let Some(group_idx) = result.comment_group_id {
            comments.add_to_group(group_idx);
        }

        // note the conditional call to `prop_init_tree_links` here in the original sim,
        // but the condition to call it appears to never be true in reroll.

        result.mono_data = Some(prop_mono_data.clone());

        result
    }

    /// Initialize the prop's AABB and volume
    /// offset: 0x27750
    fn init_aabb_and_volume(&mut self, aabbs: &PropAabbs, config: &NamePropConfig) {
        // TODO_PARAM: refactor this as a simulation param
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
        self.aabb_mesh = Some(Rc::new(root_aabb.compute_mesh(&self.aabb_vertices)));
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

    pub fn get_position(&self) -> &Vec3 {
        &self.pos
    }

    pub fn get_global_state(&self) -> PropGlobalState {
        self.global_state
    }

    pub fn get_unattached_state(&self) -> PropUnattachedState {
        self.unattached_state
    }

    pub fn get_hit_on_area(&self) -> Option<u8> {
        self.hit_on_area
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn get_attach_diam_mm(&self) -> i32 {
        self.attach_diam_mm
    }

    pub fn get_exact_attach_diam_cm(&self) -> f32 {
        self.exact_attach_diam_cm
    }

    pub fn get_attach_vol_m3(&self) -> f32 {
        self.attach_vol_m3
    }

    pub fn get_flags(&self) -> &PropFlags1 {
        &self.flags
    }

    pub fn get_flags_mut(&mut self) -> &mut PropFlags1 {
        &mut self.flags
    }

    pub fn get_flags2(&self) -> &PropFlags2 {
        &self.flags2
    }

    pub fn get_flags2_mut(&mut self) -> &mut PropFlags2 {
        &mut self.flags2
    }

    pub fn get_move_type(&self) -> Option<u16> {
        self.move_type
    }

    pub fn get_behavior_type(&self) -> Option<PropBehavior> {
        self.behavior_type
    }

    pub fn get_stationary(&self) -> bool {
        self.stationary
    }

    pub fn get_compare_vol_m3(&self) -> f32 {
        self.compare_vol_m3
    }

    pub fn get_aabb_size(&self) -> &Vec3 {
        &self.aabb_size
    }

    pub fn get_aabb_radius(&self) -> f32 {
        self.aabb_radius
    }

    pub fn get_aabb_mesh(&self) -> Option<Rc<Mesh>> {
        self.aabb_mesh.as_ref().map(|m| m.clone())
    }

    pub fn get_collision_mesh(&self) -> Option<Rc<Mesh>> {
        self.collision_mesh.as_ref().map(|s| s.clone())
    }

    pub fn get_aabb_min_point(&self) -> &Vec3 {
        // &self.aabb_mesh.sectors[0].aabb.min
        &VEC3_ZERO
    }

    pub fn get_unattached_transform(&self) -> &Mat4 {
        &self.unattached_transform
    }

    pub fn do_unattached_translation(&mut self, translation: &Vec3) {
        modify_translation!(self.unattached_transform, +=, translation);
    }

    pub fn get_attached_transform(&self) -> &Mat4 {
        &self.attached_transform
    }

    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }

    pub fn get_has_twin(&self) -> bool {
        self.has_twin
    }

    pub fn get_twin(&self) -> Option<u16> {
        self.twin_prop
    }

    pub fn get_nearest_kat_ray_idx(&mut self) -> Option<u16> {
        self.nearest_kat_ray_idx
    }

    pub fn set_nearest_kat_ray_idx(&mut self, value: Option<u16>) {
        self.nearest_kat_ray_idx = value;
    }

    pub fn get_mono_data(&self) -> Option<&Rc<PropMonoData>> {
        self.mono_data.as_ref()
    }

    pub fn get_scream_cooldown_timer(&self) -> u8 {
        self.scream_cooldown_timer
    }

    pub fn reset_scream_cooldown_timer(&mut self) {
        // TODO_PARAM
        let SCREAM_COOLDOWN_TICKS = 0xf;

        self.scream_cooldown_timer = SCREAM_COOLDOWN_TICKS;
    }

    pub fn set_kat_collision_vel(&mut self, kat_collision_vel: &Vec3) {
        self.kat_collision_vel = *kat_collision_vel;
    }

    pub fn decay_init_attached_transform(&mut self, decay: f32) {
        scale_translation!(self.init_attached_transform, decay);
    }

    pub fn get_force_no_wobble(&self) -> bool {
        self.force_no_wobble
    }

    pub fn get_dist_to_katamari(&self, player: i32) -> f32 {
        match player {
            0 => self.dist_to_p0,
            1 => self.dist_to_p1,
            _ => {
                panic!("tried to read distance of nonexistent player: {}", player);
            }
        }
    }

    pub fn set_katamari_contact(&mut self, player_idx: u8) {
        self.hit_katamari = true;
        self.contact_katamari_idx = Some(player_idx);
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

    pub fn set_disabled(&mut self, value: i32) {
        self.disabled = value != 0;
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn get_attach_life(&self) -> f32 {
        self.attach_life
    }

    pub fn set_attach_life(&mut self, attach_life: f32) {
        self.attach_life = attach_life;
    }

    pub fn set_no_parent(&mut self) {
        self.flags.remove(PropFlags1::HasParent);
        self.parent = None;
        self.tree_id = None;
    }

    pub fn set_parent(&mut self, _props: &PropsState, parent_ctrl_idx: u16, tree_group_id: u16) {
        self.flags.insert(PropFlags1::HasParent);
        self.parent = Some(parent_ctrl_idx);

        if self.link_action == Some(PropLinkAction::IntangibleChild) {
            self.flags |= PropFlags1::IntangibleChild;
        }

        self.tree_id = Some(tree_group_id);

        // TODO: line 53: `prop_update_rotation_from_parent(self)`

        mat4::identity(&mut self.motion_transform);
    }

    // Compute the root prop of this prop's tree.
    pub fn get_root_ref(&self, props: &PropsState) -> PropRef {
        if let Some(parent_ctrl_idx) = self.parent {
            props
                .get_prop(parent_ctrl_idx as usize)
                .unwrap()
                .borrow()
                .get_root_ref(props)
        } else {
            props.get_prop(self.ctrl_idx as usize).unwrap().clone()
        }
    }

    /// Add `child` as a child of this prop by adding it to the end of the
    /// sibling list.
    pub fn add_child(&mut self, props: &PropsState, child_ctrl_idx: u16) {
        if self.first_child.is_none() {
            self.first_child = Some(child_ctrl_idx);
        } else if let Some(first_child_idx) = self.first_child {
            if let Some(first_child_ref) = props.get_prop(first_child_idx as usize) {
                first_child_ref
                    .clone()
                    .borrow_mut()
                    .add_sibling(props, child_ctrl_idx);
            }
        }
    }

    /// Traverse this prop's sibling list, adding `sibling` to the end.
    pub fn add_sibling(&mut self, props: &PropsState, sibling_ctrl_idx: u16) {
        if self.next_sibling.is_none() {
            self.next_sibling = Some(sibling_ctrl_idx);
        } else if let Some(next_sibling_idx) = self.next_sibling {
            if let Some(next_sibling_ref) = props.get_prop(next_sibling_idx as usize) {
                next_sibling_ref
                    .clone()
                    .borrow_mut()
                    .add_sibling(props, sibling_ctrl_idx);
            }
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
        if self.disabled {
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

        // TODO_DESTROY: destroy the prop here
        // if (should_destroy) {
        //     self.destroy();
        // }

        should_destroy
    }

    /// offset: 0x4f8e0
    pub fn destroy(&mut self) {
        if DEBUG_CONFIG.log_destroyed_props {
            temp_debug_log!(
                "  destroying prop: ctrl_idx={}, name_idx={}",
                self.ctrl_idx,
                self.name_idx
            );
        }

        self.disabled = true;
        self.display_on = false;
        // TODO: remove this from the list `Katamari::attached_props`
        // TODO_LINKS: `prop_remove_refs_from_props()`
        // TODO_SUBOBJ: `prop_destroy:14-25`
        self.first_subobject = None;
    }

    /// Contains most the behavior of `Katamari::attach_prop` that writes to the attached prop.
    /// offset: 0x28fe4 (mid-function)
    pub fn attach_to_kat(&mut self, kat: &Katamari) {
        self.attach_life = self.compare_vol_m3;

        // TODO_LOW: fix these two fields
        self.onattach_remain_ticks = 0;
        self.onattach_game_time_ms = 0;

        self.is_attached = true;
        self.global_state = PropGlobalState::Attached;
        self.onattach_added_vol = self.attach_vol_m3 * kat.get_attach_vol_penalty();
        self.kat_center_offset = vec3_from!(-, self.pos, kat.get_center());

        let mut init_attached_transform = mat4::create();
        if !NamePropConfig::get(self.name_idx).is_unhatched_egg {
            mat4::copy(&mut init_attached_transform, &self.unattached_transform);
        } else {
            // TODO: `prop_adjust_bbox_when_hatching_egg`
        }
        set_translation!(init_attached_transform, self.kat_center_offset);

        let mut kat_rot_inv = mat4::create();
        mat4::transpose(&mut kat_rot_inv, kat.get_rotation_mat());

        // compute the prop's transform both not including and including the katamari's transform
        mat4::multiply(
            &mut self.init_attached_transform,
            &kat_rot_inv,
            &init_attached_transform,
        );
        mat4::multiply(
            &mut self.attached_transform,
            kat.get_transform(),
            &self.init_attached_transform,
        );

        // compute the prop's position from the computed attached transform
        mat4::get_translation(&mut self.pos, &self.attached_transform);
    }

    /// Computes the highest point (in local space) on this prop's AABB
    /// after transforming the AABB with its current rotation matrix.
    pub fn max_aabb_y(&self) -> f32 {
        max_transformed_y(&self.aabb_vertices, &self.rotation_mat)
    }

    pub fn detach_from_katamari(&mut self, mission_state: &MissionState, global: &mut GlobalState) {
        self.attach_life = 0.0;
        self.is_attached = false;
        // all that remains of `prop_remove_refs_from_kat`
        self.intangible_timer = 5;

        if mission_state.mission_config.is_theme_object(self.name_idx) {
            global.catch_count_b -= 1;
        }
    }

    pub fn apply_trajectory(
        &mut self,
        init_vel: &Vec3,
        trajectory: PropTrajectoryType,
        _gravity: f32,
    ) {
        match trajectory {
            PropTrajectoryType::HitAirborne => {
                // TODO_AIRBORNE
            }
            PropTrajectoryType::Normal | PropTrajectoryType::VendingMachine => {
                // TODO_LINK
                if false
                /*self.first_child.is_some() && child_prop.link_action == ParentCollectedReaction */
                {
                    self.flags.insert(PropFlags1::AirborneWithReactiveChild)
                }
                self.global_state = PropGlobalState::Unattached;
                self.unattached_state = PropUnattachedState::InTrajectory;
                // TODO
                if false
                /* global.detaching_props_from_stuck_kat*/
                {
                    self.flags.insert(PropFlags1::DetachedWhileKatamariStuck);
                }
            }
        }

        self.trajectory_velocity = *init_vel;
        // TODO_PROP_MOTION:
        /*  (prop->motionData).numAirborneBounces = 0;
        (prop->motionData).field14_0x19 = 0;
        (prop->motionData).landed = false;
        (prop->motionData).fishHopState = STATE0 */
        // TODO: `global.airborne_prop_gravity = gravity`

        if self.global_state == PropGlobalState::AirborneIntangible {
            // TODO_AIRBORNE
        } else {
            if trajectory != PropTrajectoryType::VendingMachine {
                mat4::identity(&mut self.rotation_mat);
                vec3::zero(&mut self.rotation_vec);
            }

            let _rot_axis = if self.trajectory_velocity[0] <= self.trajectory_velocity[2] {
                RotationAxis::X
            } else {
                RotationAxis::Z
            };

            // TODO_PROP_MOTION:
            /*  (prop->motionData).rotationAxis = AVar5;
            (prop->motionData).field17_0x1c = 0.2;
            (prop->motionData).field18_0x20 = 0.0 */

            self.flags2.insert(PropFlags2::UnderTrajectory);
        }
    }
}

impl Prop {
    /// Update a prop when not in the ending mission, i.e. the "normal" prop update logic.
    /// offset: 0x50050 (note: that offset's function loops over all props, and this function is
    ///                  one iteration of that loop.)
    pub fn update_nonending(&mut self, player: &Player) {
        if self.disabled {
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
        self.flags.remove(PropFlags1::AirborneWithReactiveChild);
        self.flags2.remove(PropFlags2::FollowParent);
        self.move_type = None;
        self.has_motion = false;
        self.unattached_state = PropUnattachedState::Normal;
        self.animation_type = PropAnimationType::Animation2;
        self.is_following_path = false;
    }

    /// Called from `Katamari::update_rays_with_attached_props` to position this prop while
    /// it is attached to the katamari, based on the katamari's transform `kat_transform`.
    /// offset: 0x2c330 (mid-function)
    pub fn update_transform_when_attached(&mut self, kat_transform: &Mat4) {
        mat4::multiply(
            &mut self.attached_transform,
            &kat_transform,
            &self.init_attached_transform,
        );
        mat4::get_translation(&mut self.pos, &self.attached_transform);
    }

    /// TODO_PROPS
    /// Update logic for a prop that's airborne.
    /// offset: 0x50eb0
    fn update_airborne_intangible(&mut self) {}

    /// Update logic for props which have a parent.
    /// offset: 0x2e030
    fn update_child_link(&mut self) {
        if self.parent.is_none() {
            self.flags.remove(PropFlags1::HasParent);
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
