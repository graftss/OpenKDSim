pub mod collision;
mod flags;
mod params;

use gl_matrix::{
    common::{Mat4, Vec3},
    mat4, vec3,
};

use crate::{
    collision::raycast_state::RaycastState,
    constants::{
        FRAC_4PI_3, TRANSFORM_X_POS, TRANSFORM_Y_POS, TRANSFORM_Z_POS, UNITY_TO_SIM_SCALE,
        VEC3_X_NEG, VEC3_Y_POS, VEC3_ZERO,
    },
    macros::min,
    math::normalize_bounded_angle,
    mission::state::MissionState,
    props::prop::PropRef,
};

use self::{
    collision::mesh::KatMesh,
    collision::{hit::KatamariHit, ray::KatCollisionRays},
    flags::{KatHitFlags, KatPhysicsFlags},
    params::KatamariParams,
};

use super::{
    camera::Camera,
    prince::{Prince, PrincePushDir},
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

/// (??) not sure about this
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmType {
    Closest,
    Closer,
    Close,
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

/// Katamari velocity and acceleration values.
#[derive(Debug, Default, Copy, Clone)]
pub struct KatVelocity {
    /// Current velocity
    /// offset: 0x0
    pub velocity: Vec3,

    /// Current unit velocity
    /// offset: 0x10
    pub velocity_unit: Vec3,

    /// (??)
    /// offset: 0x20
    pub speed_orth_on_floor: Vec3,

    /// (??)
    /// offset: 0x30
    pub speed_proj_on_floor: Vec3,

    /// (??)
    /// offset: 0x40
    pub last_vel_accel: Vec3,

    /// (??) Velocity + player acceleration
    /// offset: 0x50
    pub vel_accel: Vec3,

    /// Unit vector of `vel_accel`.
    /// offset: 0x60
    pub vel_accel_unit: Vec3,

    /// (??) Velocity + player accel + gravity
    /// offset: 0x70
    pub vel_accel_grav: Vec3,

    /// Unit vector of `vel_accel_grav`.
    /// offset: 0x80
    pub vel_accel_grav_unit: Vec3,

    /// (??)
    /// offset: 0x90
    pub push_vel_on_floor_unit: Vec3,

    /// Acceleration from gravity
    /// offset: 0xa0
    pub accel_gravity: Vec3,

    /// Acceleration from the contacted floor incline
    /// offset: 0xb0
    pub accel_incline: Vec3,

    /// (??) Acceleration from the contacted floor friction (or some kind of similar force)
    /// offset: 0xc0
    pub accel_ground_friction: Vec3,
}

#[derive(Debug, Default)]
pub struct Katamari {
    // BEGIN new fields (that were not part of the original simulation's `Katamari` struct.)
    /// Parameters that affect katamari movement. In the original simulation these were
    /// mostly static constants.
    params: KatamariParams,

    /// A static flag to tell to turn on the "map semi translucent" hit flag.
    /// offset: 0x10eacd
    has_map_semi_translucent_hit: bool,

    /// A static timer which affects prop collisions when >0.
    /// offset: 0x10eae0
    prop_pseudo_iframes_timer: u8,

    /// A list of all currently hit floor surfaces.
    /// offset: 0x1959f0
    hit_floors: Vec<KatamariHit>,

    /// (??) A list of all hit floor surfaces on the previous tick.
    /// Seems like it's unused?
    /// offset: 0x1941f0
    last_hit_floors: Vec<KatamariHit>,

    /// A list of all hit wall surfaces.
    hit_walls: Vec<KatamariHit>,

    /// TODO
    raycast_state: RaycastState,

    /// The number of props attached to the katamari (including unloaded ones).
    /// offset: 0x133a08
    num_attached_props: u16,

    // END new fields
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
    max_attach_vol_m3: f32,

    /// If colliding with a prop that can use its AABB for collisions,
    /// its AABB can only be used if its (compare) volume is less than this value.
    /// offset: 0x58
    use_prop_aabb_collision_vol: f32,

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

    /// The distance from the camera position to the katamari center.
    /// offset: 0x98
    dist_to_cam: f32,

    /// The alpha of attached props.
    /// offset: 0xa0
    attached_prop_alpha: f32,

    /// Various physics-related flags (some of which aren't actually true/false values, but whatever).
    /// offset: 0xa4
    pub physics_flags: KatPhysicsFlags,

    /// The value of `physics_flags` on the previous tick.
    /// offset: 0xc7
    last_physics_flags: KatPhysicsFlags,

    /// Various flags which relate to properties of surfaces the katamari currently contacts.
    /// offset: 0xea
    pub hit_flags: KatHitFlags,

    /// The value of `hit_flags` on the previous two ticks.
    /// Index 0 is the previous tick, and index 1 is two ticks ago.
    /// offset: 0xf5
    pub last_hit_flags: [KatHitFlags; 2],

    /// (??)
    /// offset: 0x10d
    brake_push_dir: KatPushDir,

    /// (??)
    /// offset: 0x10e
    input_push_dir: KatPushDir,

    /// The number of ticks the katamari has been airborne.
    /// Resets to 0 each time the katamari starts being airborne.
    /// offset: 0x114
    airborne_ticks: u16,

    /// The number of ticks the katamari has been falling.
    /// Resets to 0 each time the katamari starts falling.
    /// offset: 0x116
    ticks_falling: u16,

    /// counts down from 10 after falling from a climb; if still nonzero, can't climb again    
    /// offset: 0x118
    wallclimb_cooldown: u16,

    /// "Bounciness" or elasticity multiplier, which is a linear function of
    /// the katamari's diameter. The linear can be different depending on the stage.
    /// offset: 0x19c
    elasticity: f32,

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
    u_right_of_vel: Vec3,

    /// (??)
    /// offset: 0x450
    camera_side_vector: Vec3,

    /// The center point of the katamari on the current tick.
    /// offset: 0x460
    center: Vec3,

    /// The center point of the katamari on the previous tick.
    /// offset: 0x470
    last_center: Vec3,

    /// (??) The vector moved the previous tick.
    /// offset: 0x480
    delta_pos: Vec3,

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
    bonus_vel: Vec3,

    /// The top point of the katamari sphere.
    /// offset: 0x680
    top: Vec3,

    /// The bottom point of the katamari sphere.
    /// offset: 0x690
    bottom: Vec3,

    /// The `TopCenter` shell point.
    /// offset: 0x6a0
    shell_top_center: Vec3,

    /// The `Bottom` shell point.
    /// offset: 0x6b0
    shell_bottom: Vec3,

    /// The `Left` shell point.
    /// offset: 0x6c0
    shell_left: Vec3,

    /// The `Right` shell point.
    /// offset: 0x6d0
    shell_right: Vec3,

    /// The `TopLeft` shell point.
    /// offset: 0x6e0
    shell_top_left: Vec3,

    /// The `TopRight` shell point.
    /// offset: 0x6f0
    shell_top_right: Vec3,

    /// The katamari's transform matrix.
    /// offset: 0x710
    transform: Mat4,

    /// The katamari's "max" forward speed
    /// offset: 0x750
    max_forwards_speed: f32,

    /// The katamari's "max" backwards speed
    /// offset: 0x754
    max_backwards_speed: f32,

    /// The katamari's "sideways" forward speed
    /// offset: 0x758
    max_sideways_speed: f32,

    /// The katamari's "boost" forward speed
    /// offset: 0x75c
    max_boost_speed: f32,

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
    wallclimb_wall_unorm: Vec3,

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
    contact_floor_normal_unit: Vec3,

    /// The unit normal of the active contact wall, if one exists.
    /// offset: 0x78c
    u_contact_wall_normal: Vec3,

    /// (??) The length of the primary collision ray contacting the floor.
    /// offset: 0x7fc
    fc_ray_len: f32,

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
    water_surface_hit: Vec3,

    /// (??) The point on a surface directly below the katamari where the shadow should be drawn.
    /// offset: 0x86c
    shadow_pos: Vec3,

    /// (??) The number of ticks the katamari has been stuck between walls.
    /// offset: 0x87c
    stuck_ticks: u32,

    /// (??)
    /// offset: 0x880
    prop_ignore_collision_timer: u32,

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

    /// (??) The lowest y coordinate of all current wall contact points.
    /// offset: 0x8a0
    lowest_wall_contact_y: f32,

    /// (??) The lowest y coordinate of all current floor contact points.
    /// offset: 0x8a4
    lowest_floor_contact_y: f32,

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

    /// (??) The vector from the katamari center to the primary floor contact.
    /// offset: 0x3954
    fc_contact_ray: Vec3,

    /// (??) The contact point of the primary floor contact.
    fc_contact_point: Vec3,

    /// (??) The current vault contact point.
    /// offset: 0x3974
    vault_contact_point: Vec3,

    /// (??) The unit normal vector of the floor used for the current vault.
    /// offset: 0x3984
    vault_floor_normal_unit: Vec3,

    /// (??) The index of the collision ray being used to vault, if one exists.
    /// offset: 0x3998
    vault_ray_idx: Option<u16>,

    /// (??) The index of the collision ray used as the primary floor contact, if one exists.
    /// offset: 0x399c
    fc_ray_idx: Option<u16>,

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
    vault_ray_vec: Vec3,

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

    /// (??) Initial speed after a boost.
    /// offset: 0x3a7c
    boost_speed: f32,

    /// Number of ticks the katamari has been spinning.
    /// offset: 0x3a80
    spin_ticks: u32,

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

    pub fn get_diam_cm(&self) -> f32 {
        self.diam_cm
    }

    pub fn get_diam_m(&self) -> f32 {
        self.diam_cm * 100.0
    }

    pub fn get_vol(&self) -> f32 {
        self.vol_m3
    }

    pub fn get_prince_offset(&self) -> f32 {
        self.scaled_params.prince_offset
    }

    pub fn get_center(&self) -> &Vec3 {
        &self.center
    }

    pub fn set_center(&mut self, center: &Vec3) {
        vec3::copy(&mut self.center, center);
    }

    pub fn get_speed(&self) -> f32 {
        self.speed
    }

    /// Computes the ratio of the katamari's current speed to its "max" speed,
    /// which varies with the prince's push direction.
    pub fn get_speed_ratio(&self, push_dir: PrincePushDir) -> f32 {
        match push_dir {
            PrincePushDir::Forwards => self.speed / self.max_forwards_speed,
            PrincePushDir::Backwards => self.speed / self.max_backwards_speed,
            PrincePushDir::Sideways => self.speed / self.max_sideways_speed,
        }
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
        let trans = [x * K, y * K, z * K];

        // set the center and last center points
        vec3::copy(&mut self.center, &trans);
        vec3::copy(&mut self.last_center, &trans);

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

    pub fn update_royal_warp(&mut self, dest_pos: &Vec3) {
        self.set_center(dest_pos);
        self.reset_collision_rays();
        self.set_immobile();
        self.airborne_ticks = 0;
    }
}

impl Katamari {
    pub fn init(&mut self, player: u8, init_diam: f32, init_pos: &Vec3) {
        // extra stuff not in the original simulation
        self.max_prop_rays = self.params.max_prop_collision_rays;
        // end extra stuff

        self.player = player;
        self.mesh_index = 1;
        self.input_push_dir = KatPushDir::Forwards;

        // TODO: `kat_init:36-45`

        self.physics_flags = KatPhysicsFlags::default();
        self.hit_flags = KatHitFlags::default();

        self.attached_prop_alpha = self.params.prop_attached_alpha;

        // update sizes
        self.diam_cm = init_diam;
        self.init_diam_cm = init_diam;
        self.radius_cm = init_diam / 2.0;
        self.diam_trunc_mm = (init_diam * 10.0) as i32;

        self.last_velocity = self.velocity;

        vec3::copy(&mut self.center, &init_pos);

        vec3::copy(&mut self.bottom, &self.center);
        self.bottom[1] -= self.radius_cm;

        self.fc_ray_len = self.radius_cm;

        let rad_m = self.radius_cm / 100.0;
        self.vol_m3 = rad_m * rad_m * rad_m * FRAC_4PI_3;

        vec3::copy(&mut self.u_right_of_vel, &VEC3_X_NEG);
        mat4::identity(&mut self.transform);
        mat4::identity(&mut self.turntable_rotation_mat);
        mat4::identity(&mut self.rotation_mat);
        mat4::identity(&mut self.pitch_rotation_mat);
        mat4::identity(&mut self.spin_rotation_mat);
        vec3::copy(&mut self.bonus_vel, &VEC3_ZERO);

        // TODO: `kat_init:148-152` (zeroing out surface contact history; continues beyond line 152).

        vec3::copy(&mut self.contact_floor_normal_unit, &VEC3_Y_POS);

        self.first_attached_prop = None;
        self.last_attached_prop = None;
        self.enable_prop_rays = true;
        self.last_attached_prop_name_idx = 0;

        self.reset_collision_rays();

        self.prop_ignore_collision_timer = 0;

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
            self.wallclimb_cooldown = self.params.init_wallclimb_cooldown;
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

impl Katamari {
    /// Main katamari update function.
    /// offset: 0x1db50
    pub fn update(&mut self, prince: &Prince, camera: &Camera, mission_state: &MissionState) {
        let stage_config = mission_state.stage_config.as_ref().unwrap();
        let mission_config = mission_state.mission_config.as_ref().unwrap();

        // decrement timers
        self.wallclimb_cooldown -= 1;

        // record the previous values of various fields
        self.last_center = self.center;
        self.last_velocity = self.velocity;
        self.last_physics_flags = self.physics_flags;
        self.last_hit_flags[1] = self.last_hit_flags[0];
        self.last_hit_flags[0] = self.hit_flags;

        if self.num_floor_contacts > 0 {
            // if the katamari has a ground contact, update the `last_hit_flags`
            self.last_hit_flags[0] = self.hit_flags
        }

        let oujistate = prince.get_oujistate();
        self.physics_flags.wheel_spin = oujistate.wheel_spin;

        // TODO: `kat_update_incline_accel()`

        self.elasticity = stage_config.get_kat_elasticity(self.diam_cm);

        if !oujistate.wheel_spin && !oujistate.dash_start {
            // if the katamari is not spinning:
            self.boost_speed = 0.0;
            self.spin_ticks = 0;
            mat4::identity(&mut self.spin_rotation_mat);
        } else {
            // if the katamari is spinning:
            self.spin_ticks += 1;
            self.boost_speed = min!(
                self.boost_speed + self.scaled_params.boost_accel,
                self.scaled_params.base_max_speed * self.scaled_params.max_boost_speed
            );
        };

        let spin_rotation = normalize_bounded_angle(self.boost_speed / self.radius_cm);
        mat4::from_rotation(
            &mut self.spin_rotation_mat,
            spin_rotation,
            &self.camera_side_vector,
        );

        // TODO: self.update_velocity()
        // TODO: self.update_friction()
        // TODO: self.apply_acceleration()

        let cam_transform = camera.get_transform();
        let left = VEC3_X_NEG;
        vec3::transform_mat4(
            &mut self.camera_side_vector,
            &left,
            &cam_transform.cam_forward_yaw_rot,
        );

        self.update_collision_rays();
        // TODO: self.attract_props_to_center();

        self.attach_vol_penalty = mission_config.get_vol_penalty(self.diam_cm);
        self.update_collision(&mission_state);

        // compute distance to camera
        let mut dist_to_cam = vec3::create();
        vec3::subtract(&mut dist_to_cam, &self.center, &cam_transform.pos);
        self.dist_to_cam = vec3::len(&dist_to_cam);

        // TODO: self.update_vel_relative_to_cam()

        // TODO: `kat_update:390-415` (self.update_dust_cloud_vfx())
        // TODO: `kat_update:416-447` (self.update_prop_combo())

        if !camera.preclear.get_enabled() {
            // TODO: `kat_update:499-512` (update `camera_focus_position`, which seems to be unused)
        }
    }
}
