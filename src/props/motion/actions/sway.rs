use gl_matrix::{common::Vec3, mat4, vec3};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{FRAC_PI_180, FRAC_PI_45, VEC3_Y_NEG},
    macros::set_translation,
    math::{normalize_bounded_angle, vec3_inplace_scale},
    props::prop::{Prop, PropFlags2},
};

use super::ActionUpdate;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SwayAction {
    initialized: bool,

    /// Unused?
    /// offset: 0x1
    field_0x1: u8,

    /// offset: 0x10
    init_pos: Vec3,

    /// offset: 0x20
    y_pos_offset: f32,

    /// offset: 0x24
    sway_progress: f32,

    /// offset: 0x28
    sway_speed: f32,

    /// offset: 0x2c
    sway_angle_deg: f32,
}

impl ActionUpdate for SwayAction {
    fn update(&mut self, prop: &mut Prop) {
        if !self.initialized {
            // not initialized: do initialization (`pmot_misc_init`)
            let name_idx = prop.get_name_idx();
            let aabb_max_y = -prop.get_aabb_max_y();

            let flags2 = prop.get_flags2_mut();
            flags2.insert(PropFlags2::Motion0x40);

            self.field_0x1 = 5;

            self.y_pos_offset = match name_idx {
                // name index 0x16b is "balancing toy"
                0x16b => 0.0,
                _ => aabb_max_y,
            };

            vec3::copy(&mut self.init_pos, prop.get_position());
            self.init_pos[1] += self.y_pos_offset;

            self.sway_progress = 0.0;
            self.sway_speed = FRAC_PI_45;
            self.sway_angle_deg = 10.0;

            self.initialized = true;
        } else {
            // already initialized: normal sway update (`pmot_22_sway`)
            // offset: 0x3fa40
            let follow_parent = prop.get_flags2().contains(PropFlags2::FollowParent);
            let mut progress = self.sway_progress;

            if !follow_parent {
                // TODO_LOW: should be the `delta` passed to `Tick`, for some reason
                // TODO_PARAM: 30.0 seems to be sway amplitude
                progress += self.sway_speed * 30.0 * (1.0 / 30.0);
                progress = normalize_bounded_angle(progress);
                self.sway_progress = progress;
            }

            let y = progress.sin();
            let sway_angle_rad = y * self.sway_angle_deg * FRAC_PI_180;

            let mut sway_rot_mat = [0.0; 16];
            mat4::from_z_rotation(&mut sway_rot_mat, sway_angle_rad);

            let mut sway_down = [0.0; 3];
            vec3::transform_mat4(&mut sway_down, &VEC3_Y_NEG, &sway_rot_mat);

            vec3_inplace_scale(&mut sway_down, self.y_pos_offset);

            let mut transform = prop.get_unattached_transform().clone();
            set_translation!(transform, [0.0; 3]);

            let mut delta_pos = [0.0; 3];
            vec3::transform_mat4(&mut delta_pos, &sway_down, &transform);

            if !follow_parent {
                vec3::add(&mut prop.pos, &delta_pos, &self.init_pos);
            } else {
                vec3::subtract(&mut self.init_pos, &prop.pos, &delta_pos);
            }

            prop.rotation_vec[2] = sway_angle_rad;
        }
    }

    fn should_do_alt_motion(&self) -> bool {
        false
    }
}
