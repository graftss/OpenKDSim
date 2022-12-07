use crate::{constants::FRAC_PI_2, player::prince::PushDir};

#[derive(Debug)]
pub struct KatamariParams {
    /// The number of ticks where the katamari can't start a second climb after falling out
    /// of a first climb.
    pub init_wallclimb_cooldown_timer: u16,

    /// The maximum number of collision rays that can be induced by props.
    pub max_prop_collision_rays: u16,

    /// The alpha of props which are attached to the katamari.
    pub prop_attached_alpha: f32,

    /// The multiple of the katamari's volume to use as the
    /// `prop_use_aabb_collision_vol` threshold
    /// offset: 0x7b110
    pub prop_use_aabb_collision_vol_ratio: f32,

    /// The fraction of the katamari's volume that can be attached.
    /// (e.g. a value of 0.1 means the katamari can attach props 10% as big as it)
    /// offset: 0x7b220
    pub prop_attach_vol_ratio: f32,

    /// (??) The minimum angle between input and katamari velocity needed to brake, or something
    /// offset: 0x7b238 (but this value is `cos(thatvalue * pi/2)` because reasons)
    pub min_brake_angle: f32,

    /// (??)
    /// offset: 0x10eb1c (but this value is `cos(thatvalue * pi/2)` because reasons)
    pub max_wallclimb_angle: f32,

    /// (??) something about how the katamari collision rays can clip into surfaces
    /// offset: 0x7153c
    pub clip_len_constant: f32,

    /// The threshold on the y component of a floor normal that distinguishes
    /// flat floors (over this value) to sloped floors (under this value).
    /// (Note that a y component of 1 would be a completely flat floor)
    /// offset: 0x71608
    pub sloped_floor_normal_y_threshold: f32,

    /// (??) If two surfaces have normal vectors which dot to above this value, they're not distinguished
    /// by the collision system while the katamari contacts both of them.
    /// offset: 0x7160c
    pub surface_similarity_threshold: f32,

    /// If: - the katamari contacts a wall and a floor,
    ///     - the angle between those contacted surface normals is bigger than this value, and
    ///     - the katamari is moving
    /// then the katamari is considered stuck.
    /// default: 3pi/2
    /// offset: 0x716d4
    pub wall_to_floor_angle_stuck_threshold: f32,

    /// If: - the katamari contacts exactly 2 walls
    ///     - the angle between the walls' normals is bigger than this value,
    /// then the katamari is considered stuck.
    /// default: 5pi/6
    /// offset: 0x716f0
    pub wall_to_wall_angle_stuck_threshold: f32,

    /// When stuck between walls, the katamari will continuously detach props
    /// after this many ticks pass.
    pub detach_cooldown_when_stuck_btwn_walls: u8,

    /// The "baseline" ratio of the katamari's volume that's detached
    /// as prop volume, whenever props are detached. Depending on the source
    /// of the detachment, the volume will be multiplied scaled further.
    /// default: 0.03
    /// offset: 0x7b224
    pub base_detached_prop_vol_mult: f32,

    /// The multiplier to detached prop volume when props are detached while
    /// the katamari is stuck between walls.
    /// offset: 0x715bc (used at 0x17742)
    pub stuck_detached_prop_vol_mult: f32,

    /// A multiplier on the rate at which vaulted props decay towards the center.
    /// default: 0.015
    /// offset: 0x7b218
    pub vault_prop_pull_to_center_mult: f32,

    /// For some reason when they convert the katamari's volume to its radius,
    /// they also add 0.01 to the radius. What the hell were they thinking?
    /// offset: 0x7155c (used at 0x1eec2)
    pub radius_boost_cm: f32,

    /// The ratio between the display radius (aka the radius of the actual katamari model)
    /// and the "true" katamari radius computed from its volume.
    /// default: 0.38
    /// offset: 0x7b234
    pub display_radius_ratio: f32,

    /// The ratio of the katamari's diameter that it's able to climb up a wall.
    /// default: 0.7
    /// offset: 0x719f8
    pub max_wallclimb_height_ratio: f32,

    /// (??)
    /// default: 0.95
    /// offset: 0x719fc
    pub wallclimb_speed_penalty: f32,

