use gl_matrix::{common::Vec3, mat4, vec3};

use crate::{
    collision::raycast_state::RaycastCallType,
    constants::{FRAC_PI_2, FRAC_PI_90, VEC3_Y_NEG},
    global::GlobalState,
    macros::{
        inv_lerp, inv_lerp_clamp, lerp, max, min, panic_log, set_translation, set_y,
        temp_debug_log, vec3_from, vec3_unit_xz,
    },
    math::{
        self, acos_f32, vec3_inplace_add, vec3_inplace_add_scaled, vec3_inplace_add_vec,
        vec3_inplace_normalize, vec3_inplace_scale, vec3_inplace_subtract,
        vec3_inplace_subtract_vec, vec3_inplace_zero_small, vec3_projection, vec3_reflection,
    },
    mission::{self, state::MissionState, GameMode},
    player::{camera::Camera, prince::Prince},
    props::{
        config::NAME_PROP_CONFIGS,
        prop::{Prop, PropGlobalState, WeakPropRef},
        Props,
    },
    util::debug_log,
};

use self::{hit::SurfaceHit, ray::KatCollisionRayType};

use super::{flags::KatInclineMoveType, Katamari};

pub mod history;
pub mod hit;
pub mod mesh;
pub mod ray;

/// All surface triangles can be categorized as either a floor or a wall. They
/// are distinguished via the y component of their unit normal vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceType {
    Floor,
    Wall,
}

/// The three possible results returned by `Katamari::try_init_vault`
pub enum TryInitVaultResult {
    /// The katamari is not vaulting.
    NoVault,

    /// The katamari just started a new vault.
    InitVault,

    /// The katamari is vontinuing a vault from a previous tick.
    OldVault,
}

