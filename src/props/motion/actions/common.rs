use gl_matrix::{common::Vec3, mat4, vec3};

use crate::math::vec3_inplace_zero_small;

/// Returns `false` if the `forward_unit` vector (in the xz plane) rotated by `angle`
/// is in the same direction as `to_target_unit` (again, in the xz plane).
/// Returns `true` if the above two vectors are instead in the same direction.
/// offset: 0x37150
pub fn is_not_facing_target(angle: f32, forward_unit: &Vec3, to_target_unit: &Vec3) -> bool {
    let mut yaw_rot = [0.0; 16];
    mat4::from_y_rotation(&mut yaw_rot, angle);

    // TODO_DOC: I have no idea what to call this.
    let mut forward2 = [0.0; 3];
    vec3::transform_mat4(&mut forward2, &forward_unit, &yaw_rot);

    forward2[0] -= to_target_unit[0];
    forward2[2] -= to_target_unit[2];

    vec3_inplace_zero_small(&mut forward2, 0.00001);

    !(forward2[0] == 0.0 && forward2[2] == 0.0)
}
