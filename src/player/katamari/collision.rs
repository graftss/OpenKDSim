use gl_matrix::{common::Vec3, vec3};

use crate::{
    collision::raycast_state::RaycastCallType,
    macros::panic_log,
    math::{
        vec3_inplace_add, vec3_inplace_add_vec, vec3_inplace_scale, vec3_inplace_subtract,
        vec3_inplace_zero_small,
    },
    mission::{state::MissionState, GameMode},
    props::prop::WeakPropRef,
};

use self::{hit::SurfaceHit, ray::KatCollisionRayType};

use super::Katamari;

pub mod hit;
pub mod mesh;
pub mod ray;

// TODO: probably a better place for this
/// All surface triangles can be categorized as either a floor or a wall. They
/// are distinguished via the y component of their unit normal vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceType {
    Floor,
    Wall,
}

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
            // TODO: `kat_update_collision:105-132 (ending-specific reduced collision)
        } else {
            // TODO: `kat_update_water_contact()`
            self.update_surface_contacts();
            self.process_surface_contacts();
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

    fn update_surface_contacts(&mut self) {
        // TODO_VS: `kat_update_surface_contacts:59` (check to not run this in vs mode)

        // prepare the ray starting from katamari center and cast straight down.
        let dist_down = self.radius_cm * 3.0;
        let center = self.center.clone();
        let mut below = center.clone();
        vec3_inplace_subtract(&mut below, 0.0, -dist_down, 0.0);
        self.raycast_state.load_ray(&center, &below);

        // check for unity hits straight down.
        let found_hit = self
            .raycast_state
            .find_nearest_unity_hit(RaycastCallType::Objects, false);

        // update shadow position
        if !found_hit {
            // if there's no surface below, set the shadow position to the katamari top (idk why)
            vec3::copy(&mut self.shadow_pos, &self.center);
            self.shadow_pos[1] += self.radius_cm;
        } else if let Some(hit) = self.raycast_state.get_closest_hit() {
            // if there's a surface below, set the shadow position to the surface point below.
            vec3::copy(&mut self.shadow_pos, &hit.impact_point);

            // record when the hit is `MapSemiTranslucent`
            if hit.metadata == 0x17 {
                self.has_map_semi_translucent_hit = true;
            }
        }

        self.physics_flags.in_water_0x8 = self.physics_flags.in_water;
        if self.physics_flags.moved_more_than_rad {
            // TODO: `kat_update_surface_contacts:110-235` (weird crap when katamari moved a lot)
        } else {
            let last_center = self.last_center.clone();

            // TODO: make 0.15 a katamari param
            let shell_ray_len = self.radius_cm * 0.15;
            let mut shell_ray_vec = self.delta_pos.clone();
            vec3_inplace_scale(&mut shell_ray_vec, shell_ray_len);

            // TODO: support all 5 shell points
            let mut shell_initial_pts: [Vec3; 1] = Default::default();
            let mut shell_final_pts: [Vec3; 1] = Default::default();

            vec3::copy(&mut shell_initial_pts[0], &self.shell_top_center);
            vec3_inplace_add(
                &mut shell_initial_pts[0],
                last_center[0],
                0.0,
                last_center[2],
            );

            vec3::copy(&mut shell_final_pts[0], &self.shell_top_center);
            vec3_inplace_add_vec(&mut shell_final_pts[0], &shell_ray_vec);

            // TODO: `kat_update_surface_contacts:267-294` (support all shell points)

            for (_i, (point0, point1)) in
                Iterator::zip(shell_initial_pts.iter(), shell_final_pts.iter()).enumerate()
            {
                // check collisions along each shell ray
                self.raycast_state.load_ray(point0, point1);
                self.raycast_state
                    .find_nearest_unity_hit(RaycastCallType::Objects, false);

                // TODO: `kat_update_surface_contacts:308-372` (resolve shell ray hits)
            }

            let center = self.center.clone();
            let mut hit_ray_idx = None;
            for (ray_idx, ray) in self.collision_rays.iter().enumerate() {
                self.raycast_state.load_ray(&center, &ray.endpoint);
                if self
                    .raycast_state
                    .find_nearest_unity_hit(RaycastCallType::Objects, false)
                {
                    // TODO: there's a flag being ignored here
                    if true && !self.last_physics_flags.airborne {
                        self.physics_flags.airborne = false;
                    }
                    hit_ray_idx = Some(ray_idx);
                    break;
                }
            }

            if let Some(ray_idx) = hit_ray_idx {
                self.record_surface_contact(ray_idx as i32, None);
            }
        }
    }

    fn record_surface_contact(
        &mut self,
        ray_idx: i32,
        prop: Option<WeakPropRef>,
    ) -> Option<SurfaceType> {
        let hit = self.raycast_state.get_closest_hit().unwrap_or_else(|| {
            panic_log!(
                "`Katamari::record_surface_contact`: tried to record a nonexistent surface contact"
            );
        });

        let mut normal_unit = hit.normal_unit.clone();
        vec3_inplace_zero_small(&mut normal_unit, 1e-05);

        // use the y component of the hit surface's unit normal to decide if it's a wall or floor
        let surface_type = if self.surface_threshold_y_normal < normal_unit[1] {
            if ray_idx == -1 || ray_idx == -5 || ray_idx == -6 {
                return None;
            }

            self.floor_contact_ray_idxs[self.num_floor_contacts as usize] = ray_idx as i8;
            self.physics_flags.contacts_floor = true;
            SurfaceType::Floor
        } else {
            self.physics_flags.contacts_wall = true;
            SurfaceType::Wall
        };

        let dot = vec3::dot(&normal_unit, &self.raycast_state.ray_unit);
        let ray_clip_len =
            (1.0 - hit.impact_dist_ratio - self.params.clip_len_constant) * hit.impact_dist;

        let mut proj = vec3::clone(&normal_unit);
        vec3_inplace_scale(&mut proj, dot * ray_clip_len);

        // TODO: `kat_record_surface_contact:84-104` (edge case)

        self.add_surface_contact(surface_type, &normal_unit, &proj, ray_idx, prop);

        Some(surface_type)
    }

    fn get_num_surface_contacts(&self, surface_type: SurfaceType) -> u8 {
        match surface_type {
            SurfaceType::Floor => self.num_floor_contacts,
            SurfaceType::Wall => self.num_wall_contacts,
        }
    }

    fn inc_num_surface_contacts(&mut self, surface_type: SurfaceType) {
        match surface_type {
            SurfaceType::Floor => self.num_floor_contacts += 1,
            SurfaceType::Wall => self.num_wall_contacts += 1,
        }
    }

    fn add_surface_contact(
        &mut self,
        surface_type: SurfaceType,
        normal_unit: &Vec3,
        clip_normal: &Vec3,
        ray_idx: i32,
        prop: Option<WeakPropRef>,
    ) {
        let num_contacts = self.get_num_surface_contacts(surface_type);

        let hit_surfaces = match surface_type {
            SurfaceType::Floor => &mut self.hit_floors,
            SurfaceType::Wall => &mut self.hit_walls,
        };

        let mut clip_normal = clip_normal.clone();
        vec3_inplace_zero_small(&mut clip_normal, 1e-05);
        let clip_normal_len = vec3::length(&clip_normal);

        // check the new surface's similarity to already contacted surfaces
        let mut old_surface = None;
        let mut found_old = false;
        if num_contacts > 0 {
            for surface in hit_surfaces.iter_mut() {
                let similarity = vec3::dot(&surface.normal_unit, &normal_unit);
                if similarity >= self.params.surface_similarity_threshold {
                    // if the old surface had a longer clip length, just return
                    // without doing anything at all.
                    if clip_normal_len <= surface.clip_normal_len {
                        return;
                    }

                    // if the added surface is too similar to an existing contact surface,
                    // we're not going to bother adding it as a "new" surface.
                    // but if the new surface's clip length is longer, we're going to
                    // update that old surface with the data that was passed to this call.
                    old_surface = Some(surface);
                    found_old = true;
                    break;
                }
            }
        }

        // depending on if we're using an old surface or a new one, get a reference
        // to the surface that we're going to write data to
        let mut added_surface = match old_surface {
            Some(old) => old,
            None => {
                let new_surface = SurfaceHit::default();
                hit_surfaces.push(new_surface);
                hit_surfaces.last_mut().unwrap()
            }
        };

        let closest_hit = self.raycast_state.get_closest_hit().unwrap();

        // write the new data to the added surface
        added_surface.normal_unit = normal_unit.clone();
        added_surface.clip_normal = clip_normal.clone();
        added_surface.ray = self.raycast_state.ray.clone();
        added_surface.contact_point = closest_hit.impact_point.clone();
        added_surface.clip_normal_len = clip_normal_len;
        added_surface.impact_dist_ratio = closest_hit.impact_dist_ratio;
        added_surface.ray_len = self.raycast_state.hit_dist;
        added_surface.ray_idx = ray_idx as u16;
        added_surface.hit_attr = closest_hit.metadata.into();
        added_surface.prop = prop;

        // maintain knowledge of the lowest floor contact point
        // (the simulation also tracked lowest wall contact point, but it was unused)
        let added_y = added_surface.contact_point[1];
        if surface_type == SurfaceType::Floor && added_y < self.lowest_floor_contact_y {
            self.lowest_floor_contact_y = added_y;
            self.lowest_floor_contact_point = added_surface.contact_point.clone();
        }

        if ray_idx >= 0 {
            self.collision_rays
                .get_mut(ray_idx as usize)
                .map(|ray| ray.contacts_surface = true);
        }

        if !found_old {
            self.inc_num_surface_contacts(surface_type);
        }
    }

    fn process_surface_contacts(&mut self) {
        self.physics_flags.on_flat_floor = false;
        self.physics_flags.on_sloped_floor = false;
        self.hit_flags.clear();

        if self.has_map_semi_translucent_hit {
            // propagate a `MapSemiTranslucent` hit to the hit flags.
            self.hit_flags.map_semi_translucent = true;
            self.hit_flags.special_camera = true;
        }

        // data about the collision ray that's nearest to contacting a floor.
        let mut fc_ray_idx = None;
        let mut fc_ray = None;
        let mut fc_contact_point = None;

        // process contact floors
        if self.num_floor_contacts == 0 {
            // if not contacting a floor:
            vec3::zero(&mut self.contact_floor_normal_unit);
            vec3::zero(&mut self.contact_floor_clip);
        } else {
            // if contacting at least one floor:

            let mut max_ray_len = 0.0;
            let mut sum_normal_unit = vec3::create();
            let mut sum_clip_normal = vec3::create();
            let mut min_ratio = 2.0; // can be anything bigger than 1.0

            for floor in self.hit_floors.iter() {
                // for each contacted floor:

                // record that we're contacting either a sloped or flat floor
                // based on the floor's y normal
                if floor.normal_unit[1] <= self.params.sloped_floor_y_normal_threshold {
                    self.physics_flags.on_sloped_floor = true;
                } else {
                    self.physics_flags.on_flat_floor = true;
                }

                // maintain running sum of all floor unit normals and clip normals
                vec3_inplace_add_vec(&mut sum_normal_unit, &floor.normal_unit);
                vec3_inplace_add_vec(&mut sum_clip_normal, &floor.clip_normal);

                // maintain the max contact floor ray length
                if floor.ray_len > max_ray_len {
                    max_ray_len = floor.ray_len;
                }

                // maintain a bunch of data about the contact floor with the minimum
                // impact distance ratio
                if floor.impact_dist_ratio < min_ratio {
                    min_ratio = floor.impact_dist_ratio;
                    fc_contact_point = Some(&floor.contact_point);
                    fc_ray = Some(&floor.ray);
                    fc_ray_idx = Some(floor.ray_idx);
                }

                // turn on hit flags based on the contact floor's hit attribute
                self.hit_flags.apply_hit_attr(floor.hit_attr);
            }

            self.fc_ray_idx = fc_ray_idx;
            // TODO: `kat_compute_minmax_floor_clip_normal()`
            // TODO: `kat_apply_surface_contacts:119-125` (verify what this is doing with cheat engine)
        }

        // process contact walls
        if self.num_wall_contacts == 0 {
            // if not contacting a wall:
            self.climb_radius_cm = self.radius_cm;
            vec3::zero(&mut self.contact_wall_normal_unit);
            vec3::zero(&mut self.contact_wall_clip);
        } else {
            // else if contacting at least one wall:

            let mut max_ray_len = 0.0;

            for wall in self.hit_walls.iter() {
                // TODO: `kat_apply_surface_contacts:145-152` (verify in cheat engine)
                if wall.ray_len > max_ray_len {
                    max_ray_len = wall.ray_len;
                }

                self.hit_flags.apply_hit_attr(wall.hit_attr);
            }

            // TODO: `kat_apply_surface_contacts:163-169` (verify in cheat engine)
            // TODO: `kat_compute_minmax_wall_clip_normal`
            self.climb_radius_cm = max_ray_len;
        }

        if fc_ray_idx.is_none()
            || self.physics_flags.vault_ray_type == Some(KatCollisionRayType::Bottom)
        {
            // if the primary floor contact point is the bottom ray
            self.fc_ray_idx = None;
            self.fc_ray = None;
            self.fc_contact_point = None;
            // TODO: is this right
            self.fc_ray_len = self.collision_rays[0].ray_len;
        } else if fc_ray_idx == self.vault_ray_idx {
            self.fc_ray_idx = self.vault_ray_idx;
        } else {
            self.fc_ray_idx = fc_ray_idx;
            self.fc_ray_len = self.collision_rays[fc_ray_idx.unwrap() as usize].ray_len;
            self.fc_ray = fc_ray.map(|v| v.clone());

            let mut contact_pt = vec3::create();
            vec3::scale_and_add(
                &mut contact_pt,
                &fc_contact_point.unwrap(),
                &fc_ray.unwrap(),
                0.005,
            );
            self.fc_contact_point = Some(contact_pt);
        }
    }
}
