#[derive(Debug)]
pub struct KatamariParams {
    /// The number of ticks where the katamari can't start a second climb after falling out
    /// of a first climb.
    pub init_wallclimb_cooldown: u16,

    /// The maximum number of collision rays that can be induced by props.
    pub max_prop_collision_rays: u16,

    /// The alpha of props which are attached to the katamari.
    pub prop_attached_alpha: f32,
}

impl Default for KatamariParams {
    fn default() -> Self {
        Self {
            init_wallclimb_cooldown: 10,
            max_prop_collision_rays: 12,
            prop_attached_alpha: 0.995,
        }
    }
}
