use gl_matrix::{
    common::{Mat4, Vec3, Vec4},
    vec4,
};

use crate::{
    collision::mesh::Mesh,
    debug::DEBUG_CONFIG,
    macros::{max, min, vec3_from},
    util::color,
};

use super::Katamari;

impl Katamari {
    pub fn debug_should_log(&self) -> bool {
        return false;
    }

    pub fn debug_clip_state(&self) -> String {
        let overall_info = format!(
            "num_floors: {:?}, floor_normal:{:?}, floor_clip:{:?}, clip_trans:{:?}",
            self.num_floor_contacts,
            self.contact_floor_normal_unit,
            self.contact_floor_clip,
            self.clip_translation
        );

        let per_surface_info = self.hit_floors.iter().enumerate().fold(
            "".to_string(),
            |summary, (floor_idx, floor)| {
                summary
                    + &format!(
                        "\n  floor {floor_idx}: normal={:?}, clip={:?}",
                        floor.normal_unit, floor.clip_normal
                    )
            },
        );

        overall_info + &per_surface_info
    }

    /// Use the `debug_draw_line` delegate to draw the katamari's collision rays on the screen.
    #[rustfmt::skip]
    pub fn debug_draw_collision_rays(&self) {
        if !DEBUG_CONFIG.kat_draw_collision_rays { return; }

        if let Some(delegates) = &self.delegates {
            let mut my_delegates = delegates.borrow_mut();

            // compute max ray length
            let mut max_ray_len = 0.0;
            let mut min_ray_len = self.max_ray_len;
            for ray in self.collision_rays.iter() {
                max_ray_len = max!(max_ray_len, ray.ray_len);
                min_ray_len = min!(min_ray_len, ray.ray_len);
            }

            let mut floor_hits = vec![false; self.collision_rays.len()];
            let mut wall_hits = vec![false; self.collision_rays.len()];

            self.hit_floors.iter().for_each(|surface| if surface.ray_idx >= 0 {
                floor_hits[surface.ray_idx as usize] = true
            });
            self.hit_walls.iter().for_each(|surface| if surface.ray_idx >= 0 {
                wall_hits[surface.ray_idx as usize] = true
            });

            for (ray_idx, ray) in self.collision_rays.iter().enumerate() {
                let p0 = &self.center;
                let p1 = vec3_from!(+, ray.kat_to_endpoint, self.center);

                let base_ray_color = match ray_idx {
                    0 => color::BLACK,
                    _ if (ray_idx as u16) < self.first_prop_ray_index => color::RED,
                    _ => color::BLUE,
                };

                let hit_color = if floor_hits[ray_idx] {
                    Some(color::GREEN)
                } else if wall_hits[ray_idx] {
                    Some(color::DARK_GREEN)
                } else {
                    None
                };

                let mut color = base_ray_color.clone();

                if let Some(hit_color) = hit_color {
                    vec4::lerp(&mut color, &base_ray_color, &hit_color, 0.8);
                }

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

    pub fn debug_draw_collided_prop_mesh(&self, mesh: &Mesh, transform: &Mat4) {
        static MESH_COLOR: Vec4 = [0.0, 0.4, 0.0, 0.1];
        if let Some(delegates) = &self.delegates {
            let mut my_delegates = delegates.borrow_mut();

            for sector in mesh.sectors.iter() {
                for tri_group in sector.tri_groups.iter() {
                    my_delegates
                        .debug_draw
                        .draw_tri_group(tri_group, transform, &MESH_COLOR);
                }
            }
        }
    }

    pub fn debug_move_over_prop_bug_state(&self) -> String {
        let wall_str = if self.num_wall_contacts > 0 {
            let wall = &self.hit_walls[0];
            format!("  wall: {wall:?}")
        } else {
            "".to_owned()
        };

        format!(
            "speed: {}, num_floors:{}, num_walls:{}\n",
            self.speed, self.num_floor_contacts, self.num_wall_contacts
        ) + &wall_str
    }

    pub fn debug_velocity_state(&self) -> String {
        format!("{:?}", self.velocity)
    }
}
