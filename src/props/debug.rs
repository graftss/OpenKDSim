use gl_matrix::vec3;

use crate::debug::DebugDrawType;

use super::PropsState;

impl PropsState {
    /// Request to draw the bounding box of prop with control index `ctrl_idx`.
    pub fn debug_draw_prop_bbox(&self, ctrl_idx: u16) {
        if let Some(delegates) = &self.delegates {
            if let Some(draw) = delegates.borrow().debug_draw {
                if let Some(prop_ref) = self.props.get(ctrl_idx as usize) {
                    let prop = prop_ref.borrow();
                    let aabb = &prop.get_aabb_mesh().unwrap().clone().sectors[0].aabb;
                    let mut min = vec3::create();
                    let mut max = vec3::create();
                    vec3::transform_mat4(&mut min, &aabb.min, prop.get_unattached_transform());
                    vec3::transform_mat4(&mut max, &aabb.max, prop.get_unattached_transform());
                    draw(
                        DebugDrawType::Box,
                        min[0],
                        min[1],
                        min[2],
                        max[0],
                        max[1],
                        max[2],
                        0.0,
                        0.0,
                        1.0,
                    );
                }
            }
        }
    }
}
