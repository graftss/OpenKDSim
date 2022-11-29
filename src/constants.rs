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

/// There aren't this many stages, but this seems to be how many were allocated
/// in the simulation.
pub const NUM_STAGES: usize = 16;

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

pub const FRAC_4PI_3: f32 = PI * 4.0 / 3.0;

/// pi
/// offset: 0x716fc
pub const PI: f32 = f32::from_bits(0x40490fdb);

pub const TAU: f32 = f32::from_bits(0x40c90fdb);

/// 4pi, used for sphere volume
/// offset: 0x7174c
pub const _4PI: f32 = f32::from_bits(0x41490fdb);

/// 1/3
/// offset: 0x715b0
pub const FRAC_1_3: f32 = f32::from_bits(0x3eaaaaab);

/// PI/2
pub const FRAC_PI_2: f32 = f32::from_bits(0x3fc90fdb);

/// PI/750, used because reasons
/// offset: 0x71544
pub const FRAC_PI_750: f32 = f32::from_bits(0x1f42893b);

/// The triangulation of AABB's used in the original simulation.
/// An AABB is triangulated into 12 triangles (= 6 faces * 2 triangles/face),
/// and each triangle is encoded as a triple of vertex indices.
pub const AABB_TRIANGULATION: [u8; 36] = [
    0, 1, 2, 2, 1, 3, 2, 3, 4, 4, 3, 5, 4, 5, 6, 6, 5, 7, 6, 7, 0, 0, 7, 1, 6, 0, 2, 2, 4, 6, 1, 7,
    3, 3, 7, 5,
];

pub const ZERO: [u8; 1] = [0];
