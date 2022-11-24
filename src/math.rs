use std::f32::consts::{PI, TAU};

use gl_matrix::common::Vec3;

/// Scale `vec` by `scale` in-place.
#[inline]
pub fn vec3_scale_inplace(vec: &mut Vec3, scale: f32) {
    vec[0] *= scale;
    vec[1] *= scale;
    vec[2] *= scale;
}

/// Zero out components of `vec` with small magnitude (less than `eps`), in-place.
#[inline]
pub fn vec3_zero_small_inplace(vec: &mut Vec3, eps: f32) {
    if vec[0].abs() < eps {
        vec[0] = 0.0
    }
    if vec[1].abs() < eps {
        vec[1] = 0.0
    }
    if vec[2].abs() < eps {
        vec[2] = 0.0
    }
}

/// Normalize a bounded angle in [-2pi, 2pi] to [-pi, pi].
#[inline]
pub fn normalize_bounded_angle(angle: f32) -> f32 {
    if angle >= PI {
        angle - TAU
    } else if angle < -PI {
        angle + TAU
    } else {
        angle
    }
}
