use crate::constants::FRAC_PI_2;

#[derive(Debug)]
pub struct KatamariParams {
    /// The number of ticks where the katamari can't start a second climb after falling out
    /// of a first climb.
    pub init_wallclimb_cooldown: u16,

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
}

impl Default for KatamariParams {
    fn default() -> Self {
        Self {
            init_wallclimb_cooldown: 10,
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
        }
    }
}
