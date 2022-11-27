#[derive(Debug)]
pub struct KatamariParams {
    /// The number of ticks where the katamari can't start a second climb after falling out
    /// of a first climb.
    pub init_wallclimb_cooldown: u16,

    /// The maximum number of collision rays that can be induced by props.
    pub max_prop_collision_rays: u16,

    /// The alpha of props which are attached to the katamari.
    pub prop_attached_alpha: f32,

    /// The fraction of the katamari's volume that can be attached.
    /// (e.g. a value of 0.1 means the katamari can attach props 10% as big as it)
    /// offset: 0x7b220
    pub prop_attach_vol_ratio: f32,

    /// The multiple of the katamari's volume to use as the
    /// `prop_use_aabb_collision_vol` threshold
    /// offset: 0x7b110
    pub prop_use_aabb_collision_vol_ratio: f32,
}

impl Default for KatamariParams {
    fn default() -> Self {
        Self {
            init_wallclimb_cooldown: 10,
            max_prop_collision_rays: 12,
            prop_attached_alpha: 0.995,
            prop_attach_vol_ratio: f32::from_bits(0x3dcccccd), // 0.1
            prop_use_aabb_collision_vol_ratio: f32::from_bits(0x3f59999a), // 0.85
        }
    }
}
