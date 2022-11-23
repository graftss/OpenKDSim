use gl_matrix::common::Vec4;

#[inline]
pub fn vec4_scale_inplace(vec: &mut Vec4, scale: f32) {
    vec[0] *= scale;
    vec[1] *= scale;
    vec[2] *= scale;
}

#[inline]
pub fn vec4_zero_small_inplace(vec: &mut Vec4, eps: f32) {
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
