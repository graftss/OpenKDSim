use gl_matrix::common::{Mat4, Vec4};

use crate::{
    constants::MAT4_ID,
    util::color::{CLEAR, GREEN, RED_TRANS, TRANS_PINK},
};

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

    /// Request to draw the mesh of the prop with control index `ctrl_idx`.
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

                    my_delegates.debug_draw.draw_box(
                        &sector.aabb.min,
                        &sector.aabb.max,
                        transform,
                        &GREEN,
                    );
                }
            }
        }
    }

    /// Request to draw the mesh sector corresponding to the prop zone `zone_id`.
    pub fn debug_draw_prop_zone(&self, zone_id: u8) {
        static ZONE_TRI_COLORS: [Vec4; 1] = [TRANS_PINK];
        static TRANSFORM: Mat4 = MAT4_ID;

        if let (Some(delegates), Some(raycast)) = (&self.delegates, &self.raycasts) {
            let mut my_delegates = delegates.borrow_mut();
            let my_raycast = raycast.borrow();
            if let Some(zone_sector) = my_raycast.get_zone_mesh_sector(zone_id) {
                my_delegates
                    .debug_draw
                    .draw_mesh_sector(zone_sector, &TRANSFORM, &ZONE_TRI_COLORS)
            }
        }
    }
}
