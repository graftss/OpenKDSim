use gl_matrix::{common::Vec3, vec3};

use crate::{
    collision::raycast_state::RaycastCallType,
    macros::{panic_log, set_translation, set_y, temp_debug_log, vec3_from, vec3_unit_xz},
    math::{
        acos_f32, vec3_inplace_add, vec3_inplace_add_vec, vec3_inplace_normalize,
        vec3_inplace_scale, vec3_inplace_subtract, vec3_inplace_subtract_vec,
        vec3_inplace_zero_small, vec3_projection,
    },
    mission::{state::MissionState, GameMode},
    player::prince::Prince,
    props::prop::WeakPropRef,
};

use self::{hit::SurfaceHit, ray::KatCollisionRayType};

use super::Katamari;

pub mod history;
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
    pub fn update_collision(&mut self, prince: &Prince, mission_state: &MissionState) {
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
        if self.physics_flags.vs_mode_state == 2 {
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
            self.resolve_being_stuck();
            self.update_vault_and_climb(prince);

            if self.physics_flags.airborne && self.raycast_state.closest_hit_idx.is_some() {
                // TODO: `kat_update_collision:159-220` (update active hit before airborne?)
            }
        }

        // TODO: `kat_update_collision:222-266` (destroy collected props that are sucked inside the ball)
        // TODO: `kat_process_props_inside_sphere()`
        // TODO: `kat_process_collected_props()`
        // TODO: `kat_update_world_size_threshold??()`

        if self.physics_flags.grounded_ray_type == Some(KatCollisionRayType::Bottom)
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
        vec3_inplace_subtract(&mut below, 0.0, dist_down, 0.0);
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
                // TODO: replace this when shell points are working
                continue;

                // check collisions along each shell ray
                self.raycast_state.load_ray(point0, point1);
                self.raycast_state
                    .find_nearest_unity_hit(RaycastCallType::Objects, false);

                // TODO: `kat_update_surface_contacts:308-372` (resolve shell ray hits)
            }

            // check collision rays for surface contacts
            let center = self.center;
            for (ray_idx, ray) in self.collision_rays.iter().enumerate() {
                self.raycast_state.load_ray(&center, &ray.endpoint);
                let found_hit = self
                    .raycast_state
                    .find_nearest_unity_hit(RaycastCallType::Objects, false);
                // temp_debug_log!(
                //     "ray_idx={ray_idx}, endpt={:?}, len={:?}, found_hit={found_hit}",
                //     ray.endpoint,
                //     ray.ray_len
                // );

                if found_hit {
                    // TODO: there's a flag being ignored here
                    if true && !self.last_physics_flags.airborne {
                        self.physics_flags.airborne = false;
                    }
                    self.record_surface_contact(ray_idx as i32, None);
                }

                // TODO: break to only cast the bottom ray
                break;
            }
        }
    }

    /// offset: 0x133d0
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
        let surface_type = if self.params.surface_normal_y_threshold < normal_unit[1] {
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
                if floor.normal_unit[1] <= self.params.sloped_floor_normal_y_threshold {
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
            || self.physics_flags.grounded_ray_type == Some(KatCollisionRayType::Bottom)
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

    fn resolve_being_stuck(&mut self) {
        self.hit_history.push(
            self.num_wall_contacts,
            self.num_floor_contacts,
            &self.contact_wall_normal_unit,
            &self.contact_floor_normal_unit,
        );

        // TODO: this control flow
        let stuck_btwn_walls = if self.hit_flags.small_ledge_climb {
            true
        } else if self.num_wall_contacts == 0 {
            // if no wall contacts, not stuck
            false
        } else if self.num_wall_contacts == 1 {
            if self.num_floor_contacts == 0 {
                false
            } else {
                // contacting exactly 1 wall and at least 1 floor:

                let wall_dot_floor = vec3::dot(
                    &self.contact_wall_normal_unit,
                    &self.contact_floor_normal_unit,
                );
                // TODO: technically this needs to be an `acos` of doubles
                let angle = acos_f32(wall_dot_floor);

                // stuck if the wall-to-floor angle is over the threshold parameter and the katamari moved.
                angle > self.params.wall_to_floor_angle_stuck_threshold
                    && !vec3::exact_equals(&self.center, &self.last_center)
            }
        } else if self.num_wall_contacts == 2 {
            // contacting exactly 2 walls:
            let wall_dot_wall = vec3::dot(
                &self.hit_walls[0].normal_unit,
                &self.hit_walls[1].normal_unit,
            );

            if wall_dot_wall > self.params.wall_to_wall_angle_stuck_threshold {
                // stuck if the angle between the walls is beyond the threshold param
                self.lose_props_when_stuck();
                true
            } else {
                // also stuck if the katamari's was stuck on the previous tick
                self.stuck_ticks > 0
            }
        } else {
            // contacting 3 or more walls:
            // always stuck
            self.lose_props_when_stuck();
            true
        };

        // TODO: `kat_update_stuckness:304-335` (a case that's too annoying right now)

        if stuck_btwn_walls {
            // if stuck between walls, try to push the katamari away from the wall
            // (i.e. push it in the direction of the wall's normal).
            vec3::copy(
                &mut self.stuck_btwn_walls_push_unit,
                &self.contact_wall_normal_unit,
            );
            self.stuck_ticks += 1;
            self.physics_flags.stuck_between_walls = true;

            // also detach props every so often, because why not? maybe that'll help.
            // what the hell do i know? fuck it!
            if self.stuck_ticks > self.params.detach_cooldown_when_stuck_btwn_walls {
                // TODO: (line 352) needs camera+global state here
                if !self.physics_flags.detaching_props {
                    self.static_detaching_props = true;
                    // TODO: (line 358) needs gametype != C here
                    if self.can_detach_props {
                        let lost_vol_mult = self.params.base_detached_prop_vol_mult * 0.5;
                        self.physics_flags.detaching_props = true;
                        self.detach_props(lost_vol_mult * self.vol_m3, 0.5);
                    }
                    self.static_detaching_props = false;
                }

                self.stuck_ticks = 1;
            }
        } else {
            // if not stuck between walls:
            // reset the stuckness state
            self.stuck_ticks = 0;
            self.physics_flags.stuck_between_walls = false;
        }
    }

    /// TODO
    /// offset: 0x17790
    fn lose_props_when_stuck(&mut self) {}

    /// TODO
    /// offset: 0x26f10
    fn detach_props(&mut self, _vol: f32, _prop_speed: f32) {}

    /// (??) Update the katamari's vault and climbing state.
    /// offset: 0x14c80
    fn update_vault_and_climb(&mut self, prince: &Prince) {
        if self.physics_flags.hit_shell_ray == Some(ray::ShellRay::TopCenter)
            && self.physics_flags.contacts_floor
            && !self.physics_flags.contacts_wall
            && self.num_floor_contacts == 1
        {
            self.physics_flags.contacts_floor = false;
            vec3::zero(&mut self.contact_floor_clip);
        }

        // TODO: `kat_apply_turntable_contact()`
        self.update_clip_translation();

        if self.physics_flags.in_water
            && !self.last_physics_flags.in_water
            && self.physics_flags.airborne
        {
            // TODO: `kat_update_vault_and_climb:44` (play enter water sfx)
        }

        if self.physics_flags.grounded_ray_type != Some(KatCollisionRayType::Bottom) {
            vec3_inplace_subtract_vec(&mut self.vault_contact_point, &self.contact_wall_clip);
        }

        self.physics_flags.unknown_0x22 = false;
        'main: {
            if self.num_wall_contacts + self.num_floor_contacts == 0 {
                // if the katamari isn't contacting any surfaces
                if !self.physics_flags.climbing_wall {
                    // if not climbing a wall, reset wallclimb state
                    self.end_wallclimb();
                } else if self.is_climbing_0x898 < 1 {
                    self.wallclimb_cooldown_timer = 10;
                    self.wallclimb_ticks = 0;
                    self.end_wallclimb();
                } else {
                    self.is_climbing_0x898 -= 1;
                    if self.physics_flags.at_max_climb_height && prince.input_avg_push_len > 0.99 {
                        break 'main;
                    }
                }

                self.fc_ray_len = self.radius_cm;
                self.physics_flags.grounded_ray_type = Some(KatCollisionRayType::Bottom);
                self.vault_ray_idx = None;

                if !self.physics_flags.airborne {
                    self.airborne_ticks = 0;
                    self.falling_ticks = 0;
                } else {
                    self.airborne_ticks += 1;
                    if self.is_falling() {
                        self.falling_ticks += 1;
                    }
                }

                self.physics_flags.airborne = true;
                if self.physics_flags.climbing_wall {
                    self.airborne_ticks += 1;
                    if self.is_falling() {
                        self.falling_ticks += 1;
                    }
                    self.physics_flags.airborne = false;
                    panic_log!("??? why is this here");
                }
            } else {
                // if contacting a surface:
                self.physics_flags.unknown_0x20 = false;
                // TODO: `self.update_bonks()`
                if !self.physics_flags.contacts_floor
                    && self.physics_flags.contacts_wall
                    && !self.physics_flags.climbing_wall
                {
                    self.update_airborne_timers(true);
                } else {
                    self.update_airborne_timers(false);
                }

                // compute the projection and rejection of the katamari's velocity onto its contacted floor
                vec3_projection(
                    &mut self.velocity.vel_proj_floor,
                    &mut self.velocity.vel_rej_floor,
                    &self.velocity.velocity_unit,
                    &self.contact_floor_normal_unit,
                );

                match self.try_init_vault_speed() {
                    0 => return self.set_bottom_ray_contact(),
                    // TODO: `kat_update_vault_and_climb:144-272
                    _ => (),
                }
            }
        }

        if self.physics_flags.airborne {
            if self.physics_flags.climbing_wall && self.is_climbing_0x898 > 0 {
                self.is_climbing_0x898 -= 1;
            } else {
                if self.physics_flags.climbing_wall {
                    self.wallclimb_cooldown_timer = 10;
                    self.wallclimb_ticks = 0;
                }

                self.end_wallclimb();
            }
        }

        if self.physics_flags.unknown_0x22 {
            if !self.physics_flags.in_water {
                let play_hit_ground_sfx = match self.physics_flags.grounded_ray_type {
                    Some(KatCollisionRayType::Bottom) => true,
                    Some(KatCollisionRayType::Prop) => false,
                    Some(KatCollisionRayType::Mesh) => {
                        self.collision_rays[self.vault_ray_idx.unwrap() as usize].ray_len
                            / self.radius_cm
                            > 1.05
                    }
                    None => false,
                };

                if play_hit_ground_sfx {
                    // TODO: `kat_update_vault_and_climb:307` (play HIT_GROUND_FROM_FALL sfx)
                }
            }
        }
    }

    fn try_init_vault_speed(&mut self) -> i32 {
        return 0;
    }

    fn end_wallclimb(&mut self) {
        self.physics_flags.climbing_wall = false;
        self.physics_flags.at_max_climb_height = false;
        self.wallclimb_init_y = 0.0;
        self.wallclimb_max_height_ticks = 0;
    }

    fn update_airborne_timers(&mut self, new_airborne: bool) {
        if !self.physics_flags.airborne {
            self.airborne_ticks = 0;
            self.falling_ticks = 0;
        } else {
            self.airborne_ticks += 1;
            if self.is_falling() {
                self.falling_ticks += 1;
            }
        }
        self.physics_flags.airborne = new_airborne;
    }

    fn is_falling(&self) -> bool {
        self.velocity.accel_grav[1] + self.velocity.vel_accel[1] < 0.0
    }

    /// (??) I think this is trying to clip the katamari away from walls when the game
    /// thinks that it's stuck.
    /// TODO: there are some SHUFPS instructions here that might not be decompiled correctly.
    /// offset: 0x183f0
    fn update_clip_translation(&mut self) {
        let moving_into_wall = if self.physics_flags.contacts_wall {
            if self.physics_flags.immobile {
                // if contacting a wall and immobile, i guess we're moving into a wall
                true
            } else {
                // compute unit lateral katamari movement
                let move_xz = vec3_from!(-, self.center, self.last_center);
                let move_xz_unit = vec3_unit_xz!(move_xz);

                // compute unit lateral wall normal
                let wall_normal_xz_unit = vec3_unit_xz!(self.contact_wall_normal_unit);

                // the katamari is moving towards the wall if its velocity dot the wall normal is
                // below the similarity threshold (since the wall normal should be pointing away
                // from katamari movement, any negative similarity should work)
                vec3::dot(&move_xz_unit, &wall_normal_xz_unit)
                    <= self.params.move_into_wall_similarity
            }
        } else {
            // if the katamari doesn't contact a wall, it's not moving into one either.
            false
        };

        if self.physics_flags.stuck_between_walls && !self.last_physics_flags.stuck_between_walls {
            // if the katamari has just gotten stuck between walls:
            // push the katamari away from the wall to try to get unstuck
            let push_distance = self.scaled_params.base_max_speed * self.params.unstuck_bump_speed;
            let mut push_velocity = self.stuck_btwn_walls_push_unit;
            vec3_inplace_scale(&mut push_velocity, push_distance);

            // forcibly set the katamari's velocity to push it away from the wall
            self.set_velocity(&push_velocity);
        }

        // TODO: double check these shufps vector operations
        vec3::zero(&mut self.clip_translation);
        vec3_inplace_add_vec(&mut self.clip_translation, &self.contact_floor_clip);
        if moving_into_wall {
            vec3_inplace_add_vec(&mut self.clip_translation, &self.contact_wall_clip);
        }

        if !self.physics_flags.stuck_between_walls {
            // if not stuck between walls:
            if self.physics_flags.contacts_down_slanted_ceiling
                && self.physics_flags.contacts_floor
                && !self.physics_flags.contacts_prop_0xa
            {
                // weird collision edge case where we're not stuck between walls but we are contacting
                // a down-slanted wall/ceiling. no clue what to make of this
                if !self.last_physics_flags.contacts_down_slanted_ceiling {
                    vec3::normalize(&mut self.delta_pos_unit, &self.delta_pos);
                }
                let mut clip_translation = self.delta_pos_unit;
                vec3_inplace_scale(&mut clip_translation, -self.speed);
                self.set_velocity(&clip_translation);
                vec3_inplace_add_vec(&mut self.center, &clip_translation);
                self.clip_translation = clip_translation;
            }
        } else {
            // if stuck between walls:
            self.clip_translation[0] += self.center[0] - self.last_center[0];
            self.clip_translation[2] += self.center[2] - self.last_center[2];
            vec3_inplace_subtract_vec(&mut self.center, &self.clip_translation);
        }

        // update center, bottom, and top points
        set_translation!(self.transform, self.center);
        self.bottom = self.center;
        self.bottom[1] -= self.radius_cm;
    }
}
