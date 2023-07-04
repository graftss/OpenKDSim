use core::slice;

use gl_matrix::{mat4, vec3};

use crate::debug::DebugDrawType;

use super::PropsState;

impl PropsState {
    /// Request to draw the bounding box of prop with control index `ctrl_idx`.
    #[rustfmt::skip]
    pub fn debug_draw_prop_bbox(&self, ctrl_idx: u16) {
        if let Some(delegates) = &self.delegates {
            let my_delegates = delegates.borrow();
            if let Some(draw) = my_delegates.debug_draw {
                if let Some(prop_ref) = self.props.get(ctrl_idx as usize) {
                    let prop = prop_ref.borrow();
                    let aabb = &prop.get_aabb_mesh().unwrap().clone().sectors[0].aabb;
                    let min = aabb.min;
                    let max = aabb.max;
                    let transform = prop.get_unattached_transform();

                    unsafe {
                        let mut out = my_delegates.debug_draw_data as *mut f32;

                        let mut out_min: &mut [f32; 3] = slice::from_raw_parts_mut(out, 3).try_into().unwrap();
                        vec3::copy(&mut out_min, &min);
                        out = out.offset(3);

                        let mut out_max: &mut [f32; 3] = slice::from_raw_parts_mut(out, 3).try_into().unwrap();
                        vec3::copy(&mut out_max, &max);
                        out = out.offset(3);

                        let mut out_transform: &mut [f32; 16] = slice::from_raw_parts_mut(out, 16).try_into().unwrap();
                        mat4::copy(&mut out_transform, &transform);
                        out = out.offset(16);

                        let out_color: &mut [f32; 4] = slice::from_raw_parts_mut(out, 4).try_into().unwrap();
                        out_color[0] = 0.0;
                        out_color[1] = 0.0;
                        out_color[2] = 1.0;
                        out_color[3] = 1.0;

                        draw(DebugDrawType::Box);
                    }
                }
            }
        }
    }
}