    /// The number of ticks that the katamari can be in the `KatBoostEffectState::Build` state.
    pub boost_build_duration: u16,

    /// The number of ticks that the katamari can be in the `KatBoostEffectState::Release` state.
    pub boost_release_duration: u16,

    /// The number of ticks that the katamari can be in the `KatBoostEffectState::Release` state
    /// while it's in water.
    pub boost_release_duration_in_water: u16,

    /// Multiplier to katamari speed when pushing forwards.
    /// default: 1.0
    /// offset: 0x7b0ec
    pub forwards_speed_mult: f32,

    /// Multiplier to katamari speed when pushing backwards.
    /// default: 1.0
    /// offset: 0x7b124
    pub sideways_speed_mult: f32,

    /// Multiplier to katamari speed when pushing sideways.
    /// default: 1.0
    /// offset: 0x7b128
    pub backwards_speed_mult: f32,

    /// Multiplier to katamari speed when boosting.
    /// default: 1.0
    /// offset: 0x7a25c
    pub boost_speed_mult: f32,

    /// The katamari can only brake if it's moving at least this ratio of its max speed.
    /// This stops the katamari from triggering a brake when it's moving very slowly.
    /// default: 0.45
    /// offset: 0x7b250
    pub brakeable_max_speed_ratio: f32,

    /// The number of ticks between brake vfx plays (while braking).
    /// default: 10
    pub brake_vfx_cooldown: u8,

    /// (??) used in `update_friction`
    /// default: 0.01
    /// offset: 0x7155c (used at 0x21645)
    pub min_speed_to_move: f32,

    /// The value that scales the acceleration effect of friction when grounded
    /// by katamari's bottom collision ray.
    /// default: 0.0352
    /// offset: 0x7b200
    pub bottom_ray_friction: f32,

    /// A value that scales the acceleration effect of friction when grounded
    /// by one of the katamari's mesh or prop collision rays.
    /// default: 0.005
    /// offset: 0x7b214
    pub nonbottom_ray_friction: f32,

    /// The factor by which friction is reduced when moving downhill on a `SpeedCheckOff` surface.
    /// default: 0.55
    /// offset: 0x715c8
    pub speed_check_off_friction_reduction: f32,

    /// If the dot product of the katamari's velocity and a contact wall's normal is below this
    /// value, the katamari is considered to be moving "towards" the wall.
    /// default: 0.01
    /// offset: 0x7155c (used at 0x18549)
    pub move_into_wall_similarity: f32,

    /// The proportion of the katamari's max speed that it's bumped by when stuck between walls,
    /// in an attempt to get it unstuck.
    /// default: 0.05
    /// offset: 0x71578 (used at 0x185ab)
    pub unstuck_bump_speed: f32,

    /// The minimum y normal distinguishing a wall from a floor.
    /// default: TODO
    /// offset: TODO
    pub surface_normal_y_threshold: f32,

    /// The maximum length that a collision ray is allowed to have, expressed as a multiple of
    /// the katamari's radius. (e.g. the default value of 2.5 means that no collision ray can be
    /// longer than 2.5 times the katamari's radius)
    /// default: 2.5
    /// offset: 0x716ec (used at 0x1c5ba)
    pub max_ray_len_radii: f32,

    /// The multiple of the katamari's usual collision radius that's applied to
    /// check for AABB collisions with props which have the "increased collision radius" property.
    /// default: 1.2
    /// offset: 0x71624 (used at 0x1c8c4)
    pub increased_collision_radius_mult: f32,

    /// If a floor surface's slope is larger than this value, it is considered steep enough
    /// to automatically accelerate the katamari downwards along it.
    /// default: 0.0453
    /// offset: 0x7b254
    pub min_slope_grade_causing_accel: f32,

    /// The maximum floor slope grade for the purposes of accelerating the katamari.
    /// All slopes with this grade or higher will have the same impact on the katamari's
    /// acceleration.
    /// default: 0.74
    /// offset: 0x7b204
    pub effective_max_slope_grade: f32,

    /// The number of ticks during which the katamari's uphill acceleration eases in after
    /// it starts moving uphill.
    pub uphill_accel_easein_duration: f32,

    /// The number of ticks during which the katamari's downhill acceleration eases in after
    /// it starts moving downhill
    pub downhill_accel_easein_duration: f32,

