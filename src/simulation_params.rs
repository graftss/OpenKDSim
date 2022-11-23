#[derive(Debug)]
pub struct SimulationParams {
    /// The number of ticks where the katamari can't start a second climb after falling out
    /// of a first climb.
    pub kat_init_wallclimb_cooldown: u16,

    /// See `destroy_prop_diam_ratio_clearprops`. This ratio applies to normal mission.
    /// offset: 0x9eac8
    pub destroy_prop_diam_ratio_normal: f32,

    /// The largest a prop can be (represented as a ratio of the katamari's volume) such that
    /// it will destroy itself once the katamari moves far enough away from it to have 0 alpha.
    /// This ratio in particular applies to `GameMode::ClearProp` missions.
    /// offset: 0x9eacc
    pub destroy_prop_diam_ratio_clearprops: f32,

    /// See `destroy_prop_diam_ratio_clearprop`. This ratio is smaller, and applies to
    /// missions which have the `keep_smaller_props_alive` flag on.
    /// offset: 0x9ead0
    pub destroy_prop_diam_ratio_reduced: f32,

    /// The ratio of vertical velocity a prop keeps after bouncing off of a surface while airborne.
    /// offset: 0x155230
    pub prop_vertical_vel_decay_after_bounce: f32,

    /// The ratio of lateral velocity a prop keeps after bouncing off of a surface while airborne.
    /// offset: 0x155234
    pub prop_lateral_vel_decay_after_bounce: f32,

    /// The number of ticks a prop spends spinning on the ground after finishing airborne bouncing.
    /// offset: 0x155240
    pub prop_spin_after_landing_ticks: i32,

    /// When a katamari and prop collide, this is the minimum angle difference in their movement
    /// directions which can cause the prop to wobble.
    /// offset: 0x155244
    pub prop_min_hit_angle_for_wobble_deg: f32,

    /// The alpha of props which are attached to the katamari.
    pub prop_attached_alpha: f32,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            kat_init_wallclimb_cooldown: 10,
            prop_vertical_vel_decay_after_bounce: 0.46,
            prop_lateral_vel_decay_after_bounce: 0.46,
            prop_spin_after_landing_ticks: 60,
            prop_min_hit_angle_for_wobble_deg: 70.0,
            destroy_prop_diam_ratio_normal: 0.234,
            destroy_prop_diam_ratio_clearprops: 0.145,
            destroy_prop_diam_ratio_reduced: 0.1,
            prop_attached_alpha: 0.995,
        }
    }
}
