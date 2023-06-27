use gl_matrix::vec3;



use super::PropsState;

impl PropsState {
    // TODO: this keeps crashing the game, and i don't know why
    /// Request to draw the bounding box of prop with control index `ctrl_idx`.
    pub fn debug_draw_prop_bbox(&self, ctrl_idx: u16) {
        if let Some(delegates) = &self.delegates {
            if let Some(debug_draw_box) = delegates.borrow().debug_draw_box {
                if let Some(prop_ref) = self.props.get(ctrl_idx as usize) {
                    let prop = prop_ref.borrow();
                    let aabb = &prop.get_aabb_mesh().sectors[0].aabb;
                    let mut min = vec3::create();
                    let mut max = vec3::create();
                    vec3::transform_mat4(&mut min, &aabb.min, prop.get_unattached_transform());
                    vec3::transform_mat4(&mut max, &aabb.max, prop.get_unattached_transform());
                    debug_draw_box(
                        min[0], min[1], min[2], max[0], max[1], max[2], 0.0, 0.0, 1.0,
                    );
                }
            }
        }
    }
}
