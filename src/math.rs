use std::f32::consts::{PI, TAU};

use gl_matrix::{
    common::{Mat4, Vec3},
    mat4, vec3,
};

use crate::{
    constants::{FRAC_1_3, _4PI},
    macros::{set_translation, set_y},
};

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

/// Subtract `other` from `vec` in-place.
#[inline]
pub fn vec3_inplace_subtract_vec(vec: &mut Vec3, other: &Vec3) {
    vec[0] -= other[0];
    vec[1] -= other[1];
    vec[2] -= other[2];
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

/// Add `scale * other` to `vec` in-place.
#[inline]
pub fn vec3_inplace_add_scaled(vec: &mut Vec3, other: &Vec3, scale: f32) {
    vec[0] += other[0] * scale;
    vec[1] += other[1] * scale;
    vec[2] += other[2] * scale;
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

/// Computes the vector projection and rejection of u onto v.
pub fn vec3_projection(u_proj_v: &mut Vec3, u_rej_v: &mut Vec3, u: &Vec3, v: &Vec3) {
    vec3::scale(u_proj_v, &v, vec3::dot(u, v));
    vec3::subtract(u_rej_v, u, u_proj_v);
}

/// Computes the refection of `u` across `v`
/// (see https://math.stackexchange.com/questions/13261/how-to-get-a-reflection-vector)
pub fn vec3_reflection(u_reflect_v: &mut Vec3, u: &Vec3, v: &Vec3) {
    let dot = vec3::dot(u, v);
    vec3::scale_and_add(u_reflect_v, u, v, -2.0 * dot);
}

/// Writes the yaw rotation matrix of `mat` to `out`.
pub fn mat4_compute_yaw_rot(out: &mut Mat4, mat: &Mat4) {
    let mut left_unit = vec3::create();
    let mut mat_rot = mat.clone();
    set_translation!(mat_rot, [0.0, 0.0, 0.0]);
    vec3::transform_mat4(&mut left_unit, &[1.0, 0.0, 0.0], &mat_rot);

    if left_unit[0] == 0.0 && left_unit[2] == 0.0 {
        // if the left vector is somehow the y+ or y- axis, just set it to x+, apparently
        left_unit[0] = 1.0;
    } else {
        set_y!(left_unit, 0.0);
        vec3_inplace_normalize(&mut left_unit);
    }

    // using the left vector, compute the yaw rotation component of the lookat matrix.
    mat4::identity(out);
    out[0] = left_unit[0];
    out[2] = left_unit[2];
    out[8] = -left_unit[2];
    out[10] = left_unit[0];
}

/// This is kind of janky but it seems to match the simulation.
pub fn mat4_look_at(out: &mut Mat4, eye: &Vec3, target: &Vec3, up: &Vec3) -> Mat4 {
    let eyex = eye[0];
    let eyey = eye[1];
    let eyez = eye[2];
    let upx = up[0];
    let upy = up[1];
    let upz = up[2];

    let mut z0 = eyex - target[0];
    let mut z1 = eyey - target[1];
    let mut z2 = eyez - target[2];

    let mut len = z0 * z0 + z1 * z1 + z2 * z2;
    if len > 0_f32 {
        len = 1. / f32::sqrt(len);
        z0 *= len;
        z1 *= len;
        z2 *= len;
    }

    let mut x0 = upy * z2 - upz * z1;
    let mut x1 = upz * z0 - upx * z2;
    let mut x2 = upx * z1 - upy * z0;

    len = x0 * x0 + x1 * x1 + x2 * x2;
    if len > 0_f32 {
        len = 1. / f32::sqrt(len);
        x0 *= len;
        x1 *= len;
        x2 *= len;
    }

    out[0] = -x0;
    out[4] = -x1;
    out[8] = -x2;
    out[3] = 0.;
    out[1] = z1 * x2 - z2 * x1;
    out[5] = z2 * x0 - z0 * x2;
    out[9] = z0 * x1 - z1 * x0;
    out[7] = 0.;
    out[2] = -z0; // neg
    out[6] = -z1;
    out[10] = -z2;
    out[11] = 0.;
    out[12] = eyex;
    out[13] = eyey;
    out[14] = eyez;
    out[15] = 1.;

    *out
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

/// Maps [-1.0, 1.0] to [PI, 0] using `acos`
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
