use gl_matrix::{
    common::{Mat4, Vec4},
    vec4,
};

const SPLINE_COEFFS: Mat4 = [
    2.0, 1.0, 1.0, -2.0, -3.0, -2.0, -1.0, 3.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
];

/// some kind of spline tuning, but it's just always 0 in the original simulation
/// offset: 0x10eadc
const SPLINE_PARAM: f32 = 0.0;

/// offset: 0xfd40
pub fn compute_spline_point(m: &Mat4, t: f32) -> Vec4 {
    let powers = [t * t * t, t * t, t, 1.0];
    let K = (1.0 - SPLINE_PARAM) * 0.5;

    let mut vec = [0.0; 4];
    vec4::transform_mat4(&mut vec, &powers, &SPLINE_COEFFS);

    [
        (m[4] - m[0] + m[8] - m[4]) * K * vec[1]
            + m[4] * vec[0]
            + (m[12] - m[8] + m[8] - m[4]) * K * vec[3]
            + m[8] * vec[3],
        (m[9] - m[5] + m[13] - m[9]) * K * vec[2]
            + m[5] * vec[0]
            + (m[5] - m[1] + m[9] - m[5]) * K * vec[1]
            + vec[3] * m[9],
        (m[6] - m[2] + m[10] - m[6]) * K * vec[1]
            + m[6] * vec[0]
            + (m[14] - m[10] + m[10] - m[6]) * K * vec[2]
            + m[10] * vec[3],
        (m[7] - m[3] + m[11] - m[7]) * K * vec[1]
            + m[7] * vec[0]
            + (m[15] - m[11] + m[11] - m[7]) * K * vec[2]
            + m[11] * vec[3],
    ]
}
