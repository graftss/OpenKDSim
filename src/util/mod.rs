pub mod spline;

use std::{fs::OpenOptions, path::Path};

use gl_matrix::common::{Mat4, Vec3, Vec4};

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
        *val = f32::from_le_bytes(bytes[val_offset..val_offset + 4].try_into().unwrap());
    }
}

/// Read a `Vec3` from offset `offset` of a `u8` slice.
pub fn vec3_from_le_bytes(out: &mut Vec3, bytes: &[u8], offset: usize) {
    for (i, val) in out.iter_mut().enumerate() {
        let val_offset = offset + i * 4;
        *val = f32::from_le_bytes(bytes[val_offset..val_offset + 4].try_into().unwrap());
    }
}

/// Write the string `str` to the file `path`.
pub fn debug_write(path: &str, str: &str) {
    use std::io::Write;

    let path = Path::new(path);

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(path)
        .unwrap();

    if let Err(_e) = writeln!(file, "{}", str) {}
}

const DEBUG_LOG_PATH: &'static str =
    "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Katamari Damacy REROLL\\debug.log";

pub fn debug_log(str: &str) {
    debug_write(&DEBUG_LOG_PATH, str);
}
