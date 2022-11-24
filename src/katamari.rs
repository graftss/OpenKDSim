mod collision;
pub mod mesh;

use gl_matrix::{
    common::{Mat4, Vec4},
    mat4, vec4,
};

use crate::{
    constants::{
        TRANSFORM_X_POS, TRANSFORM_Y_POS, TRANSFORM_Z_POS, UNITY_TO_SIM_SCALE, VEC4_X_NEG,
        VEC4_Y_POS, VEC4_ZERO, _4PI_3,
    },
    prop::PropRef,
    simulation_params::SimulationParams,
};

use self::{
    collision::{KatCollisionRayType, KatCollisionRays, ShellRay},
    mesh::KatMesh,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KatPushDir {
    Forwards,
    Backwards,
    Sideways,
}

impl Default for KatPushDir {
    fn default() -> Self {
        Self::Forwards
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KatInclineMoveType {
    MoveFlatground,
    MoveUphill,
    MoveDownhill,
}

impl Default for KatInclineMoveType {
    fn default() -> Self {
        Self::MoveFlatground
    }
}

/// (??) not sure about this
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmType {
    Closest,
    Closer,
    Close,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct KatPhysicsFlags {
    /// If true, the katamari has no surface contacts.
    /// offset: 0x0
    pub airborne: bool,

    /// If true, the katamari is climbing a wall.
    /// offset: 0x1
    pub climbing_wall: bool,

    /// If true, the katamari is at its maximum climb height (so it can't climb higher).
    /// offset: 0x2
    pub at_max_climb_height: bool,

    /// If true, the katamari is braking.
    /// offset: 0x3
    pub braking: bool,

    /// If true, the katamari just bonked something (only true the frame it bonks).
    /// offset: 0x4
    pub bonked: bool,

    /// If true, the katamari is contacting a wall.
    /// offset: 0x5
    pub contacts_wall: bool,

    /// If true, the katamari is contacting a wall.
    /// offset: 0x6
    pub contacts_floor: bool,

    /// If true, the katamari is in water.
    /// offset: 0x7
    pub in_water: bool,

    /// (??) copy of `in_water`
    /// offset: 0x8
    pub in_water_copy: bool,

    /// (??) If true, the katamari was hit by a moving prop.
    /// offset: 0x9
    pub hit_by_moving_prop: bool,

    /// (??) If true, the katamari is contacting a prop.
    /// offset: 0xa
    pub _contacts_prop: bool,

    /// (??) If true, the katamari is contacting the bottom of a downward-slanting surface.
    /// (e.g. can be triggered under mas1 table by setting simulation+0x71614 to 3, which
    /// changes the definition of how downward-slanting such a surface needs to be)
    /// offset: 0xb
    pub contacts_slanted_ceiling: bool,

    /// (??) If true, the katamari moved more than its own radius during the last tick.
    /// offset: 0xc
    pub moved_more_than_radius_0xc: bool,

    /// If true, the katamari is contacting a prop.
    /// offset: 0xd
    pub contacts_prop: bool,

    /// (??) A shell ray which collided with something
    /// offset: 0xe
    pub hit_shell_ray: Option<ShellRay>,

    /// If true, the katamari is completely submerged underwater.
    /// offset: 0xf
    pub under_water: bool,

    /// If true, the katamari is not moving.
    /// offset: 0x10
    pub stationary: bool,

    /// (??) The type of boundary ray currently acting as the pivot.
    /// offset: 0x11
    pub pivot_ray: Option<KatCollisionRayType>,

    /// If true, the katamari is contacting a non-flat floor (normal < 0.9999).
    /// offset: 0x12
    pub on_sloped_floor: bool,

    /// If true, the katamari is contacting a flat floor (normal >= 0.9999).
    /// offset: 0x13
    pub on_flat_floor: bool,

    /// (??)
    /// offset: 0x14
    pub moved_too_much_0x14: bool,

    /// (??)
    /// offset: 0x15
    pub incline_move_type: KatInclineMoveType,

    /// If true, the katamari is spinning
    /// offset: 0x16
    pub wheel_spin: bool,

    /// True if not boosting AND input below the "min push" threshold.
    /// offset: 0x17
    pub no_input_push: bool,

    /// True if the katamari moved more than its radius on the previous tick.
    /// offset: 0x19
    pub moved_more_than_rad: bool,

    /// True if the katamari is considered stuck between walls.
    /// offset: 0x1a
    pub stuck_between_walls: bool,

    /// (??)
    /// offset: 0x1b
    pub is_colliding: bool,

    /// True if the katamari should emit the "puff of smoke" vfx as it moves.
    /// By default this occurs when it's over 12m in the World stage.
    /// offset: 0x1c
    pub should_emit_smoke: bool,

    /// (??)
    /// offset: 0x1d
    pub moved_more_than_rad_0x1d: bool,
}

/// A group of flags which mostly record if the katamari is contacting certain special types of surfaces
/// with non-standard `HitAttribute` values.
#[derive(Debug, Default, Clone, Copy)]
pub struct KatHitFlags {
    /// If true, ignores "pushing downward" incline effect (e.g. on park entrance steps)
    /// offset: 0x0
    pub force_flatground: bool,

    /// (??) True while climbing a prop, and also certain surfaces e.g. park steps.
    /// offset: 0x1
    pub wall_climb_free: bool,

    /// (??)
    /// offset: 0x2
    pub small_ledge_climb: bool,

    /// True when speed should be uncapped while moving downhill (e.g. big hill in Town stage)
    /// offset: 0x3
    pub speed_check_off: bool,

    /// (??)
    /// offset: 0x4
    pub flag_0x4: bool,

    /// (??)
    /// offset: 0x5
    pub flag_0x5: bool,

    /// True when the camera should be zoomed in (e.g. under House stage table, under trees outside World park).
    /// offset: 0x6
    pub special_camera: bool,

    /// (??) Applies when contacting a "NoReactionNoSlope" surface
    /// offset: 0x7
    pub no_reaction_no_slope: bool,

    /// True if the "turntable" spin effect should be applied.
    /// offset: 0x8
    pub on_turntable: bool,

    /// True when contacting a "WallClimbDisabled" surface.
    /// If true, climbing is disabled. (e.g. the legs under the mas1 table)
    /// offset: 0x9
    pub wall_climb_disabled: bool,

    /// (??) True when contacting a "MapSemiTranslucent" surface.
    /// offset: 0xa
    pub map_semi_translucent: bool,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct KatVelocity {
    /// Current velocity
    /// offset: 0x0
    pub velocity: Vec4,

    /// Current unit velocity
    /// offset: 0x10
    pub velocity_unit: Vec4,

    /// (??)
    /// offset: 0x20
    pub speed_orth_on_floor: Vec4,

    /// (??)
    /// offset: 0x30
    pub speed_proj_on_floor: Vec4,

    /// (??)
    /// offset: 0x40
    pub last_vel_accel: Vec4,

    /// (??) Velocity + player acceleration
    /// offset: 0x50
    pub vel_accel: Vec4,

    /// Unit vector of `vel_accel`.
    /// offset: 0x60
    pub vel_accel_unit: Vec4,

    /// (??) Velocity + player accel + gravity
    /// offset: 0x70
    pub vel_accel_grav: Vec4,

    /// Unit vector of `vel_accel_grav`.
    /// offset: 0x80
    pub vel_accel_grav_unit: Vec4,

    /// (??)
    /// offset: 0x90
    pub push_vel_on_floor_unit: Vec4,

    /// Acceleration from gravity
    /// offset: 0xa0
    pub accel_gravity: Vec4,

    /// Acceleration from the contacted floor incline
    /// offset: 0xb0
    pub accel_incline: Vec4,

    /// (??) Acceleration from the contacted floor friction (or some kind of similar force)
    /// offset: 0xc0
    pub accel_ground_friction: Vec4,
}

/// Katamari parameters which vary based on the katamari's current size.
#[derive(Debug, Default, Clone, Copy)]
pub struct KatScaledParams {
    /// Base max speed which acts as a multiplier on the speeds of all movement types.
    pub base_max_speed: f32,

    /// Downward acceleration from gravity.
    pub gravity_accel: f32,

    /// (??) The force exerted when braking with forwards movement.
    pub brake_forwards_force: f32,

    /// (??) The force exerted when braking with backwards movement.
    pub brake_backwards_force: f32,

    /// (??) The force exerted when braking with sideways movement.
    pub brake_sideways_force: f32,

    /// (??) The force exerted when braking boost movement.
    pub brake_boost_force: f32,

    /// Max speed with forwards movement.
    pub max_forwards_speed: f32,

    /// Max speed with backwards movement.
    pub max_backwards_speed: f32,

    /// Max speed with sideways movement.
    pub max_sideways_speed: f32,

    /// Max speed with boost movement.
    pub max_boost_speed: f32,

    /// (??)
    pub push_uphill_accel: f32,

    /// (??)
    pub not_push_uphill_accel: f32,

    /// Acceleration during forwards movement.
    pub forwards_accel: f32,

    /// Acceleration during backwards movement.
    pub backwards_accel: f32,

    /// Acceleration during sideways movement.
    pub sideways_accel: f32,

    /// Acceleration during boost movement.
    pub boost_accel: f32,

    /// The prince's lateral distance from the katamari center.
    pub prince_offset: f32,

    /// (??)
    pub alarm_closest_range: f32,

    /// (??)
    pub alarm_closer_range: f32,

    /// (??)
    pub alarm_close_range: f32,
}

#[derive(Debug, Default)]
pub struct Katamari {
    /// A reference to the vector of katamari meshes.
    /// offset: 0x0
    meshes: Vec<KatMesh>,

    /// The player who owns this katamari.
    /// offset: 0x44
    player: u8,

    /// The index of the katamari mesh being used.
    /// offset: 0x47
    mesh_index: u8,

    /// The volume of the katamari (in m^3).
    /// offset: 0x50
    vol_m3: f32,

    /// The maximum prop volume that can be collected (in m^3).
    /// offset: 0x54
    max_pickup_vol_m3: f32,

    /// The exact diameter of the katamari (in cm).
    /// offset: 0x5c
    diam_cm: f32,

    /// The truncated diameter of the katamari (in mm).
    /// offset: 0x60
    diam_trunc_mm: i32,

    /// The initial exact diameter of the katamari (in cm).
    /// offset: 0x64
    init_diam_cm: f32,

    /// The radius of the katamari (in cm).
    /// offset: 0x68
    radius_cm: f32,

    /// The visual radius of the katamari "ball" (in cm).
    /// offset: 0x70
    display_rad_cm: f32,

    /// The circumference of the katamari (in cm).
    /// offset: 0x74
    circumf_cm: f32,

    /// The speed of the katamari on the current tick.
    /// offset: 0x78
    speed: f32,

    /// The speed of the katamari on the previous tick.
    /// offset: 0x7c
    last_speed: f32,

    /// The alpha of attached props.
    /// offset: 0xa0
    attached_prop_alpha: f32,

    /// Various physics-related flags (some of which aren't actually true/false values, but whatever).
    /// offset: 0xa4
    physics_flags: KatPhysicsFlags,

    /// The value of `physics_flags` on the previous tick.
    /// offset: 0xc7
    last_physics_flags: KatPhysicsFlags,

    /// Various flags which relate to properties of surfaces the katamari currently contacts.
    /// offset: 0xea
    hit_flags: KatHitFlags,

    /// The value of `hit_flags` on the previous two ticks.
    /// Index 0 is the previous tick, and index 1 is two ticks ago.
    /// offset: 0xf5
    last_hit_flags: [KatHitFlags; 2],

    /// (??)
    /// offset: 0x10d
    brake_push_dir: KatPushDir,

    /// (??)
    /// offset: 0x10e
    input_push_dir: KatPushDir,

    /// The number of ticks the katamari has been airborne.
    /// Resets to 0 each time the katamari starts being airborne.
    /// offset: 0x114
    ticks_airborne: u16,

    /// The number of ticks the katamari has been falling.
    /// Resets to 0 each time the katamari starts falling.
    /// offset: 0x116
    ticks_falling: u16,

    /// counts down from 10 after falling from a climb; if still nonzero, can't climb again    
    /// offset: 0x118
    wallclimb_cooldown: u16,

    /// Katamari velocities.
    /// offset: 0x240
    velocity: KatVelocity,

    /// Katamari velocities on the previous tick.
    /// offset: 0x310
    last_velocity: KatVelocity,

    /// The diameter used to compute size-based scaling params (in cm).
    /// offset: 0x3e0
    scaled_params_size: f32,

    /// Katamari params which vary continuously based on the katamari's current size.
    /// offset: 0x3e4
    scaled_params: KatScaledParams,

    /// (??) The unit vector that's pointing "rightwards" relative to the katamari's lateral velocity.
    /// offset: 0x440
    u_right_of_vel: Vec4,

    /// (??)
    /// offset: 0x450
    camera_side_vector: Vec4,

    /// The center point of the katamari on the current tick.
    /// offset: 0x460
    center: Vec4,

    /// The center point of the katamari on the previous tick.
    /// offset: 0x470
    last_center: Vec4,

    /// (??) The vector moved the previous tick.
    /// offset: 0x480
    delta_pos: Vec4,

    /// (??) The pitch rotation component of the transform
    /// offset: 0x520
    pitch_rotation_mat: Mat4,

    /// The rotation component of the transform (with translation zeroed out)
    /// offset: 0x5a0
    rotation_mat: Mat4,

    /// (??) The extra rotation induced by being on a spinning turntable
    /// offset: 0x5e0
    turntable_rotation_mat: Mat4,

    /// (??) Extra flat velocity??
    /// offset: 0x660
    bonus_vel: Vec4,

    /// The top point of the katamari sphere.
    /// offset: 0x680
    top: Vec4,

    /// The bottom point of the katamari sphere.
    /// offset: 0x690
    bottom: Vec4,

    /// The `TopCenter` shell point.
    /// offset: 0x6a0
    shell_top_center: Vec4,

    /// The `Bottom` shell point.
    /// offset: 0x6b0
    shell_bottom: Vec4,

    /// The `Left` shell point.
    /// offset: 0x6c0
    shell_left: Vec4,

    /// The `Right` shell point.
    /// offset: 0x6d0
    shell_right: Vec4,

    /// The `TopLeft` shell point.
    /// offset: 0x6e0
    shell_top_left: Vec4,

    /// The `TopRight` shell point.
    /// offset: 0x6f0
    shell_top_right: Vec4,

    /// The katamari's transform matrix.
    /// offset: 0x710
    transform: Mat4,

    /// The katamari's radius when it started climbing.
    /// offset: 0x768
    wallclimb_init_rad: f32,

    /// The distance moved upward each tick during a wall climb.
    /// offset: 0x76c
    wallclimb_speed: f32,

    /// The initial y position of the katamari when it started a wall climb.
    /// offset: 0x770
    wallclimb_init_y: f32,

    /// The unit normal of the wall being climbed.
    /// offset: 0x774
    wallclimb_wall_unorm: Vec4,

    /// The number of ticks since the katamari started climbing a wall.
    /// offset: 0x784
    wallclimb_ticks: u16,

    /// The number of ticks the katamari can gain height while climbing a wall
    /// offset: 0x786
    wallclimb_max_height_ticks: u16,

    /// The number of ticks since the katamari started moving downhill.
    /// offset: 0x788
    move_downhill_ticks: u16,

    /// The number of ticks since the katamari started moving uphill.
    /// offset: 0x78a
    move_uphill_ticks: u16,

    /// The unit normal of the active contact floor, if one exists.
    /// offset: 0x78c
    contact_floor_normal_unit: Vec4,

    /// The unit normal of the active contact wall, if one exists.
    /// offset: 0x78c
    u_contact_wall_normal: Vec4,

    /// The length of the collision ray contacting the floor.
    /// offset: 0x7fc
    contact_floor_ray_len: f32,

    /// A multiplier affecting how fast pivoted props are sucked in towards the center
    /// of the katamari (which also reduces the length of their induced collision ray).
    /// offset: 0x800
    pivot_prop_decay_mult: f32,

    /// The number of floors contacted by collision rays.
    /// offset: 0x804
    num_floor_contacts: u16,

    /// The number of walls contacted by collision rays.
    /// offset: 0x806
    num_wall_contacts: u16,

    /// The number of floors contacted by collision rays on the previous tick.
    /// offset: 0x808
    last_num_floor_contacts: u16,

    /// The number of walls contacted by collision rays on the previous tick.
    /// offset: 0x80a
    last_num_wall_contacts: u16,

    /// When the katamari is underwater, the point on the water surface that's directly
    /// above the katamari center.
    /// offset: 0x85c
    water_surface_hit: Vec4,

    /// (??) The point on a surface directly below the katamari where the shadow should be drawn.
    /// offset: 0x86c
    shadow_pos: Vec4,

    /// (??) The number of ticks the katamari has been stuck between walls.
    /// offset: 0x87c
    stuck_ticks: u32,

    /// (??)
    /// offset: 0x880
    prop_interaction_iframes: i32,

    /// The (real-time) game time when the last collision occurred.
    /// offset: 0x884
    last_collision_game_time_ms: i32,

    /// (??) The prop which is colliding with the katamari. (why are there two such props in ghidra)
    /// offset: 0x888
    contact_prop: Option<PropRef>,

    /// (??) this might be the cooldown on the "struggle" VFX that plays when almost at max climb height
    /// offset: 0x898
    is_climbing: u16,

    /// The cooldown period for the "ripple" VFX that plays continuously while the katamari is in water.
    /// offset: 0x89a
    water_ripple_vfx_timer: u16,

    /// The cooldown period for the "splash" VFX when the katamari enters water.
    /// offset: 0x89c
    water_splash_vfx_timer: u16,

    /// The cooldown period for the "splash" SFX that plays continuously while the katamari is in water.
    /// offset: 0x89e
    water_sfx_timer: u16,

    /// The katamari's current collision rays.
    /// offset: 0x8d0
    collision_rays: KatCollisionRays,

    /// The katamari's collision rays on the previous tick.
    /// offset: 0x20d0
    last_collision_rays: KatCollisionRays,

    /// The number of mesh collision rays. (The default mesh has 18.)
    /// offset: 0x38d0
    num_mesh_rays: u16,

    /// The maximum number of prop-induced collision rays. (The default value is 12.)
    max_prop_rays: u16,

    /// The katamari's transform at the beginning of a vault.
    /// offset: 0x38d4
    vault_init_transform: Mat4,

    /// (??) Some kind of transform for when the katamari is vaulting
    /// offset: 0x3914
    vault_transform_unknown: Mat4,

    /// (??) The vector from the katamari center to the vault contact point.
    /// offset: 0x3954
    vault_contact_ray: Vec4,

    /// (??) The current vault contact point.
    /// offset: 0x3964
    vault_contact_point: Vec4,

    /// The index of the collision ray being used to vault, if one exists.
    vault_ray_idx: Option<u16>,

    /// (??) The maximum allowed length of any collision ray.
    /// offset: 0x39ac
    max_allowed_ray_len: f32,

    /// The average length of all collision rays.
    /// offset: 0x39b0
    average_ray_len: f32,

    /// The number of ticks the katamari since the katamari started its current vault.
    /// offset: 0x39bc
    vault_time_ticks: u32,

    /// The collision ray index of the first prop ray.
    /// offset: 0x39c0
    first_prop_ray_index: u16,

    /// If true, collision rays induced by props are allowed (which is the default behavior).
    /// offset: 0x38c2
    enable_prop_rays: bool,

    /// The collision ray vector
    vault_ray_vec: Vec4,

    /// The first prop that was attached to the katamari.
    /// offset: 0x39d8
    first_attached_prop: Option<PropRef>,

    /// The last prop that was attached to the katamari.
    /// offset: 0x39e0
    last_attached_prop: Option<PropRef>,

    /// The name index of the last attached prop.
    /// offset: 0x39e8
    last_attached_prop_name_idx: u16,

    /// The penalty multiplier that will be applied to a prop when it is attached.
    /// offset: 0x3a70
    attach_vol_penalty: f32,

    /// (??) The number of props collected in the current combo
    /// offset: 0x3a78
    prop_combo_count: u32,

    /// The extra rotation matrix applied to the katamari's transform when spinning.
    /// offset: 0x3a84
    spin_rotation_mat: Mat4,

    /// (??) vs mode value. returned from `KataVsGet_CatchCount` API function.
    /// offset: 0x3ad4
    pub vs_catch_count: i16,

    /// (??) vs mode value. returned from `KataVsGet_AttackCount` API function.
    /// offset: 0x3ad6
    pub vs_attack_count: i16,

    /// True if the player is in the "look L1" state.
    /// offset: 0x3b38
    is_look_l1: bool,

    /// Whether or not the "something's coming" alarm is going off.
    /// offset: 0x3b84
    alarm_mode: bool,

    /// The "urgency" of the "something's coming" alarm, if the alarm is active.
    /// offset: 0x3b86
    alarm_type: Option<AlarmType>,
}

impl Katamari {
    pub fn get_init_radius(&self) -> f32 {
        self.init_diam_cm / 2.0
    }

    pub fn get_radius(&self) -> f32 {
        self.radius_cm
    }

    pub fn get_display_radius(&self) -> f32 {
        self.display_rad_cm
    }

    pub fn get_diam_int(&self) -> i32 {
        self.diam_trunc_mm
    }

    pub fn get_vol(&self) -> f32 {
        self.vol_m3
    }

    pub fn get_prince_offset(&self) -> f32 {
        self.scaled_params.prince_offset
    }

    pub fn get_center(&self) -> &Vec4 {
        &self.center
    }

    pub fn set_look_l1(&mut self, is_look_l1: bool) {
        self.is_look_l1 = is_look_l1;
    }

    pub fn get_translation(
        &self,
        x: &mut f32,
        y: &mut f32,
        z: &mut f32,
        sx: &mut f32,
        sy: &mut f32,
        sz: &mut f32,
    ) -> () {
        // sort of hacky to read the translation directly out of the matrix but whatever.
        // the builtin `mat4::get_translation` writes the values to a `Vec3` instead of individual floats.
        // (see: https://docs.rs/gl_matrix/latest/src/gl_matrix/mat4.rs.html#1030-1036)
        *x = self.transform[12];
        *y = self.transform[13];
        *z = self.transform[14];

        *sx = self.shadow_pos[0];
        *sy = self.shadow_pos[1];
        *sz = self.shadow_pos[2];
    }

    pub fn set_translation(&mut self, x: f32, y: f32, z: f32) {
        let K = UNITY_TO_SIM_SCALE;
        let trans = [x * K, y * K, z * K, 1.0];

        // set the center and last center points
        vec4::copy(&mut self.center, &trans);
        vec4::copy(&mut self.last_center, &trans);

        // set the translation component of the transform matrix
        self.transform[TRANSFORM_X_POS] = trans[0];
        self.transform[TRANSFORM_Y_POS] = trans[1];
        self.transform[TRANSFORM_Z_POS] = trans[2];
    }

    pub fn get_matrix(
        &self,
        xx: &mut f32,
        xy: &mut f32,
        xz: &mut f32,
        yx: &mut f32,
        yy: &mut f32,
        yz: &mut f32,
        zx: &mut f32,
        zy: &mut f32,
        zz: &mut f32,
    ) -> () {
        *xx = self.transform[0];
        *xy = self.transform[1];
        *xz = self.transform[2];
        *yx = self.transform[4];
        *yy = self.transform[5];
        *yz = self.transform[6];
        *zx = self.transform[8];
        *zy = self.transform[9];
        *zz = self.transform[10];
    }

    pub fn get_alarm(&self, alarm_mode: &mut i32, alarm_type: &mut i32) {
        *alarm_mode = self.alarm_mode as i32;
        *alarm_type = self.alarm_type.unwrap_or(AlarmType::Closest) as i32;
    }

    pub fn is_in_water(&self) -> bool {
        self.physics_flags.in_water
    }
}

impl Katamari {
    ///
    pub fn init(
        &mut self,
        player: u8,
        init_diam: f32,
        init_pos: &Vec4,
        sim_params: &SimulationParams,
    ) {
        // extra stuff not in the original simulation
        self.max_prop_rays = sim_params.kat_max_prop_collision_rays;
        // end extra stuff

        self.player = player;
        self.mesh_index = 1;
        self.input_push_dir = KatPushDir::Forwards;

        // TODO: `kat_init:36-45`

        self.physics_flags = KatPhysicsFlags::default();
        self.hit_flags = KatHitFlags::default();

        self.attached_prop_alpha = sim_params.prop_attached_alpha;

        self.diam_cm = init_diam;
        self.init_diam_cm = init_diam;
        self.radius_cm = init_diam / 2.0;
        self.diam_trunc_mm = (init_diam * 10.0) as i32;

        self.last_velocity = self.velocity;

        vec4::copy(&mut self.center, &init_pos);

        vec4::copy(&mut self.bottom, &self.center);
        self.bottom[1] -= self.radius_cm;

        self.contact_floor_ray_len = self.radius_cm;

        let rad_m = self.radius_cm / 100.0;
        self.vol_m3 = rad_m * rad_m * rad_m * _4PI_3;

        vec4::copy(&mut self.u_right_of_vel, &VEC4_X_NEG);
        mat4::identity(&mut self.transform);
        mat4::identity(&mut self.turntable_rotation_mat);
        mat4::identity(&mut self.rotation_mat);
        mat4::identity(&mut self.pitch_rotation_mat);
        mat4::identity(&mut self.spin_rotation_mat);
        vec4::copy(&mut self.bonus_vel, &VEC4_ZERO);

        // TODO: `kat_init:148-152` (zeroing out surface contact history; continues beyond line 152).

        vec4::copy(&mut self.contact_floor_normal_unit, &VEC4_Y_POS);

        self.first_attached_prop = None;
        self.last_attached_prop = None;
        self.enable_prop_rays = true;
        self.last_attached_prop_name_idx = 0;

        self.reset_collision_rays();

        self.prop_interaction_iframes = 0;

        // TODO: `kat_init:181-237` (camera initialization using static mission/area params table)

        self.set_immobile();

        // TODO: `kat_update_size_scaled_params()`
        // TODO: `kat_init:253`

        self.prop_combo_count = 0;
        self.physics_flags.wheel_spin = false;
        self.last_collision_game_time_ms = 0;

        // TODO: `kat_init:270-275` (prop combo initialization)

        self.is_climbing = 0;
        if self.physics_flags.climbing_wall {
            self.wallclimb_ticks = 0;
            self.wallclimb_cooldown = sim_params.kat_init_wallclimb_cooldown;
        }

        self.physics_flags.climbing_wall = false;
        self.physics_flags.at_max_climb_height = false;
        self.wallclimb_init_y = 0.0;
        self.wallclimb_max_height_ticks = 0;

        // TODO: `kat_init:284-285` (not sure what this is about)
        // TODO: `kat_init:286-288` (compute initial airborne prop gravity)
    }

    /// Forcibly end the katamari's movement, if it's moving.
    /// offset: 0x1f390
    pub fn set_immobile(&mut self) {
        // TODO
    }
}
