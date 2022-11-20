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
pub const MAX_PROPS: usize = 199;
