use gl_matrix::vec3;

use crate::mission::{state::MissionState, GameMode};

use self::ray::KatCollisionRayType;

use super::Katamari;

pub mod hit;
pub mod mesh;
pub mod ray;

impl Katamari {
    /// The main function to update the katamari's collision state.
    pub fn update_collision(&mut self, mission_state: &MissionState) {
        self.last_num_floor_contacts = self.num_floor_contacts;
        self.last_num_wall_contacts = self.num_wall_contacts;
        self.num_floor_contacts = 0;
        self.num_wall_contacts = 0;
        self.fc_ray_idx = None;

        // TODO_VS: `kat_update_collision:41-61` (compute vol ratio for vs mode)

        self.use_prop_aabb_collision_vol =
            self.vol_m3 * self.params.prop_use_aabb_collision_vol_ratio;
        self.contact_prop = None;
        self.max_attach_vol_m3 = self.vol_m3 * self.params.prop_attach_vol_ratio;
        self.physics_flags.reset_for_collision_update();

        if self.prop_pseudo_iframes_timer > 0 {
            self.prop_pseudo_iframes_timer -= 1;
        }
        if self.prop_ignore_collision_timer > 0 {
            self.prop_ignore_collision_timer -= 1;
        }

        self.lowest_wall_contact_y = 100000.0;
        self.lowest_floor_contact_y = 100000.0;

        // don't check collision when you're in whatever vsmode state this is
        if self.physics_flags.vs_mode_some_state == 2 {
            return;
        }

        // record if the katamari moved more than its radius
        let mut moved = vec3::create();
        vec3::subtract(&mut moved, &self.last_center, &self.center);
        self.physics_flags.moved_more_than_rad = self.radius_cm <= vec3::length(&moved);
        self.physics_flags.moved_too_much_0x14 = false;

        // TODO_VS: `kat_update_collision:96-101` (decrement timer)

        // TODO: `kat_initial_process_props()`

        if mission_state.gamemode == GameMode::Ending {
            // TODO: `kat_update_collision:105-132
        } else {
            // TODO: `kat_update_water_contact()`
            // TODO: `kat_update_surface_contacts()`
            // TODO: `kat_apply_surface_contacts()`
            // TODO: `kat_update_contact_history_and_stuckness()`
            // TODO: `kat_update_vault_and_climb()

            if self.physics_flags.airborne && self.raycast_state.closest_hit_idx.is_some() {
                // TODO: `kat_update_collision:159-220` (update active hit before airborne?)
            }
        }

        // TODO: `kat_update_collision:222-266` (destroy collected props that are sucked inside the ball)
        // TODO: `kat_process_props_inside_sphere()`
        // TODO: `kat_process_collected_props()`
        // TODO: `kat_update_world_size_threshold??()`

        if self.physics_flags.vault_ray_type == Some(KatCollisionRayType::Bottom)
            || self.fc_ray_len < 1.0
        {
            self.fc_ray_len = self.radius_cm;
        }
    }
}
