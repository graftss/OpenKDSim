use crate::constants::FRAC_PI_2;

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

    /// (??)
    /// offset: 0x7b254 (but this value is `cos(thatvalue * pi/2)` because reasons)
    pub min_slope_grade: f32,

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
    pub sloped_floor_y_normal_threshold: f32,

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
}

impl Default for KatamariParams {
    fn default() -> Self {
        Self {
            init_wallclimb_cooldown_timer: 10,
            max_prop_collision_rays: 12,
            prop_attached_alpha: 0.995,
            prop_attach_vol_ratio: f32::from_bits(0x3dcccccd), // 0.1
            prop_use_aabb_collision_vol_ratio: f32::from_bits(0x3f59999a), // 0.85
            min_slope_grade: (f32::from_bits(0x3d398c7e) * FRAC_PI_2).cos(),
            min_brake_angle: (f32::from_bits(0x3eab020c) * FRAC_PI_2).cos(),
            max_wallclimb_angle: (f32::from_bits(0x3ecccccd) * FRAC_PI_2).cos(),
            clip_len_constant: f32::from_bits(0x3a03126f),
            surface_similarity_threshold: f32::from_bits(0x3f7ffeb0),
            sloped_floor_y_normal_threshold: f32::from_bits(0x3f7ff972),
            wall_to_floor_angle_stuck_threshold: f32::from_bits(0x40060a92), // 3pi/2
            wall_to_wall_angle_stuck_threshold: f32::from_bits(0x40278d36),  // 5pi/6
            detach_cooldown_when_stuck_btwn_walls: 15,
            base_detached_prop_vol_mult: f32::from_bits(0x3cf5c28f),
            stuck_detached_prop_vol_mult: 0.5,
            vault_prop_pull_to_center_mult: f32::from_bits(0x3c75c28f), // 0.015
            radius_boost_cm: f32::from_bits(0x3c23d70a),                // 0.01
            display_radius_ratio: f32::from_bits(0x3ec28f5c),           // 0.38
            max_wallclimb_height_ratio: f32::from_bits(0x3f333333),     // 0.7
        }
    }
}