impl Katamari {
    /// The main function to update the katamari's collision state.
    pub fn update_collision(
        &mut self,
        prince: &mut Prince,
        camera: &Camera,
        global: &GlobalState,
        mission_state: &MissionState,
        props: &mut Props,
    ) {
        self.last_num_floor_contacts = self.num_floor_contacts;
        self.last_num_wall_contacts = self.num_wall_contacts;
        self.num_floor_contacts = 0;
        self.hit_floors.clear();
        self.num_wall_contacts = 0;
        self.hit_walls.clear();
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
        if self.ignore_prop_collision_timer > 0 {
            self.ignore_prop_collision_timer -= 1;
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
        self.physics_flags.moved_more_than_rad_0x14 = false;

        // TODO_VS: `kat_update_collision:96-101` (decrement timer)

        self.find_nearby_props(props);

        if mission_state.gamemode == GameMode::Ending {
            // TODO_ENDING: `kat_update_collision:105-132 (ending-specific reduced collision)
        } else {
            // TODO: `kat_update_water_contact()`
            // self.debug_log_clip_data("0x1302a");
            self.update_surface_contacts();
            // self.debug_log_clip_data("0x1303a");
            self.process_surface_contacts();
            // self.debug_log_clip_data("0x13042");
            self.resolve_being_stuck();
            self.update_vault_and_climb(prince, camera, global);

            if self.physics_flags.airborne && self.raycast_state.closest_hit_idx.is_some() {
                // TODO: `kat_update_collision:159-220` (update active hit before airborne?)
            }
        }

        // TODO: `kat_update_collision:222-266` (destroy collected props that are sucked inside the ball)
        self.process_nearby_collectible_props(mission_state);
        // TODO: `kat_process_collected_props()`
        // TODO: `kat_update_world_size_threshold??()`

        if self.physics_flags.grounded_ray_type == Some(KatCollisionRayType::Bottom)
            || self.fc_ray_len < 1.0
        {
            self.fc_ray_len = self.radius_cm;
        }
    }

    /// Iterate over all props to find those which are "nearby" the katamari.
    /// In the original implementation, "nearby" means the prop-katamari distance is
    /// less than the sum of (1) the radii of the bounding spheres of the katamari and the prop,
    /// and (2) the distance that the katamari moved over the last frame.
    /// offset: 0x28870
    fn find_nearby_props(&mut self, props: &mut Props) {
        // TODO_VS: `kat_find_nearby_props:43` (return immediately if vs mode or if other vs condition holds)

        // TODO_PARAM: make this a global parameter
        let MAX_COLLECTION_CHECKS_PER_FRAME = 0x80;

        // compute the distance the katamari moved since the last frame
        let kat_move = vec3_from!(-, self.center, self.last_center);
        let kat_move_len = vec3::length(&kat_move);
        let mut lateral_move_unit = [kat_move[0], 0.0, kat_move[2]];
        vec3_inplace_normalize(&mut lateral_move_unit);

        self.nearby_collectible_props.clear();
        self.collected_props.clear();

        self.contact_prop = None;

        if self.ignore_prop_collision_timer != 0 {
            return;
        }

        for prop_ref in props.props.iter_mut() {
            let mut prop = prop_ref.borrow_mut();

            // return early from the collision check if the prop is still intangible.
            if prop.intangible_timer > 0 {
                return prop.intangible_timer -= 1;
            }

            if prop.scream_cooldown_timer > 0 {
                prop.scream_cooldown_timer -= 1;
            }

            prop.near_player = false;

            // more early return conditions where collision with the katamari isn't checked
            if prop.global_state != PropGlobalState::Unattached
                || prop.is_disabled()
                || prop.force_intangible
            {
                return;
            }

            // The prop-katamari distance decreased at most by the distance the katamari just moved.
            // If that minimum distance is still bigger than the sum of the kat's and prop's bounding
            // spheres, they can't collide.
            let min_dist_to_kat = max!(prop.get_dist_to_katamari(0) - kat_move_len, 0.0);
            if min_dist_to_kat > self.radius_cm + prop.get_radius() {
                return;
            }

            let prop_config = NAME_PROP_CONFIGS.get(prop.get_name_idx() as usize).unwrap();
            let collectible =
                self.diam_trunc_mm >= prop.get_attach_diam_mm() && !prop_config.is_dummy_hit;
            // TODO_VS: `kat_find_nearby_props:105-111` (different vol required to collect props in vs mode)

            // if the prop and katamari sphere might meet AND the prop is collectible, save this
            // prop for later to fully check if it should be collected.
            if collectible {
                self.nearby_collectible_props.push(prop_ref.clone());
                if self.nearby_collectible_props.len() >= MAX_COLLECTION_CHECKS_PER_FRAME {
                    return;
                }
                continue;
            }

            // otherwise, the prop is nearby, but uncollectible.
            prop.near_player = true;

            // TODO_PROPS: `kat_check_prop_mesh_collision()` instead of `false` below
            let did_collide = false;
            if did_collide {
                // TODO_PROPS: `kat_find_nearby_props:138-221`
            }
        }
    }

    /// offset: 0x28640
    fn process_nearby_collectible_props(&mut self, mission_state: &MissionState) {
        // TODO_PARAM
        let SQUASH_PROP_VOL_MULTIPLIER = 3.0;
        let MAX_COLLECTED_PROPS_PER_FRAME = 0x40;

        if mission_state.is_ending() {
            // TODO_ENDING: `kat_process_nearby_collectible_props:13-33`
        } else {
            for prop_ref in self.nearby_collectible_props.iter_mut() {
                let prop = prop_ref.borrow_mut();
                let prop_config = NAME_PROP_CONFIGS.get(prop.get_name_idx() as usize).unwrap();

                let link_cond = prop.parent.is_none() || prop.get_flags() & 4 == 0;
                let is_dummy = prop_config.is_dummy_hit;
                let did_collide = false; // TODO: `kat_intersects_prop_bbox()`
                if link_cond && !is_dummy && did_collide {
                    // if the katamari collided with the prop's bbox:
                    let can_prop_be_airborne = prop.get_move_type().is_some()
                        && !prop.get_stationary()
                        && prop_config.can_be_airborne;
                    let is_prop_squashed = self.max_attach_vol_m3
                        > SQUASH_PROP_VOL_MULTIPLIER * prop.get_compare_vol_m3();

                    if can_prop_be_airborne && !is_prop_squashed {
                        // TODO_AIRBORNE: `kat_init_prop_launch()`
                    } else {
                        self.collected_props.push(prop_ref.clone());
                        if self.collected_props.len() >= MAX_COLLECTED_PROPS_PER_FRAME {
                            return;
                        }
                        if let Some(delegates) = &self.delegates {
                            if let Some(log_prop_collected) = delegates.borrow().log_prop_collected
                            {
                                log_prop_collected(prop.get_ctrl_idx() as i32);
                            }
                        }
                    }
                }
            }
        }
    }

    fn intersects_prop_bbox(&mut self, prop: &Prop, mission_state: &MissionState) -> bool {
        // if the prop-player distance is less than the sum of radii, we must have a collision.
        if prop.get_dist_to_katamari(0) < prop.get_aabb_radius() + self.radius_cm {
            return true;
        }

        let prop_config = NAME_PROP_CONFIGS.get(prop.get_name_idx() as usize).unwrap();

        let mut kat_sphere_rad = if prop_config.easier_to_collect {
            self.avg_mesh_ray_len
        } else {
            self.larger_avg_mesh_ray_len
        };

        if mission_state.is_ending() {
            kat_sphere_rad += kat_sphere_rad;
        }

        // compute the unit vector from the prop to the katamari
        let mut prop_pos = vec3::create();
        prop.get_pos(&mut prop_pos);
        let mut prop_to_kat_unit = vec3_from!(-, self.center, prop_pos);
        vec3_inplace_normalize(&mut prop_to_kat_unit);

        // compute the ray from the katamari's center towards the prop, with length `kat_sphere_rad`.
        let kat_center = self.center.clone();
        let mut ray_endpoint = vec3::create();
        vec3::scale_and_add(
            &mut ray_endpoint,
            &kat_center,
            &prop_to_kat_unit,
            -kat_sphere_rad,
        );

        self.raycast_state.load_ray(&kat_center, &ray_endpoint);
        // TODO: `collision_segment_hits_meshes(prop.aabb_mesh, prop.unattached_transform, false)`
        false
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

            vec3::copy(&mut shell_initial_pts[0], &self.shell_top);
            vec3_inplace_add(
                &mut shell_initial_pts[0],
                last_center[0],
                0.0,
                last_center[2],
            );

            vec3::copy(&mut shell_final_pts[0], &self.shell_top);
            vec3_inplace_add_vec(&mut shell_final_pts[0], &shell_ray_vec);

            // TODO: `kat_update_surface_contacts:267-294` (support all shell points)

            for (_i, (_point0, _point1)) in
                Iterator::zip(shell_initial_pts.iter(), shell_final_pts.iter()).enumerate()
            {
                // check collisions along each shell ray
                self.raycast_state.load_ray(_point0, _point1);
                self.raycast_state
                    .find_nearest_unity_hit(RaycastCallType::Objects, false);

                // TODO: replace this when shell points are working
                break;

                // TODO: `kat_update_surface_contacts:308-372` (resolve shell ray hits)
            }

            // check collision rays for surface contacts
            let center = self.center;
            let rays = &self.collision_rays.clone();
            for (ray_idx, ray) in rays.iter().enumerate() {
                self.raycast_state.load_ray(&center, &ray.endpoint);
                let found_hit = self
                    .raycast_state
                    .find_nearest_unity_hit(RaycastCallType::Objects, false);

                if found_hit {
                    // TODO: there's a flag being ignored here
                    if true && !self.last_physics_flags.airborne {
                        self.physics_flags.airborne = false;
                    }
                    self.record_surface_contact(ray_idx as i32, None);
                }
            }
        }
    }

