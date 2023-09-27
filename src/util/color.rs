use gl_matrix::common::Vec4;

/// The RGBA color (1, 0, 0, 1).
pub const RED: Vec4 = [1.0, 0.0, 0.0, 1.0];
pub const RED_TRANS: Vec4 = [1.0, 0.0, 0.0, 0.3];

pub const PINK: Vec4 = [0.98, 0.55, 0.45, 1.0];
pub const TRANS_PINK: Vec4 = [0.98, 0.55, 0.45, 0.3];

/// The RGBA color (0, 1, 0, 1).
pub const GREEN: Vec4 = [0.0, 1.0, 0.0, 1.0];
pub const GREEN_TRANS: Vec4 = [0.0, 1.0, 0.0, 0.3];

/// The RGBA color (0, 0.5, 0, 1).
pub const DARK_GREEN: Vec4 = [0.0, 0.5, 0.0, 1.0];
pub const DARK_GREEN_TRANS: Vec4 = [0.0, 0.5, 0.0, 0.3];

/// The RGBA color (0, 0, 1, 1).
pub const BLUE: Vec4 = [0.0, 0.0, 1.0, 1.0];
pub const BLUE_TRANS: Vec4 = [0.0, 0.0, 1.0, 0.3];

/// The RGBA color (0, 0, 0, 1).
pub const BLACK: Vec4 = [0.0, 0.0, 0.0, 1.0];
pub const BLACK_TRANS: Vec4 = [0.0, 0.0, 0.0, 0.3];

pub const CLEAR: Vec4 = [0.0, 0.0, 0.0, 0.0];
