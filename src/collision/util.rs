use gl_matrix::{
    common::{Mat4, Vec3},
    vec3,
};

/// Computes the highest y coordinate among all points in `vertices`
/// after they are transformed by `transform`.
pub fn max_transformed_y(vertices: &Vec<Vec3>, transform: &Mat4) -> f32 {
    let mut max_y = -f32::INFINITY;
    let mut trans_pt = [0.0; 3];

    for aabb_pt in vertices.iter() {
        // apply the `transform` to the `aabb_pt`
        vec3::transform_mat4(&mut trans_pt, &aabb_pt, &transform);
        if trans_pt[1] > max_y {
            max_y = trans_pt[1];
        }
    }

    max_y
}
