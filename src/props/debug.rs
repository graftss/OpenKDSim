use gl_matrix::common::Vec4;

use crate::util::color::{CLEAR, RED_TRANS};

use super::PropsState;

impl PropsState {
    /// Request to draw the bounding box of prop with control index `ctrl_idx`.
    #[rustfmt::skip]
    pub fn debug_draw_prop_bbox(&self, ctrl_idx: u16) {
        static BBOX_COLOR: Vec4 = [0.0, 0.0, 1.0, 0.8];
        if let Some(delegates) = &self.delegates {
            let mut my_delegates = delegates.borrow_mut();
            if let Some(prop_ref) = self.props.get(ctrl_idx as usize) {
                let prop = prop_ref.borrow();
                let aabb = &prop.get_aabb_mesh().unwrap().clone().sectors[0].aabb;
                let min = aabb.min;
                let max = aabb.max;
                let transform = prop.get_unattached_transform();

                my_delegates.debug_draw.draw_box(&min, &max, transform, &BBOX_COLOR);
            }
        }
    }

    pub fn debug_draw_prop_mesh(&self, ctrl_idx: u16) {
        static TRI_GROUP_COLORS: [Vec4; 2] = [RED_TRANS, CLEAR];

        if let Some(delegates) = &self.delegates {
            let mut my_delegates = delegates.borrow_mut();
            if let Some(prop_ref) = self.props.get(ctrl_idx as usize) {
                let prop = prop_ref.borrow();
                let transform = prop.get_unattached_transform();
                let sectors = &prop.get_collision_mesh().unwrap().clone().sectors;

                for sector in sectors.iter() {
                    my_delegates
                        .debug_draw
                        .draw_mesh_sector(sector, transform, &TRI_GROUP_COLORS);
                }
            }
        }
    }
}
