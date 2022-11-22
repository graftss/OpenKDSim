/// The rescale factor when translating Unity coordinates to simulation coordinates.
/// (All simulation world space coordinates are 100 times bigger than equivalent Unity coordinates.)
pub static UNITY_TO_SIM_SCALE: f32 = 100.0;

/// The index of the x translation in a `gl_matrix::Mat4` array.
pub static TRANSFORM_X_POS: usize = 12;

/// The index of the y translation in a `gl_matrix::Mat4` array.
pub static TRANSFORM_Y_POS: usize = 13;

/// The index of the z translation in a `gl_matrix::Mat4` array.
pub static TRANSFORM_Z_POS: usize = 14;

/// The number of prop types.
pub const NUM_NAME_PROPS: usize = 1718;

/// The number of missions.
pub const NUM_MISSIONS: usize = 37;

/// The maximum number of players.
pub const MAX_PLAYERS: usize = 2;

/// The maximum number of props in a mission.
pub const MAX_PROPS: usize = 4000;

pub struct PhysicsConstants {
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
}

impl Default for PhysicsConstants {
    fn default() -> Self {
        Self {
            prop_vertical_vel_decay_after_bounce: 0.46,
            prop_lateral_vel_decay_after_bounce: 0.46,
            prop_spin_after_landing_ticks: 60,
            prop_min_hit_angle_for_wobble_deg: 70.0,
        }
    }
}