    /// The threshold value for the dot product of camera-forward and katamari-velocity directions
    /// which separates forwards, backwards, and sideways movement.
    /// default: 0.70710677 (i.e. 1/sqrt(2))
    /// offset: 0x715d8
    pub cam_relative_vel_sideways_threshold: f32,

    /// (??)
    /// default: 0x3
    /// offset: 0x7b208
    pub vault_tuning_0x7b208: f32,
}

impl Default for KatamariParams {
    fn default() -> Self {
        Self {
            init_wallclimb_cooldown_timer: 10,
            max_prop_collision_rays: 12,
            prop_attached_alpha: 0.995,
            prop_attach_vol_ratio: f32::from_bits(0x3dcccccd), // 0.1
            prop_use_aabb_collision_vol_ratio: f32::from_bits(0x3f59999a), // 0.85
            min_brake_angle: (f32::from_bits(0x3eab020c) * FRAC_PI_2).cos(),
            max_wallclimb_angle: (f32::from_bits(0x3ecccccd) * FRAC_PI_2).cos(),
            clip_len_constant: f32::from_bits(0x3a03126f),
            surface_similarity_threshold: f32::from_bits(0x3f7ffeb0),
            sloped_floor_normal_y_threshold: f32::from_bits(0x3f7ff972),
            wall_to_floor_angle_stuck_threshold: f32::from_bits(0x40060a92), // 3pi/2
            wall_to_wall_angle_stuck_threshold: f32::from_bits(0x40278d36),  // 5pi/6
            detach_cooldown_when_stuck_btwn_walls: 15,
            base_detached_prop_vol_mult: f32::from_bits(0x3cf5c28f),
            stuck_detached_prop_vol_mult: 0.5,
            vault_prop_pull_to_center_mult: f32::from_bits(0x3c75c28f), // 0.015
            radius_boost_cm: f32::from_bits(0x3c23d70a),                // 0.01
            display_radius_ratio: f32::from_bits(0x3ec28f5c),           // 0.38
            max_wallclimb_height_ratio: f32::from_bits(0x3f333333),
            wallclimb_speed_penalty: f32::from_bits(0x3f733333), // 0.95
            boost_build_duration: 14,
            boost_release_duration: 15,
            boost_release_duration_in_water: 10,
            forwards_speed_mult: 1.0,
            sideways_speed_mult: 1.0,
            backwards_speed_mult: 1.0,
            boost_speed_mult: 1.0,
            brakeable_max_speed_ratio: f32::from_bits(0x3ee66666),
            brake_vfx_cooldown: 10,
            min_speed_to_move: f32::from_bits(0x3c23d70a), // 0.01
            bottom_ray_friction: f32::from_bits(0x3d102de0), // 0.0352
            nonbottom_ray_friction: f32::from_bits(0x3ba3d70a), // 0.005
            speed_check_off_friction_reduction: f32::from_bits(0x3f0ccccd), // 0.55
            move_into_wall_similarity: f32::from_bits(0x3c23d70a), // 0.01
            unstuck_bump_speed: f32::from_bits(0x3d4ccccd), // 0.05
            surface_normal_y_threshold: (f32::from_bits(0x3f3d70a4) * FRAC_PI_2).cos(),
            max_ray_len_radii: 2.5,
            increased_collision_radius_mult: 1.2,
            min_slope_grade_causing_accel: f32::from_bits(0x3d398c7e), // 0.0453
            effective_max_slope_grade: f32::from_bits(0x3f3d70a4),     // 0.074
            uphill_accel_easein_duration: 20.0,
            downhill_accel_easein_duration: 20.0,
            cam_relative_vel_sideways_threshold: f32::from_bits(0x3f3504f3), // 0.70710677, or 1/sqrt(2)
            vault_tuning_0x7b208: f32::from_bits(0x3e99999a),                // 0.3
        }
    }
}

impl KatamariParams {
    pub fn get_speed_mult(&self, push_dir: PushDir) -> f32 {
        match push_dir {
            PushDir::Forwards => self.forwards_speed_mult,
            PushDir::Backwards => self.backwards_speed_mult,
            PushDir::Sideways => self.sideways_speed_mult,
        }
    }
}
