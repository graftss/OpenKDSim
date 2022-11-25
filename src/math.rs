use std::f32::consts::{PI, TAU};

use gl_matrix::{
    common::{Vec2, Vec3},
    vec2,
};

/// Scale `vec` by `scale` in-place.
#[inline]
pub fn vec3_inplace_scale(vec: &mut Vec3, scale: f32) {
    vec[0] *= scale;
    vec[1] *= scale;
    vec[2] *= scale;
}

/// Zero out components of `vec` with small magnitude (less than `eps`), in-place.
#[inline]
pub fn vec3_inplace_zero_small(vec: &mut Vec3, eps: f32) {
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

pub fn vec3_inplace_normalize(vec: &mut Vec3) {
    let x = vec[0];
    let y = vec[1];
    let z = vec[2];

    let mut len = x * x + y * y + z * z;
    if len > 0_f32 {
        //TODO: evaluate use of glm_invsqrt here?
        len = 1_f32 / f32::sqrt(len);
    }

    vec[0] = vec[0] * len;
    vec[1] = vec[1] * len;
    vec[2] = vec[2] * len;
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
