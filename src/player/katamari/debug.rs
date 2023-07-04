use gl_matrix::{common::Vec3, vec3};

use crate::{
    constants::VEC3_X_POS,
    debug::DEBUG_CONFIG,
    macros::{debug_log, vec3_from},
    math::acos_f32,
};

use super::Katamari;

impl Katamari {
    pub fn debug_should_log(&self) -> bool {
        return false;
    }

    /// Print information about the katamari's position and collision.
    /// `offset_note` is designed to be the offset in the original simulation corresponding
    /// to the point in the open simulation at which this function is called.
    /// That way, this data can be compared to the analogous data via a breakpoint in the
    /// original simulation.
    pub fn debug_log_clip_data(&self, offset_note: &str) {
        debug_log!("  {}", offset_note);
        debug_log!("    center:{:?}", self.center);

        // debug_log!("    rotation_speed:{:?}", self.rotation_speed);
        // debug_log!("    rotation_mat:{:?}", self.rotation_mat);
        // debug_log!("    rotation_axis:{:?}", self.rotation_axis_unit);
        // debug_log!("    camera_side_vector:{:?}", self.camera_side_vector);

        debug_log!("    contact floor clip:{:?}", self.contact_floor_clip);
        debug_log!("    clip_translation:{:?}", self.clip_translation);
        debug_log!(
            "    contact_floor_normal_unit:{:?}",
            self.contact_floor_normal_unit
        );
        debug_log!("    num_floor_contacts:{:?}", self.num_floor_contacts);
        for (idx, floor) in self.hit_floors.iter().enumerate() {
            debug_log!("    f{}: {:?}", idx, floor);
        }

        // for (idx, ray) in self.collision_rays.iter().enumerate() {
        //     debug_log!("    ray {}: {:?} (len={:?})", idx, ray.ray_local, ray.ray_len);
        //     if idx == 18 { break; }
        // }

        // fc data
        // debug_log!("    fc_ray_idx: {:?}", self.fc_ray_idx);
        // debug_log!("    fc_ray: {:?}", self.fc_ray);
        // debug_log!("    fc_ray_len: {:?}", self.fc_ray);
        // debug_log!("    fc_contact_point: {:?}", self.fc_contact_point);

        // bottom collision ray
        if let Some(ray) = self.collision_rays.get(0) {
            debug_log!("    bottom contact: {}", ray.contacts_surface);
            debug_log!("    bottom endpoint: {:?}", ray.endpoint);
            debug_log!("    bottom len: {}", ray.ray_len);
        } else {
            debug_log!("  NO BOTTOM RAY");
        }
    }

    /// Use the `debug_draw_line` delegate to draw the katamari's collision rays on the screen.
    #[rustfmt::skip]
    pub fn debug_draw_collision_rays(&self) {
        if !DEBUG_CONFIG.kat_draw_collision_rays { return; }

        if let Some(delegates) = &self.delegates {
            let mut my_delegates = delegates.borrow_mut();
            for (ray_idx, ray) in self.collision_rays.iter().enumerate() {
                let p0 = &self.center;
                let p1 = vec3_from!(+, ray.kat_to_endpoint, self.center);
                let color = if self.vault_ray_idx == Some(ray_idx as i16) {
                    [0.0, 1.0, 0.0, 1.0]
                } else {
                    [1.0, 0.0, 0.0, 1.0]
                };

                my_delegates.debug_draw.draw_line(p0, &p1, &color);
            }
        }
    }

    #[rustfmt::skip]
    pub fn debug_draw_shell_rays(
        &self,
        shell_initial_pts: &[Vec3; 5],
        shell_final_pts: &[Vec3; 5],
    ) {
        if !DEBUG_CONFIG.kat_draw_shell_rays { return; }

        let SHELL_RAY_COLOR = [0.0, 1.0, 1.0, 1.0];

        if let Some(delegates) = &self.delegates {
            let mut my_delegates = delegates.borrow_mut();
            for i in 0..5 {
                let p0 = shell_initial_pts[i];
                let p1 = shell_final_pts[i];
                my_delegates.debug_draw.draw_line(&p0, &p1, &SHELL_RAY_COLOR);
            }
        }
    }

    pub fn debug_velocity_state(&self) -> String {
        let vel_accel_len = vec3::length(&self.velocity.vel_accel);
        let angle = acos_f32(vec3::dot(&VEC3_X_POS, &self.velocity.vel_accel));
        format!(
            "vel_accel_len:{:?}, bottom_ray={:?}, angle={}, speed={}",
            vel_accel_len, self.physics_flags.grounded_ray_type, angle, self.speed
        )
    }
}
