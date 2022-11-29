use std::f32::consts::{PI, TAU};

use gl_matrix::common::Vec3;

use crate::constants::{FRAC_1_3, _4PI};

/// Scale `vec` by `scale` in-place.
#[inline]
pub fn vec3_inplace_scale(vec: &mut Vec3, scale: f32) {
    vec[0] *= scale;
    vec[1] *= scale;
    vec[2] *= scale;
}

/// Subtract `<dx, dy, dz>` from `vec` in-place.
#[inline]
pub fn vec3_inplace_subtract(vec: &mut Vec3, dx: f32, dy: f32, dz: f32) {
    vec[0] -= dx;
    vec[1] -= dy;
    vec[2] -= dz;
}

/// Add `<dx, dy, dz>` to `vec` in-place.
#[inline]
pub fn vec3_inplace_add(vec: &mut Vec3, dx: f32, dy: f32, dz: f32) {
    vec[0] += dx;
    vec[1] += dy;
    vec[2] += dz;
}

/// Add `other` to `vec` in-place.
#[inline]
pub fn vec3_inplace_add_vec(vec: &mut Vec3, other: &Vec3) {
    vec[0] += other[0];
    vec[1] += other[1];
    vec[2] += other[2];
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
    if len > 1e-8 {
        len = 1_f32 / f32::sqrt(len);
    }

    vec[0] *= len;
    vec[1] *= len;
    vec[2] *= len;
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

#[inline]
pub fn change_bounded_angle(angle: &mut f32, delta: f32) {
    *angle = normalize_bounded_angle(*angle + delta);
}

#[inline]
pub fn acos_f32(value: f32) -> f32 {
    match value {
        _ if value >= 1.0 => 0.0,
        _ if value <= -1.0 => PI,
        _ => value.acos(),
    }
}

// TODO: this should probably be using the janky `power` function defined in
// the simulation for true accuracy
#[inline]
pub fn vol_to_rad(vol: f32) -> f32 {
    (vol * 3.0 / _4PI).powf(FRAC_1_3)
}
