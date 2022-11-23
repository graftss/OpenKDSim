use gl_matrix::common::Vec4;

#[inline]
pub fn vec4_scale(vec: &mut Vec4, scale: f32) {
    vec[0] *= scale;
    vec[1] *= scale;
    vec[2] *= scale;
}
