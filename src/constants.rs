use std::f32::consts::PI;

use gl_matrix::common::{Vec3, Vec4};

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

/// The zero vector.
pub const VEC4_ZERO: Vec4 = [0.0, 0.0, 0.0, 1.0];

/// The positive x axis unit vector.
pub const VEC4_X_POS: Vec4 = [1.0, 0.0, 0.0, 1.0];

/// The negative x axis unit vector.
pub const VEC4_X_NEG: Vec4 = [-1.0, 0.0, 0.0, 1.0];

/// The positive y axis unit vector
pub const VEC4_Y_POS: Vec4 = [0.0, 1.0, 0.0, 1.0];

/// The negative y axis unit vector.
pub const VEC4_Y_NEG: Vec4 = [0.0, -1.0, 0.0, 1.0];

/// The positive z axis unit vector.
pub const VEC4_Z_POS: Vec4 = [0.0, 0.0, 1.0, 1.0];

/// The negative z axis unit vector.
pub const VEC4_Z_NEG: Vec4 = [0.0, 0.0, -1.0, 1.0];

/// The zero vector.
pub const VEC3_ZERO: Vec3 = [0.0, 0.0, 0.0];

/// The positive x axis unit vector.
pub const VEC3_X_POS: Vec3 = [1.0, 0.0, 0.0];

/// The negative x axis unit vector.
pub const VEC3_X_NEG: Vec3 = [-1.0, 0.0, 0.0];

/// The positive y axis unit vector
pub const VEC3_Y_POS: Vec3 = [0.0, 1.0, 0.0];

/// The negative y axis unit vector.
pub const VEC3_Y_NEG: Vec3 = [0.0, -1.0, 0.0];

/// The positive z axis unit vector.
pub const VEC3_Z_POS: Vec3 = [0.0, 0.0, 1.0];

/// The negative z axis unit vector.
pub const VEC3_Z_NEG: Vec3 = [0.0, 0.0, -1.0];

pub const _4PI_3: f32 = PI * 4.0 / 3.0;