    /// Returns `None` if the surface contact arose from a shell ray.
    /// If not, then the surface contact arose from a collision ray and the returned value is
    /// type of surface that is contacted (either wall or floor).
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

        let mut impact_unit = hit.normal_unit;
        vec3_inplace_zero_small(&mut impact_unit, 1e-05);

        // use the y component of the hit surface's unit normal to decide if it's a wall or floor
        let surface_type = if self.params.surface_normal_y_threshold < impact_unit[1] {
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

        let dot = vec3::dot(&impact_unit, &self.raycast_state.ray_unit);
        let ray_clip_len = (1.0 - hit.impact_dist_ratio - self.params.clip_len_constant)
            * self.raycast_state.ray_len;
        // temp_debug_log!("ray_idx={}, dot={}, impact_unit={:?}, ray_unit={:?}", ray_idx, dot, impact_unit, self.raycast_state.ray_unit);
        // temp_debug_log!("ray_clip_len={:?}, impact_dist_ratio={:?}, clip_len_const={:?}, impact_dist={:?}", ray_clip_len, hit.impact_dist_ratio, self.params.clip_len_constant, self.raycast_state.ray_len);

        let mut clip_normal = vec3::clone(&impact_unit);
        vec3_inplace_scale(&mut clip_normal, dot * ray_clip_len);

        // temp_debug_log!("   impact_point:{:?}, dist_ratio:{}", hit.impact_point, hit.impact_dist_ratio);
        // temp_debug_log!("   ray={:?}, p0={:?}, p1={:?}", self.raycast_state.ray, self.raycast_state.point0, self.raycast_state.point1);
        // temp_debug_log!("   dot:{dot}, ray_clip_len={ray_clip_len}, clip_normal: {clip_normal:?}");

        if impact_unit[1] < -0.1 {
            let normal_angle_y = acos_f32(impact_unit[1]);
            if normal_angle_y < 1.047198 {
                self.physics_flags.contacts_down_slanted_ceiling = true;
                if self.physics_flags.contacts_prop_0xa {
                    set_y!(clip_normal, 0.0);
                }
            }
        }

        self.add_surface_contact(surface_type, &impact_unit, &clip_normal, ray_idx, prop);

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
        added_surface.ray_len = self.raycast_state.ray_len;
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
            // temp_debug_log!("   new surface contact: type={surface_type:?}, ray_idx={ray_idx}");
            self.inc_num_surface_contacts(surface_type);
        }
    }

    /// offset: 0x13930
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
        let mut min_ratio_ray_idx = None;
        let mut min_ratio_ray = None;
        let mut min_ratio_contact_pt = None;

        // process contact floors
        if self.num_floor_contacts == 0 {
            // if not contacting a floor:
            vec3::zero(&mut self.contact_floor_normal_unit);
            vec3::zero(&mut self.contact_floor_clip);
        } else {
            // if contacting at least one floor:

            let mut max_contact_ray_len = 0.0;
            let mut min_ratio = 2.0; // can be anything bigger than 1.0
            let mut sum_floor_normals = vec3::create();
            let mut sum_clip_normal = vec3::create();

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
                vec3_inplace_add_vec(&mut sum_floor_normals, &floor.normal_unit);
                vec3_inplace_add_vec(&mut sum_clip_normal, &floor.clip_normal);

                // maintain the max contact floor ray length
                if floor.ray_len > max_contact_ray_len {
                    max_contact_ray_len = floor.ray_len;
                }

                // maintain a bunch of data about the contact floor with the minimum
                // impact distance ratio
                if floor.impact_dist_ratio < min_ratio {
                    min_ratio = floor.impact_dist_ratio;
                    min_ratio_contact_pt = Some(floor.contact_point);
                    min_ratio_ray = Some(floor.ray);
                    min_ratio_ray_idx = Some(floor.ray_idx);
                }

                // turn on hit flags based on the contact floor's hit attribute
                self.hit_flags.apply_hit_attr(floor.hit_attr);
            }

            self.fc_ray_idx = min_ratio_ray_idx;
            self.compute_contact_floor_clip();

            // TODO: the following line is possibly wrong
            vec3::normalize(&mut self.contact_floor_normal_unit, &sum_floor_normals);
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
            let mut sum_wall_normals = vec3::create();

            for wall in self.hit_walls.iter() {
                // TODO: `kat_apply_surface_contacts:145-152` (verify in cheat engine)
                if wall.ray_len > max_ray_len {
                    max_ray_len = wall.ray_len;
                }

                vec3_inplace_add_vec(&mut sum_wall_normals, &wall.normal_unit);
                self.hit_flags.apply_hit_attr(wall.hit_attr);
            }

            // TODO: `kat_apply_surface_contacts:163-169` (verify in cheat engine)
            vec3::normalize(&mut self.contact_wall_normal_unit, &sum_wall_normals);
            self.compute_contact_wall_clip();
            self.climb_radius_cm = max_ray_len;
        }

        if min_ratio_ray_idx.is_none()
            || self.physics_flags.grounded_ray_type == Some(KatCollisionRayType::Bottom)
        {
            // if the primary floor contact point is from the bottom ray:
            self.fc_ray_idx = min_ratio_ray_idx;
            self.fc_ray = min_ratio_ray;
            self.fc_contact_point = min_ratio_contact_pt;

            self.fc_ray_len = if min_ratio_ray_idx.is_none() {
                self.climb_radius_cm
            } else {
                self.collision_rays[0].ray_len
            };
        } else if min_ratio_ray_idx == self.vault_ray_idx {
            // if the primary floor contact point is from the vault ray:
            self.fc_ray_idx = self.vault_ray_idx;
        } else {
            // if the primary floor contact point is from a non-bottom, non-vault ray:
            self.fc_ray_idx = min_ratio_ray_idx;
            self.fc_ray = min_ratio_ray;
            self.fc_ray_len = self.collision_rays[min_ratio_ray_idx.unwrap() as usize].ray_len;

            let mut contact_pt = vec3::create();
            vec3::scale_and_add(
                &mut contact_pt,
                &min_ratio_contact_pt.unwrap(),
                &min_ratio_ray.unwrap(),
                0.005,
            );
            self.fc_contact_point = Some(contact_pt);
        }
    }

    /// offset: 0x169f0
    fn compute_contact_floor_clip(&mut self) {
        // compute the smallest and largest x, y, and z coordinates over all
        // contacted floor surfaces
        let mut min_clip_coords = [0.0; 3];
        let mut max_clip_coords = [0.0; 3];
        if self.num_floor_contacts > 0 {
            // if the player contacts a floor:
            for floor in self.hit_floors.iter() {
                let clip = floor.clip_normal;
                for i in 0..2 {
                    if clip[i] > max_clip_coords[i] {
                        max_clip_coords[i] = clip[i];
                    } else if clip[i] < min_clip_coords[i] {
                        min_clip_coords[i] = clip[i];
                    }
                }
            }
        }

        vec3::add(
            &mut self.contact_floor_clip,
            &min_clip_coords,
            &max_clip_coords,
        );
        vec3_inplace_zero_small(&mut self.contact_floor_clip, 0.0001);
    }

    /// offset: 0x16b80
    fn compute_contact_wall_clip(&mut self) {
        // compute the smallest and largest x, y, and z coordinates over all
        // contacted floor surfaces
        let mut min_clip_coords = [0.0; 3];
        let mut max_clip_coords = [0.0; 3];
        if self.num_wall_contacts > 0 {
            // if the player contacts a floor:
            for wall in self.hit_walls.iter() {
                let clip = wall.clip_normal;
                for i in [0, 1, 2] {
                    if clip[i] > max_clip_coords[i] {
                        max_clip_coords[i] = clip[i];
                    } else if clip[i] < min_clip_coords[i] {
                        min_clip_coords[i] = clip[i];
                    }
                }
            }
        }

        vec3::add(
            &mut self.contact_wall_clip,
            &min_clip_coords,
            &max_clip_coords,
        );
        vec3_inplace_zero_small(&mut self.contact_wall_clip, 0.0001);
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
                // TODO_LOW: technically this needs to be an `acos` of doubles
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
    fn update_vault_and_climb(
        &mut self,
        prince: &mut Prince,
        camera: &Camera,
        global: &GlobalState,
    ) {
        if self.physics_flags.hit_shell_ray == Some(ray::ShellRay::TopCenter)
            && self.physics_flags.contacts_floor
            && !self.physics_flags.contacts_wall
            && self.num_floor_contacts == 1
        {
            self.physics_flags.contacts_floor = false;
            vec3::zero(&mut self.contact_floor_clip);
        }

        // TODO: `kat_apply_turntable_contact()`
        self.apply_clip_translation();

        if self.physics_flags.in_water
            && !self.last_physics_flags.in_water
            && self.physics_flags.airborne
        {
            // TODO_FX: `kat_update_vault_and_climb:44` (play enter water sfx)
        }

        if self.physics_flags.grounded_ray_type != Some(KatCollisionRayType::Bottom) {
            vec3_inplace_subtract_vec(&mut self.vault_contact_point, &self.contact_wall_clip);
        }

        self.physics_flags.just_hit_ground_hard = false;
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
                self.update_wall_contacts(prince, camera, global);
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
                    &self.velocity.vel_unit,
                    &self.contact_floor_normal_unit,
                );

                match self.try_init_vault() {
                    // case 1: the katamari isn't vaulting
                    TryInitVaultResult::NoVault => return self.set_bottom_ray_contact(),

                    // case 2: the katamari is starting a new vault
                    TryInitVaultResult::InitVault => {
                        let ray_idx = self.vault_ray_idx.unwrap();
                        let ray_type = self.ray_type_by_idx(ray_idx);
                        self.physics_flags.grounded_ray_type = ray_type;

                        if ray_type == Some(KatCollisionRayType::Prop) {
                            // if the vault ray is from a prop:
                            // update `prop_vault_ray`
                            let ray = &self.collision_rays[ray_idx as usize];
                            vec3::scale(&mut self.prop_vault_ray, &ray.ray_local_unit, ray.ray_len);
                        }

                        // reset the `vault_transform` to the identity
                        mat4::identity(&mut self.vault_transform);

                        // save a copy of the katamari's transform when the vault started
                        mat4::copy(&mut self.vault_init_transform, &self.transform);

                        // set the vault floor normal to the contact floor normal
                        vec3::copy(
                            &mut self.vault_floor_normal_unit,
                            &self.contact_floor_normal_unit,
                        );

                        // (??) stretch the floor contact ray that's being vaulted on
                        let mut fc_ray = self.fc_ray.unwrap();
                        vec3_inplace_scale(&mut fc_ray, self.params.vault_ray_stretch);
                        self.fc_ray = Some(fc_ray);

                        // Readjust the vault contact point to where the stretched ray ends
                        vec3::add(&mut self.vault_contact_point, &self.center, &fc_ray);
                        vec3_inplace_zero_small(&mut self.vault_contact_point, 0.0001);

                        self.vault_ticks = 0;

                        let ray = self
                            .collision_rays
                            .get(self.vault_ray_idx.unwrap() as usize)
                            .unwrap();
                        let ray_len_t = inv_lerp!(ray.ray_len, self.radius_cm, self.max_ray_len);
                        if 0.3 <= ray_len_t {
                            // TODO_FX: play VAULTING sfx with volume `ray_len_t`
                        }
                    }

                    // case 3: continuing a vault that was initialized on a previous tick
                    TryInitVaultResult::OldVault => {
                        if let Some(ray_idx) = self.vault_ray_idx {
                            // update the grounded ray type based on the vaulted ray's index
                            self.physics_flags.grounded_ray_type = self.ray_type_by_idx(ray_idx);

                            // (??) i guess this is pushing the katamari up out of the ground
                            // if the vault ray is clipped too far into the ground?
                            if self.clip_translation[1] <= -1.0 {
                                self.vault_contact_point[1] -= self.clip_translation[1];
                            }
                        }
                    }
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

        if self.physics_flags.just_hit_ground_hard {
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
                    // TODO_FX: `kat_update_vault_and_climb:307` (play HIT_GROUND_FROM_FALL sfx)
                }
            }
        }
    }

    /// offset: 0x15950
    fn update_wall_contacts(&mut self, prince: &mut Prince, camera: &Camera, global: &GlobalState) {
        if self.physics_flags.climbing_wall {
            // TODO_CLIMB: `kat_update_wall_contacts:47-58`
            if self.can_climb_wall_contact(prince) {
                return self.update_wallclimb();
            }

            return self.end_wallclimb();
        }

        if self.speed <= 0.0 {
            return self.play_bonk_fx(false);
        }

        let mut flag_a = true;
        let mut should_halve_speed = false;
        let flag_d = false;

        // TODO_VS: `vs_attack` check here
        let contacts_wall = self.num_wall_contacts > 0;

        let can_bonk_and_lose_props = if !self.physics_flags.moved_more_than_rad_0x1d {
            true
        } else {
            flag_a = contacts_wall || self.falling_ticks < 10;
            should_halve_speed = contacts_wall;
            contacts_wall
        };

        let mut surface_normal_unit = [0.0; 3];
        let mut _impact_directness = 0.0;
        let mut _impact_force = 0.0;
        let mut _impact_volume = 0.0;

        if !self.physics_flags.airborne {
            // if the katamari contacts a surface:

            // compute unit lateral velocity
            let lateral_vel_unit = vec3_unit_xz!(self.velocity.vel_accel);

            // compute unit lateral wall normal (usually)
            vec3::copy(&mut surface_normal_unit, &self.contact_wall_normal_unit);
            if !flag_d || !self.physics_flags.moved_more_than_rad_0x1d {
                set_y!(surface_normal_unit, 0.0);
            }
            vec3_inplace_normalize(&mut surface_normal_unit);

            if vec3::dot(&lateral_vel_unit, &surface_normal_unit) > 0.0 {
                return;
            }

            _impact_force = self.compute_impact_force();
            _impact_directness =
                self.compute_impact_directness(&lateral_vel_unit, &surface_normal_unit);
            _impact_volume = _impact_force * _impact_directness;

            if self.can_climb_wall_contact(prince) {
                return self.update_wallclimb();
            } else {
                return self.end_wallclimb();
            }
        } else {
            // if the katamari is airborne:
            vec3::zero(&mut self.velocity.vel_grav);

            // compute the net unit surface normal, which is usually the unit vector
            // sum of the unit floor and wall contact normals.
            if !self.physics_flags.vs_attack {
                vec3::add(
                    &mut surface_normal_unit,
                    &self.contact_floor_normal_unit,
                    &self.contact_wall_normal_unit,
                );
            } else if self.num_wall_contacts == 0 {
                vec3::copy(&mut surface_normal_unit, &self.contact_floor_normal_unit);
            } else {
                vec3::copy(&mut surface_normal_unit, &self.contact_wall_normal_unit);
            }
            vec3_inplace_normalize(&mut surface_normal_unit);

            // if the katamari's "velocity + gravity" is similar to the `surface_normal_unit`, do nothing.
            if vec3::dot(&self.velocity.vel_accel_grav_unit, &surface_normal_unit) > 0.0 {
                return;
            }

            let check_a =
                self.physics_flags.contacts_floor && !self.physics_flags.moved_more_than_rad_0x1d;
            let check_b = self.physics_flags.wheel_spin || self.airborne_ticks < 5;
            if check_a && check_b {
                set_y!(self.velocity.vel_accel, 0.0);
                vec3::zero(&mut self.init_bonk_velocity);
                return;
            }

            _impact_force = self.compute_impact_force();
            _impact_directness = self.compute_impact_directness(
                &self.velocity.vel_accel_grav_unit,
                &surface_normal_unit,
            );
            _impact_volume = _impact_force * _impact_directness;

            if self.physics_flags.moved_more_than_rad_0x14
                && self.physics_flags.grounded_by_mesh_or_prop()
            {}

            let magic_num_0x7b264 = 70.0;
            let magic_num_0x71580 = 0.1;
            let falling_tick_ratio = self.falling_ticks as f32 / magic_num_0x7b264;
            if flag_a {
                // TODO_FX: `kat_update_wall_contacts:218-220` (vibration)
                if !self.physics_flags.contacts_wall
                    && self.physics_flags.airborne
                    && falling_tick_ratio >= magic_num_0x71580
                {
                    if !self.physics_flags.in_water {
                        // TODO_FX: `kat_update_wall_contacts:224-246`
                    }
                    self.physics_flags.just_hit_ground_hard = true;
                }
            }

            if !self.physics_flags.contacts_floor {
                // if the katamari doesn't contact a floor
                // (??) bump the katamari's center away from the wall? (i.e. in the direction of
                // the wall normal)
                let magic_num = 0.1;

                let mut bump = [0.0; 3];
                vec3::scale(&mut bump, &surface_normal_unit, self.radius_cm * magic_num);
                vec3_inplace_add_vec(&mut self.center, &bump);
            }
        }

        // this should be a no-op?
        if vec3::len(&surface_normal_unit) <= 1e-05 {
            panic_log!("weird edge case");
            // return;
        }

        if should_halve_speed {
            self.speed *= 0.5;
        }

        if self.physics_flags.contacts_floor || self.physics_flags.contacts_wall {
            self.airborne_ticks = 0;
            self.falling_ticks = 0;
        }

        if _impact_directness <= 0.0 {
            if _impact_force > 0.0 {
                return;
            }
            if self.physics_flags.hit_by_moving_prop {
                return;
            }

            vec3::zero(&mut self.velocity.vel);
            vec3::zero(&mut self.velocity.vel_unit);
            vec3::zero(&mut self.velocity.vel_accel);
            vec3::zero(&mut self.velocity.vel_accel_unit);
            vec3::zero(&mut self.velocity.vel_accel_grav);
            vec3::zero(&mut self.velocity.vel_accel_grav_unit);
            return;
        }

        let param_xz_elasticity = 0.95;
        let param_min_speed_ratio = 0.3;
        let param_min_impact_angle = 0.3;
        let param_sound_cooldown_ms = 0xa5;

        let mut _play_map_sound = false;
        let mut speed = self.speed;
        if !self.hit_flags.no_reaction_no_slope {
            // if not on a `NoReactionNoSlope` surface:

            let mut refl = [0.0; 3];
            vec3_reflection(
                &mut refl,
                &self.velocity.vel_accel_grav_unit,
                &surface_normal_unit,
            );
            vec3_inplace_scale(&mut refl, -1.0);

            if !self.physics_flags.contacts_wall {
                if should_halve_speed {
                } else {
                    refl[0] *= param_xz_elasticity;
                    refl[1] *= self.y_elasticity;
                    refl[2] *= param_xz_elasticity;
                }
                // TODO_VS: `kat_update_wall_contacts:324-326`
            } else {
                let speed_ratio = self.speed / self.scaled_params.base_max_speed;
                _play_map_sound = speed_ratio > param_min_speed_ratio
                    && _impact_directness > param_min_impact_angle
                    && global.game_time_ms - self.last_collision_game_time_ms
                        > param_sound_cooldown_ms;
                speed *= self.y_elasticity;
            }

            self.physics_flags.bonked = true;
            vec3::scale(&mut self.init_bonk_velocity, &refl, -speed);
        } else {
            // if on a `NoReactionNoSlope` surface:
            set_y!(self.velocity.vel_accel, 0.0);
            vec3::normalize(&mut self.velocity.vel_accel_unit, &self.velocity.vel_accel);
            if self.physics_flags.moved_more_than_rad_0x1d {
                // TODO_LOW: goto
            }
        }

        prince.end_spin_and_boost(self);

        if !can_bonk_and_lose_props {
            return;
        }

        if global.game_time_ms - self.last_collision_game_time_ms >= 0xa6 {
            // the katamari bonks and loses props

            self.last_collision_game_time_ms = global.game_time_ms;
            self.props_lost_from_bonks = 0;
            if !self.physics_flags.climbing_wall
                && !self.last_physics_flags.climbing_wall
                && _impact_force > 0.0
            {
                // TODO_LOW: `kat_begin_screen_shake()`
                let _can_lose_props = !camera.state.cam_eff_1P && !global.map_change_mode;
                // TODO_PROPS: `kat_update_wall_contacts:380-409` (lose props from collision, play bonk sfx)
            }
        }

        self.play_bonk_fx(false);
    }

    /// Returns `true` if the katamari can climb its current wall contact (which could be either
    /// a map surface or a prop surface). This covers both when a new wallclimb could start, or
    /// when the current wallclimb should continue.
    /// offset: 0x16540
    fn can_climb_wall_contact(&mut self, prince: &Prince) -> bool {
        'early_returns: {
            if self.contact_prop.is_none()
                || self.num_wall_contacts != 1
                || self.num_floor_contacts == 0
            {
                // if one of the following hold:
                //   - the katamari isn't colliding with a prop
                //   - the katamari's isn't colliding with exactly 1 wall
                //   - the katamari isn't colliding with a floor

                // do some early checks to rule out the possibility of being able to wallclimb
                if self.hit_flags.wall_climb_disabled {
                    return false;
                }
                if self.physics_flags.hit_shell_ray.is_some() {
                    return false;
                }
                if self.hit_flags.small_ledge_climb {
                    break 'early_returns;
                } else if !self.hit_flags.wall_climb_free {
                    if self.wallclimb_cooldown_timer > 0 {
                        return false;
                    }
                    if self.last_physics_flags.airborne {
                        return false;
                    }
                    if self.physics_flags.airborne {
                        return false;
                    }
                    if self.physics_flags.incline_move_type != KatInclineMoveType::MoveFlatground {
                        return false;
                    }
                }
            } else {
                // if all of the following hold:
                //   - the katamari contacts a prop
                //   - the katamari is colliding with exactly 1 wall
                //   - the katamari is colliding with a floor

                // TODO_PROPS: `kat_can_climb_wall_contact:69-82` (init prop wallclimb)
            }

            if self.num_wall_contacts > 1 {
                return false;
            }
        }

        // don't start a new wall climb if the katamari doesn't currently contact a wall
        if !self.physics_flags.contacts_wall && !self.physics_flags.climbing_wall {
            return false;
        }

        // check that the angle between the katamari's push velocity and the wall normal are close
        // enough to admit a wallclimb. since the wall normal is actually pointing *out* of the wall,
        // we need to throw in a negative somewhere in there.
        let similarity = vec3::dot(
            &self.velocity.push_vel_on_floor_unit,
            &self.hit_walls[0].normal_unit,
        );
        let angle = acos_f32(-similarity);
        if angle > self.params.max_wallclimb_angle {
            return false;
        }

        // check that the input is strong enough (and forward enough) to admit a wallclimb
        if !prince.has_wallclimb_input() {
            return false;
        }

        if !self.physics_flags.climbing_wall {
            // if the katamari isn't already wallclimbing:
            // start a new wallclimb
            self.wallclimb_normal_unit = self.contact_wall_normal_unit;
            self.wallclimb_speed = 0.0;
            self.physics_flags.at_max_climb_height = false;

            // (??) not sure what this is doing
            // TODO_LOW: factor out magic number as param
            if !self.hit_flags.wall_climb_free && self.base_speed * 0.95 < self.speed {
                return false;
            }
        }

        // finally, check the similarity between the katamari's push velocity and the wall
        // normal. if the two vectors are similar enough, we can start a wallclimb.
        let mut lateral_push_vel = self.velocity.push_vel_on_floor_unit;
        set_y!(lateral_push_vel, 0.0);
        vec3_inplace_normalize(&mut lateral_push_vel);

        let mut lateral_wallclimb_normal = self.wallclimb_normal_unit;
        set_y!(lateral_wallclimb_normal, 0.0);
        vec3_inplace_normalize(&mut lateral_wallclimb_normal);

        let push_to_wall_similarity = -vec3::dot(&lateral_push_vel, &lateral_wallclimb_normal);
        return push_to_wall_similarity >= self.params.min_wallclimb_similarity;
    }

    /// Update the current wallclimb.
    /// offset: 0x16930
    fn update_wallclimb(&mut self) {
        if !self.physics_flags.at_max_climb_height {
            self.is_climbing_0x898 = 1;
        }

        self.wallclimb_cooldown_timer = 0;

        if !self.physics_flags.climbing_wall {
            self.wallclimb_init_y = self.center[1];
            self.wallclimb_init_radius = self.climb_radius_cm;
        }

        self.physics_flags.climbing_wall = true;
        vec3::zero(&mut self.init_bonk_velocity);
        self.physics_flags.grounded_ray_type = Some(KatCollisionRayType::Bottom);
        self.vault_ray_idx = None;
        self.fc_ray_idx = None;
    }

    /// End the current wallclimb, if one is ongoing.
    /// offset: 0x12ca0
    fn end_wallclimb(&mut self) {
        if self.physics_flags.climbing_wall {
            if self.is_climbing_0x898 > 0 {
                return self.is_climbing_0x898 -= 1;
            }
            self.wallclimb_ticks = 0;
            self.wallclimb_cooldown_timer = 10;
        }

        self.physics_flags.climbing_wall = false;
        self.physics_flags.at_max_climb_height = false;
        self.wallclimb_init_y = 0.0;
        self.wallclimb_max_height_ticks = 0;
    }

    /// (??)
    /// offset: 0x16ed0
    fn compute_impact_force(&self) -> f32 {
        if !self.physics_flags.moved_more_than_rad_0x1d && self.physics_flags.airborne {
            if self.physics_flags.contacts_floor {
                // if contacting a floor, interpolate the number of ticks the katamari has been falling
                return inv_lerp_clamp!(
                    self.falling_ticks as f32,
                    self.params.min_impact_falling_frames as f32,
                    self.params.max_impact_falling_frames as f32
                );
            }

            // if not contacting a floor, return the ratio of the katamari's speed to its base speed
            return (self.speed / self.base_speed).clamp(0.0, 1.0);
        }

        let speed_ratio = (self.speed / self.base_speed).clamp(0.0, 1.0);
        return ((speed_ratio - 0.25) * 4.0).clamp(0.0, 1.0);
    }

    /// (??)
    /// offset: 0x16df0
    fn compute_impact_directness(&self, kat_vel: &Vec3, surface_normal: &Vec3) -> f32 {
        if self.last_physics_flags.climbing_wall {
            return 0.0;
        }
        // TODO_VS: `kat_compute_impact_angle:12-14`

        let similarity = if !self.physics_flags.airborne || !self.physics_flags.contacts_floor {
            vec3::dot(&kat_vel, &surface_normal)
        } else {
            vec3::dot(&VEC3_Y_NEG, &surface_normal)
        };

        if similarity < 0.0 {
            return 0.0;
        }

        let angle = acos_f32(similarity);
        return (angle - FRAC_PI_2) / FRAC_PI_2;
    }

    /// (??)
    /// offset: 0x12750
    fn play_bonk_fx(&mut self, _prop_moving: bool) {
        // TODO_FX
    }

    /// Check the current primary floor contact ray to see if a vault on that ray should be
    /// initialized.
    /// offset: 0x153c0
    fn try_init_vault(&mut self) -> TryInitVaultResult {
        // early returns when a vault isn't starting
        if self.fc_ray_idx == Some(0) || self.fc_ray_idx.is_none() {
            return TryInitVaultResult::NoVault;
        }
        if self.fc_ray_idx == self.vault_ray_idx {
            return TryInitVaultResult::OldVault;
        }

        // early return when the collision ray isn't strictly longer than the katamari radius
        let vault_ray_len_radii = self.fc_ray_len / self.radius_cm;
        if vault_ray_len_radii <= 1.0 {
            return TryInitVaultResult::NoVault;
        }

        // compute features of the vault ray length
        self.vault_ray_idx = self.fc_ray_idx;
        self.vault_ray_len_radii = vault_ray_len_radii;
        self.vault_ray_max_len_ratio =
            min!(vault_ray_len_radii / self.params.max_ray_len_radii, 1.0);

        // compute unit rejection of katamari velocity onto floor normal
        let mut vel_proj_floor = [0.0; 3];
        let mut vel_rej_floor = [0.0; 3];
        vec3_projection(
            &mut vel_proj_floor,
            &mut vel_rej_floor,
            &self.velocity.vel_unit,
            &self.contact_floor_normal_unit,
        );
        vec3_inplace_normalize(&mut vel_rej_floor);

        // compute the unit `fc_ray`
        let mut fc_ray_unit = self.fc_ray.unwrap();
        vec3_inplace_normalize(&mut fc_ray_unit);
        vec3_inplace_zero_small(&mut fc_ray_unit, 1e-05);

        let mut ray_proj_floor = [0.0; 3];
        let mut ray_rej_floor = [0.0; 3];
        vec3_projection(
            &mut ray_proj_floor,
            &mut ray_rej_floor,
            &fc_ray_unit,
            &self.contact_floor_normal_unit,
        );
        vec3_inplace_zero_small(&mut ray_rej_floor, 1e-05);

        // transform the angle between the rejections:
        //   [0, PI/2] -> [1, 0]
        //   [PI/2, PI] -> 0
        let rej_similarity = vec3::dot(&vel_rej_floor, &ray_rej_floor);
        let rej_angle = acos_f32(rej_similarity);
        self.vault_rej_angle_t = inv_lerp_clamp!(rej_angle, 0.0, FRAC_PI_2);

        // (??) set the initial vault speed
        if FRAC_PI_90 < rej_similarity {
            let speed_t =
                inv_lerp_clamp!(self.speed, self.max_forwards_speed, self.max_boost_speed);
            let ray_len_t = inv_lerp_clamp!(self.fc_ray_len, self.radius_cm, self.max_ray_len);
            let ray_len_k = lerp!(ray_len_t, 1.0, self.params.vault_tuning_0x7b208);
            let speed_k = lerp!(speed_t, ray_len_k, 1.0);

            // TODO: if this is buggy, this part is probably why
            let mut vel_reflect_floor = [0.0; 3];
            vec3_reflection(&mut vel_reflect_floor, &ray_rej_floor, &vel_rej_floor);
            vec3_inplace_scale(&mut vel_reflect_floor, -1.0);

            let mut vel_accel = [0.0; 3];
            vec3::lerp(&mut vel_accel, &vel_rej_floor, &vel_reflect_floor, speed_k);

            vec3::scale(&mut self.velocity.vel_accel, &vel_accel, self.speed);
        }

        return TryInitVaultResult::InitVault;
    }

    fn another_end_wallclimb(&mut self) {
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
        self.velocity.vel_grav[1] + self.velocity.vel_accel[1] < 0.0
    }

    /// offset: 0x183f0
    fn apply_clip_translation(&mut self) {
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

                // the katamari is moving towards the wall if:
                // its velocity dot the wall normal is below the similarity threshold
                // (since the wall normal should be pointing away from katamari movement, any
                // negative similarity should also work)
                vec3::dot(&move_xz_unit, &wall_normal_xz_unit)
                    <= self.params.move_into_wall_similarity
            }
        } else {
            // if the katamari doesn't contact a wall, it's not moving into one either.
            false
        };

        vec3::zero(&mut self.clip_translation);

        if self.physics_flags.stuck_between_walls && !self.last_physics_flags.stuck_between_walls {
            // if the katamari has just gotten stuck between walls:
            // push the katamari away from the wall to try to get unstuck
            let push_distance = self.scaled_params.base_max_speed * self.params.unstuck_bump_speed;
            let mut push_velocity = self.stuck_btwn_walls_push_unit;
            vec3_inplace_scale(&mut push_velocity, push_distance);

            // forcibly set the katamari's velocity to push it away from the wall
            self.set_velocity(&push_velocity);
        }

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
                // TODO_LOW: `kat_apply_clip_translation:125-146` (weird edge case??)
            }
        } else {
            // if stuck between walls:
            self.clip_translation[0] += self.center[0] - self.last_center[0];
            self.clip_translation[2] += self.center[2] - self.last_center[2];
        }

        // apply the clip translation to the katamari center
        vec3_inplace_subtract_vec(&mut self.center, &self.clip_translation);

        // update center, bottom, and top points
        set_translation!(self.transform, self.center);

        self.bottom = self.center;
        self.bottom[1] -= self.radius_cm;

        self.top = self.center;
        self.top[1] += self.radius_cm;
    }
}
