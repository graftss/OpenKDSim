use gl_matrix::common::{Mat4, Vec4};

use crate::constants::{TRANSFORM_X_POS, TRANSFORM_Y_POS, TRANSFORM_Z_POS, UNITY_TO_SIM_SCALE};

/// Rescale the translation components of a `Mat4` transform from
/// simulation coordinates to Unity coordinates.
pub fn scale_sim_transform(transform: &mut Mat4) {
  transform[TRANSFORM_X_POS] /= UNITY_TO_SIM_SCALE;
  transform[TRANSFORM_Y_POS] /= UNITY_TO_SIM_SCALE;
  transform[TRANSFORM_Z_POS] /= UNITY_TO_SIM_SCALE;
}

/// Read a `Vec4` from offset `offset` of a `u8` slice.
pub fn vec4_from_le_bytes(out: &mut Vec4, bytes: &[u8], offset: usize) {
  for (i, val) in out.iter_mut().enumerate() {
    let val_offset = offset + i * 4;
    *val = f32::from_le_bytes(bytes[val_offset..val_offset+4].try_into().unwrap());
  }
}
