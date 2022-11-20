use gl_matrix::common::Mat4;

use crate::constants::{TRANSFORM_X_POS, TRANSFORM_Y_POS, TRANSFORM_Z_POS, UNITY_TO_SIM_SCALE};

/// Rescale the translation components of a `Mat4` transform from
/// simulation coordinates to Unity coordinates.
pub fn scale_sim_transform(transform: &mut Mat4) {
  transform[TRANSFORM_X_POS] /= UNITY_TO_SIM_SCALE;
  transform[TRANSFORM_Y_POS] /= UNITY_TO_SIM_SCALE;
  transform[TRANSFORM_Z_POS] /= UNITY_TO_SIM_SCALE;
}
