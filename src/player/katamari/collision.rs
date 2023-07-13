use gl_matrix::{common::Vec3, mat4, vec3};

use crate::{
    collision::{hit_attribute::HitAttribute, raycast_state::RaycastCallType},
    constants::{FRAC_5PI_12, FRAC_PI_2, FRAC_PI_90, PI, VEC3_Y_NEG, VEC3_ZERO},
    debug::DEBUG_CONFIG,
    delegates::{has_delegates::HasDelegates, sound_id::SoundId, vfx_id::VfxId},
    global::GlobalState,
    macros::{
        inv_lerp, inv_lerp_clamp, lerp, mark_address, mark_call, max, min, modify_translation,
        panic_log, set_translation, set_y, vec3_from, vec3_unit_xz,
    },
    math::{
        acos_f32, vec3_inplace_add_vec, vec3_inplace_normalize, vec3_inplace_scale,
        vec3_inplace_subtract, vec3_inplace_subtract_vec, vec3_inplace_zero_small, vec3_projection,
        vec3_reflection,
    },
    mission::{state::MissionState, GameMode, GameType, Mission},
    player::{camera::Camera, katamari::flags::GroundedRay, prince::Prince},
    props::{
        config::{NamePropConfig, NAME_PROP_CONFIGS},
        prop::{
            Prop, PropFlags1, PropFlags2, PropGlobalState, PropRef, PropTrajectoryType,
            PropUnattachedState,
        },
        PropsState,
    },
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

/// Encodes the three possible return values from `Katamari::record_surface_contact`.
/// The `Floor` and `Wall` values report that a surface of that type was contacted.
/// `ShellTop` reports that a floor was contacted by a top shell ray, and in this
/// case a surface contact is *not* recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordSurfaceContactResult {
    Floor,
    Wall,
    ShellTop,
}

impl TryFrom<RecordSurfaceContactResult> for SurfaceType {
    type Error = ();
    fn try_from(value: RecordSurfaceContactResult) -> Result<Self, Self::Error> {
        match value {
            RecordSurfaceContactResult::Floor => Ok(Self::Floor),
            RecordSurfaceContactResult::Wall => Ok(Self::Wall),
            RecordSurfaceContactResult::ShellTop => Err(()),
        }
    }
}

/// The three possible results returned by `Katamari::try_init_vault`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// offset: 0x12cf0
    pub fn update_collision(
        &mut self,
        prince: &mut Prince,
        camera: &Camera,
        global: &mut GlobalState,
        mission_state: &MissionState,
        props: &mut PropsState,
    ) {
        self.last_num_floor_contacts = self.num_floor_contacts;
        self.last_num_wall_contacts = self.num_wall_contacts;
        self.num_floor_contacts = 0;
        self.hit_floors.clear();
        self.num_wall_contacts = 0;
        self.hit_walls.clear();
        self.fc_ray_idx = None;

        // TODO_VS: `kat_update_collision:41-61` (compute vol ratio for vs mode)

        self.aabb_prop_collision_vol_m3 =
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
        self.physics_flags.moved_fast = self.radius_cm <= vec3::length(&moved);
        self.physics_flags.moved_fast_shell_hit = false;

        // TODO_VS: `kat_update_collision:96-101` (decrement timer)

        mark_address!("0x12f7f");
        self.find_nearby_props(props, prince, mission_state);
        mark_address!("0x12f87");

        if mission_state.gamemode == GameMode::Ending {
            // TODO_ENDING: `kat_update_collision:105-132 (ending-specific reduced collision)
        } else {
            // TODO: `kat_update_water_contact()`
            mark_address!("0x1302a");

            self.compute_surface_contacts();
            mark_address!("0x1303a");

            self.process_surface_contacts();
            mark_address!("0x13042");

            self.resolve_being_stuck(mission_state, global);
            mark_address!("0x1304a");

            self.update_vault_and_climb(prince, camera, global, mission_state);
            mark_address!("0x13052");

            if self.physics_flags.airborne && self.raycast_state.closest_hit_idx.is_some() {
                let mut top = [0.0, self.radius_cm + self.radius_cm, 0.0];
                vec3_inplace_add_vec(&mut top, &self.center);
                self.raycast_state.load_ray(&self.center, &top);
                let found_hit = self
                    .raycast_state
                    .find_nearest_unity_hit(RaycastCallType::Objects, false);

                if let Some(hit) = self.raycast_state.get_closest_hit() {
                    if found_hit && (hit.metadata == HitAttribute::SpecialCamera as i32) {
                        self.hit_flags.special_camera = true;
                    }

                    // TODO_LOW: original sim updates `self.active_hit_before_airborne` here, but
                    // that field seems to be otherwise unused
                }
            }
        }

        // TODO_PARAM
        let MIN_ATTACHED_PROPS_FOR_SOMETHING = 100.0;
        let MAX_ATTACHED_PROPS_FOR_SOMETHING = 190.0;

        let attached_props = self.num_attached_props as f32;
        let t = inv_lerp_clamp!(
            attached_props,
            MIN_ATTACHED_PROPS_FOR_SOMETHING,
            MAX_ATTACHED_PROPS_FOR_SOMETHING
        );
        let destroy_props_radius =
            self.display_radius_cm + t * (self.radius_cm - self.display_radius_cm) * 0.75;

        for prop_ref in self.attached_props.iter_mut() {
            let mut prop = prop_ref.borrow_mut();
            prop.do_unattached_translation(&self.clip_translation);

            let in_destroy_range = prop.get_dist_to_katamari(self.player as i32)
                + prop.get_radius()
                < destroy_props_radius;

            if !prop.is_disabled() && in_destroy_range {
                prop.destroy();
            }
        }

        self.process_nearby_collectible_props(mission_state);
        self.process_collected_props(props, mission_state, global);
        // TODO: `kat_update_world_size_threshold??()`

        if self.physics_flags.grounded_ray_type.is_bottom() || self.fc_ray_len < 1.0 {
            self.fc_ray_len = self.radius_cm;
        }
    }

    /// Iterate over all props to find those which are "nearby" the katamari.
    /// In the original implementation, "nearby" means the prop-katamari distance is
    /// less than the sum of (1) the radii of the bounding spheres of the katamari and the prop,
    /// and (2) the distance that the katamari moved over the last frame.
    /// offset: 0x28870
    fn find_nearby_props(
        &mut self,
        props: &mut PropsState,
        prince: &mut Prince,
        mission_state: &MissionState,
    ) {
        // TODO_VS: `kat_find_nearby_props:43` (return immediately if vs mode or if other vs condition holds)

        // TODO_PARAM: make this a global parameter
        let MAX_COLLECTION_CHECKS_PER_FRAME = 0x80;

        // compute the distance the katamari moved since the last frame
        let kat_move = vec3_from!(-, self.center, self.last_center);
        let kat_move_len = vec3::length(&kat_move);
        let mut lateral_move_unit = [kat_move[0], 0.0, kat_move[2]];
        vec3_inplace_normalize(&mut lateral_move_unit);

        self.nearby_collectible_props.clear();
        self.new_collected_props.clear();

        self.contact_prop = None;

        if self.ignore_prop_collision_timer != 0 {
            return;
        }

        let mut cloned_prop_list = props.props.clone();
        for prop_ref in cloned_prop_list.iter_mut() {
            let mut prop = prop_ref.borrow_mut();

            // return early from the collision check if the prop is still intangible.
            if prop.intangible_timer > 0 {
                prop.intangible_timer -= 1;
                continue;
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
                continue;
            }

            // The prop-katamari distance decreased at most by the distance the katamari just moved.
            // If that minimum distance is still bigger than the sum of the kat's and prop's bounding
            // spheres, they can't collide.
            let min_dist_to_kat = max!(prop.get_dist_to_katamari(0) - kat_move_len, 0.0);

            if min_dist_to_kat > self.radius_cm + prop.get_radius() {
                continue;
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

            let did_collide =
                self.check_prop_mesh_collision(prop_ref.clone(), &mut prop, mission_state);

            if !did_collide {
                continue;
            }

            // TODO_PARAM
            let MIN_MAX_SPEED_RATIO_FOR_SCREAM = 0.6;
            let MIN_DIAM_RATIO_FOR_SCREAM = 5.0;

            let fast_enough_for_scream = self.max_speed_ratio >= MIN_MAX_SPEED_RATIO_FOR_SCREAM;
            let big_enough_for_scream =
                prop.get_exact_attach_diam_cm() / self.diam_cm <= MIN_DIAM_RATIO_FOR_SCREAM;
            let scream_off_cooldown = prop.get_scream_cooldown_timer() == 0;

            if fast_enough_for_scream && big_enough_for_scream && scream_off_cooldown {
                let scream_kind = NamePropConfig::get(prop.get_name_idx()).scream_sfx_kind;
                if scream_kind > 0 {
                    let sound_id = SoundId::PropBonk(scream_kind);
                    self.play_sound_fx(sound_id, 1.0, 0);
                }
            }

            prop.reset_scream_cooldown_timer();

            self.contact_prop = Some(prop_ref.clone());
            prop.set_kat_collision_vel(&kat_move);
            self.resolve_uncollectible_prop_collision(props, prop_ref.clone(), &mut prop);

            // TODO_WOBBLE: `kat_find_nearby_props:169-178`

            // the remainder of the function handles collisions with spinning fight props
            // (e.g. sumo bout, judo contest)
            if prop.get_flags2().contains(PropFlags2::SpinningFight) {
                let prop_to_player = vec3_from!(-, self.center, prop.pos);
                let prop_to_player_lateral_unit = vec3_unit_xz!(prop_to_player);

                // TODO_PARAM
                let SPINNING_FIGHT_HIT_SPEED_MULT = 0.1;
                self.speed = prop.get_radius() * SPINNING_FIGHT_HIT_SPEED_MULT;
                vec3::scale(
                    &mut self.velocity.vel_accel,
                    &prop_to_player_lateral_unit,
                    self.speed,
                );

                // TODO_VS: this call doesn't happen in vs mode
                prince.end_spin_and_boost(self);
            }
        }
    }

    /// Returns `true` if this katamari meets `prop` in a non-collection collision.
    /// This collision uses the more precise collision mesh of `prop` as opposed to its AABB mesh.
    /// offset: 0x29480
    fn check_prop_mesh_collision(
        &mut self,
        prop_ref: PropRef,
        prop: &mut Prop,
        mission_state: &MissionState,
    ) -> bool {
        // check if the loaded area is below the area where the prop becomes intangible
        // TODO: should that `<=` be a `<`?
        let hit_on_area = prop.get_hit_on_area();
        if hit_on_area.is_some() && mission_state.area <= hit_on_area.unwrap() {
            return false;
        }

        // TODO_WOBBLE: `kat_check_prop_mesh_collision:102-103` (intangibility check while prop is wobbling)

        // only use the prop's AABB mesh for collision if (1) the prop's `NamePropConfig` allows it,
        // and (2) the katamari is bigger than the prop.
        let use_aabb_for_collision =
            NamePropConfig::get(prop.get_name_idx()).use_aabb_for_collision;
        let prop_bigger_than_kat = prop.get_compare_vol_m3() >= self.aabb_prop_collision_vol_m3;
        let prop_mesh = if use_aabb_for_collision && !prop_bigger_than_kat {
            prop.get_aabb_mesh()
        } else {
            prop.get_collision_mesh()
        }
        .unwrap();

        let mut prop_rot = prop.get_unattached_transform().clone();
        modify_translation!(prop_rot, =, VEC3_ZERO);

        let mut prop_rot_inv = mat4::create();
        mat4::transpose(&mut prop_rot_inv, &prop_rot);

        let prop_transform = &prop.get_unattached_transform().clone();

        // The vector from the prop to the katamari in world space.
        let prop_to_kat_world = vec3_from!(-, self.center, prop.pos);

        // The vector from the prop to the katamari, relative to the prop's coordinate space.
        let mut prop_to_kat_local = vec3::create();
        vec3::transform_mat4(&mut prop_to_kat_local, &prop_to_kat_world, &prop_rot_inv);

        if self.physics_flags.immobile {
            if self.physics_flags.moved_fast {
                // TODO: `kat_check_prop_mesh_collision:LAB18002a05e` (goto to check shell rays?)
            }
        } else if self.physics_flags.moved_fast {
            // check shell ray collisions with props when katamari moved more than its radius

            // TODO_BUG: the original sim seems to expect the shell ray at index 4 to be the bottom ray,
            // but in reality it's `top_left`. this might be a source of dubs?
            let mut shell_inits: [Vec3; 5] = Default::default();
            let mut shell_ends: [Vec3; 5] = Default::default();

            let shell_init_base = &vec3_from!(-, self.last_center, self.shell_vec);
            let shell_end_base = &self.center;

            vec3::copy(&mut shell_inits[0], shell_init_base);
            vec3::add(&mut shell_inits[1], shell_init_base, &self.shell_top);
            vec3::add(&mut shell_inits[2], shell_init_base, &self.shell_left);
            vec3::add(&mut shell_inits[3], shell_init_base, &self.shell_right);
            vec3::add(&mut shell_inits[4], shell_init_base, &self.shell_top_left);

            vec3::copy(&mut shell_ends[0], &shell_end_base);
            vec3::add(&mut shell_ends[1], &shell_end_base, &self.shell_top);
            vec3::add(&mut shell_ends[2], &shell_end_base, &self.shell_left);
            vec3::add(&mut shell_ends[3], &shell_end_base, &self.shell_right);
            vec3::add(&mut shell_ends[4], &shell_end_base, &self.shell_top_left);

            let mut found_hit = false;
            for i in 0..5 {
                self.raycast_state.load_ray(&shell_inits[i], &shell_ends[i]);
                let hit_prop = self
                    .raycast_state
                    .ray_hits_mesh(&prop_mesh, prop_transform, false);

                if hit_prop == 0 {
                    continue;
                }

                prop.set_katamari_contact(self.player);
                self.physics_flags.airborne = true;
                self.physics_flags.moved_fast_shell_hit_0x14 = true;
                self.physics_flags.moved_fast_shell_hit_0x1d = true;
                found_hit = true;

                let mut shell_ray = vec3::create();
                match i {
                    0 => {}
                    1 => {
                        vec3::copy(&mut shell_ray, &self.shell_top);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Top);
                    }
                    2 => {
                        vec3::copy(&mut shell_ray, &self.shell_left);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Left);
                    }
                    3 => {
                        vec3::copy(&mut shell_ray, &self.shell_right);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Right);
                    }
                    4 => {
                        vec3::copy(&mut shell_ray, &self.shell_bottom);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Bottom);
                    }
                    _ => {}
                }

                if let Some(hit) = self.raycast_state.get_closest_hit_mut() {
                    // TODO_BUG: in the original sum, if `hit_shell_ray` wasn't assigned above
                    // (which happens when the only shell ray hit is when `i` is 0)
                    let shell_ray_idx = match self.physics_flags.hit_shell_ray {
                        Some(shell_ray) => -(shell_ray as i16),
                        None => 0,
                    };

                    vec3_inplace_subtract_vec(&mut hit.impact_point, &shell_ray);
                    vec3_inplace_subtract_vec(&mut self.raycast_state.point1, &shell_ray);

                    let record_result =
                        self.record_surface_contact(shell_ray_idx, Some(prop_ref.clone()));

                    if record_result != RecordSurfaceContactResult::ShellTop {
                        self.physics_flags.moved_fast_shell_hit = true;
                        self.play_bonk_fx(prop.get_move_type().is_some());
                        self.contact_prop = Some(prop_ref.clone());
                        return true;
                    }
                }
            }

            if found_hit {
                self.contact_prop = Some(prop_ref.clone());
                return true;
            }
        }

        // test "slow" shell rays against prop collision mesh
        let mut shell_inits: [Vec3; 5] = Default::default();
        let mut shell_ends: [Vec3; 5] = Default::default();

        let shell_init_base = &mut vec3::create();
        vec3::scale_and_add(shell_init_base, &self.last_center, &self.shell_vec, -0.5);

        let shell_end_base = &self.center;

        vec3::add(&mut shell_inits[0], shell_init_base, &self.shell_top);
        vec3::add(&mut shell_inits[1], shell_init_base, &self.shell_left);
        vec3::add(&mut shell_inits[2], shell_init_base, &self.shell_right);
        vec3::add(&mut shell_inits[3], shell_init_base, &self.shell_top_left);
        vec3::add(&mut shell_inits[4], shell_init_base, &self.shell_top_right);

        vec3::add(&mut shell_ends[0], &shell_end_base, &self.shell_top);
        vec3::add(&mut shell_ends[1], &shell_end_base, &self.shell_left);
        vec3::add(&mut shell_ends[2], &shell_end_base, &self.shell_right);
        vec3::add(&mut shell_ends[3], &shell_end_base, &self.shell_top_left);
        vec3::add(&mut shell_ends[4], &shell_end_base, &self.shell_top_right);

        for i in 0..5 {
            self.raycast_state.load_ray(&shell_inits[i], &shell_ends[i]);
            let found_hit = self
                .raycast_state
                .ray_hits_mesh(&prop_mesh, &prop_transform, false);

            if found_hit == 0 {
                continue;
            }

            // process shell hit
            self.physics_flags.kat_hit_surface_maybe_0xd = true;
            self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Top);
            prop.set_katamari_contact(self.player);

            let mut shell_ray = vec3::create();
            match i {
                0 => {}
                1 => {
                    vec3::copy(&mut shell_ray, &self.shell_top);
                    self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Top);
                }
                2 => {
                    vec3::copy(&mut shell_ray, &self.shell_left);
                    self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Left);
                }
                3 => {
                    vec3::copy(&mut shell_ray, &self.shell_right);
                    self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Right);
                }
                4 => {
                    vec3::copy(&mut shell_ray, &self.shell_top_left);
                    self.physics_flags.hit_shell_ray = Some(ray::ShellRay::TopLeft);
                }
                5 => {
                    vec3::copy(&mut shell_ray, &self.shell_top_right);
                    self.physics_flags.hit_shell_ray = Some(ray::ShellRay::TopRight);
                }
                _ => {}
            }

            if let Some(hit) = self.raycast_state.get_closest_hit_mut() {
                vec3_inplace_subtract_vec(&mut hit.impact_point, &shell_ray);
                vec3_inplace_subtract_vec(&mut self.raycast_state.point1, &shell_ray);

                let ray_idx = self
                    .physics_flags
                    .hit_shell_ray
                    .map(|sr| sr as i16)
                    .unwrap_or(0);
                let record_result = self.record_surface_contact(ray_idx, Some(prop_ref.clone()));

                if record_result != RecordSurfaceContactResult::ShellTop {
                    self.play_bonk_fx(prop.get_move_type().is_some());
                    break;
                }
            }
        }

        // test kat collision rays against prop collision mesh
        let mut found_any_hit = false;
        // TODO_PERF: don't clone collision rays here
        let rays = self.collision_rays.clone();
        for (ray_idx, ray) in rays.iter().enumerate() {
            let ray_endpoint = vec3_from!(+, self.center, ray.kat_to_endpoint);
            self.raycast_state.load_ray(&self.center, &ray_endpoint);

            let found_hit = self
                .raycast_state
                .ray_hits_mesh(&prop_mesh, &prop_transform, false);

            if found_hit == 0 {
                continue;
            }

            self.physics_flags.kat_hit_surface_maybe_0xd = true;
            prop.set_katamari_contact(self.player);
            found_any_hit = true;

            if let Some(hit) = self.raycast_state.get_closest_hit_mut() {
                vec3_inplace_zero_small(&mut hit.normal_unit, 0.00001);
            }

            let should_contact = match (prop.get_global_state(), prop.get_unattached_state()) {
                (PropGlobalState::Unattached, PropUnattachedState::State2) => false,
                (PropGlobalState::Unattached, PropUnattachedState::AirborneBounced) => false,
                (PropGlobalState::AirborneIntangible, _) => false,
                _ => true,
            };

            if should_contact {
                self.record_surface_contact(ray_idx as i16, Some(prop_ref.clone()));
            } else {
                // TODO_PARAM
                let MIN_KAT_SPEED_AFTER_PROP_HIT = 0.5;
                let KAT_SPEED_AFTER_PROP_HIT_MULT = 0.75;
                let PROP_INTANGIBILITY_AFTER_HIT = 10;

                // TODO_DOC: if the katamari isn't going to contact the prop, then it
                // should bounce off of the prop instead
                self.physics_flags.contacts_prop_0xa = true;
                prop.intangible_timer = PROP_INTANGIBILITY_AFTER_HIT;

                let base_speed = self.max_boost_speed;

                // the katamari's speed after the collision is a clamped
                let prop_moved = vec3::length(&vec3_from!(-, prop.last_pos, prop.pos));
                let next_kat_speed = KAT_SPEED_AFTER_PROP_HIT_MULT
                    * prop_moved.clamp(base_speed * MIN_KAT_SPEED_AFTER_PROP_HIT, base_speed);

                // its velocity is in the direction that the prop moved the previous frame
                let mut next_kat_vel = vec3_unit_xz!(vec3_from!(-, prop.pos, self.center));
                vec3_inplace_scale(&mut next_kat_vel, next_kat_speed);
                self.set_velocity(&next_kat_vel);
                self.physics_flags.hit_by_moving_prop = true;
            }

            self.play_bonk_fx(prop.get_move_type().is_some());
        }

        if found_any_hit {
            self.contact_prop = Some(prop_ref.clone());

            // if any aabb was hit, attempt to draw the prop's mesh
            if DEBUG_CONFIG.draw_collided_prop_mesh {
                self.debug_draw_collided_prop_mesh(&prop_mesh, &prop_transform);
            }
        }

        found_any_hit
    }

    /// Resolve a collision between this katamari and an uncollectible prop.
    /// offset: 0x2af40
    fn resolve_uncollectible_prop_collision(
        &mut self,
        _props: &mut PropsState,
        _prop_ref: PropRef,
        _prop: &mut Prop,
    ) {
        // temp_debug_log!("... ctrl_idx={}", prop.get_ctrl_idx());
        // let root_prop = if prop.has_parent() {
        //     let root_ref = prop.get_root_ref(props);
        //     root_ref.clone().borrow_mut()
        // } else {
        //     prop
        // };

        // // TODO_LINK:
        // // if `root_prop.link_action + ~CHILDREN_INTANGIBLE & 0xfd == 0` { root_prop = prop }

        // // Handle collisions with a stationary prop.
        // if root_prop.get_move_type().is_none() {
        //     return self.resolve_stationary_prop_collision(prop);
        // }

        // // TODO_DOC: what is this doing, something to do with turntables
        // let behavior_cond = root_prop.get_behavior_type() == Some(PropBehavior::Value0x15);
        // let name_idx = prop.get_name_idx();
        // let prop_is_turntable = name_idx == 0x31d // Manhole Cover
        //     || name_idx == 0x35b // Round Table
        //     || name_idx == 0x55b; // Parking Turntable
        // let prop_barely_moved = vec3::len(&vec3_from!(-, prop.last_pos, prop.pos)) <= 1.0;
        // let weird_cond = behavior_cond || prop_is_turntable || prop_barely_moved;
        // if root_prop.get_stationary() && weird_cond {
        //     return;
        // }

        // if self.physics_flags.vs_attack {
        //     return;
        // }

        // if root_prop.get_flags2().contains(PropFlags2::Wobble) {
        //     return;
        // }

        // TODO_PROP_MOTION: `kat_resolve_uncollectible_prop_collision:72-`
    }

    /// Resolve a collision between this katamari and an uncollectible, stationary prop.
    /// This implements inelastic collisions exhibited by e.g. toy capsules and baseballs.
    /// offset: 0x2bd60
    fn resolve_stationary_prop_collision(&mut self, prop: &mut Prop) {
        if prop.get_unattached_state() != PropUnattachedState::InelasticRoll {
            return;
        }

        // TODO_PROP_MOTION: `kat_resolve_stationary_prop_collision` (inelastic rolling collision)
    }

    /// offset: 0x28640
    fn process_nearby_collectible_props(&mut self, mission_state: &MissionState) {
        // TODO_PARAM
        let SQUASH_PROP_VOL_MULTIPLIER = 3.0;
        let MAX_COLLECTED_PROPS_PER_FRAME = 0x40;

        if mission_state.is_ending() {
            // TODO_ENDING: `kat_process_nearby_collectible_props:13-33`
        } else {
            // TODO_PERF: not amazing having to clone the `nearby_collectible_props` list here
            for prop_ref in self.nearby_collectible_props.clone() {
                let prop = prop_ref.borrow_mut();
                let prop_config = NAME_PROP_CONFIGS.get(prop.get_name_idx() as usize).unwrap();

                let link_cond = prop.parent.is_none()
                    || !prop.get_flags().contains(PropFlags1::IntangibleChild);
                let is_dummy = prop_config.is_dummy_hit;
                let did_collide = self.intersects_prop_bbox(&prop, mission_state);
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
                        self.new_collected_props.push(prop_ref.clone());
                        if self.new_collected_props.len() >= MAX_COLLECTED_PROPS_PER_FRAME {
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
            self.larger_avg_mesh_ray_len
        } else {
            self.avg_mesh_ray_len
        };

        if mission_state.is_ending() {
            kat_sphere_rad += kat_sphere_rad;
        }

        // compute the unit vector from the prop to the katamari
        let prop_pos = prop.get_position().clone();
        let mut prop_to_kat_unit = vec3_from!(-, self.center, prop_pos);
        vec3_inplace_normalize(&mut prop_to_kat_unit);

        // compute the ray from the katamari's center towards the prop, with length `kat_sphere_rad`.
        let mut ray_endpoint = vec3::create();
        vec3::scale_and_add(
            &mut ray_endpoint,
            &self.center,
            &prop_to_kat_unit,
            -kat_sphere_rad,
        );

        self.raycast_state.load_ray(&self.center, &ray_endpoint);
        let num_hit_tris = self.raycast_state.ray_hits_mesh(
            &prop.get_aabb_mesh().unwrap(),
            prop.get_unattached_transform(),
            false,
        );
        num_hit_tris > 0
    }

    /// offset: 0x280c0
    fn process_collected_props(
        &mut self,
        props: &PropsState,
        mission_state: &MissionState,
        global: &mut GlobalState,
    ) {
        if !self.new_collected_props.is_empty() {
            // if at least one prop was collected:
            if let Some(delegates) = &self.delegates {
                if let Some(_vibration) = delegates.borrow().vibration {
                    // TODO_VIBRATION (have to figure out the arguments)
                }
            }

            // TODO_LOW: the rest of this block should only be run if gamemode isn't 4
            let base_sound_id = mission_state
                .stage_config
                .get_base_collect_object_sound_id(self.diam_trunc_mm as u32);
            let rng2 = global.rng.get_rng2() as u16;
            let sound_id = base_sound_id + (rng2 % 3);
            self.play_sound_fx(sound_id.into(), 1.0, 0);
        }

        let mut collected_props = self.new_collected_props.clone();

        for (collection_idx, prop_ref) in collected_props.iter_mut().enumerate().rev() {
            // TODO_LOW: early exit from this loop if we reached the gametype c goal
            //           (which is a fixed # of collected props)
            let mut prop = prop_ref.borrow_mut();
            let name_idx = prop.get_name_idx();
            let prop_config = NamePropConfig::get(name_idx);

            if prop.global_state == PropGlobalState::Attached {
                continue;
            }

            // TODO_LOW: prop.game_time_when_collected = game_time_ms (move to `prop.attach_to_kat` or whatever)
            prop.set_katamari_contact(self.player);

            self.attach_prop(props, prop_ref, &mut prop, mission_state, global);
            // TODO_LINK: `attach_prop_with_children(prop)`
            prop.get_flags2_mut().remove(PropFlags2::Flee);

            // if the object has a scream type, play its collection scream sfx
            let sfx_kind = prop_config.scream_sfx_kind;
            if sfx_kind > 0 {
                let sound_id = SoundId::PropCollect(sfx_kind);
                self.play_sound_fx(sound_id, 1.0, 0);
            }

            // TODO: check that this (meaning `collection_idx == 0`) actually hits the last iteration
            // of this loop
            if collection_idx == 0 {
                self.last_attached_prop_name_idx = name_idx;
                self.last_attached_prop = Some(prop_ref.clone());
            }

            // TODO_COMBO: `kat_process_collected_props:111-164` (update collection combo)

            if prop_config.has_treasure_vfx {
                static VFX_DIR: Vec3 = [0.0, 0.0, 0.0];

                let scale = prop.get_aabb_size()[1];

                self.play_vfx(
                    VfxId::Treasure,
                    &self.center,
                    &VFX_DIR,
                    scale,
                    -1,
                    self.player,
                );
            }
        }
    }

    /// Attaches `prop` to the katamari.
    /// offset: 0x28ef0
    fn attach_prop(
        &mut self,
        props: &PropsState,
        prop_ref: &PropRef,
        prop: &mut Prop,
        mission_state: &MissionState,
        global: &mut GlobalState,
    ) {
        // early return if the prop is already attached
        if prop.get_global_state() == PropGlobalState::Attached {
            return;
        }

        // immediately force disable dummy hit objects when attached and early return.
        // the four name index constants below are dummy putter hit and dummy hit 01 to 03.
        match prop.get_name_idx() {
            0x89 | 0x277 | 499 | 0x26b => {
                return prop.set_disabled(1);
            }
            _ => (),
        }

        // TODO: `kat_attach_props:52-53` (increment global counter of # attached props for some reason)

        // update the score in theme object constellations
        if mission_state.mission_config.game_type == GameType::NumThemeProps {
            // increment gemini score if the attached prop's twin is already attached
            if mission_state.mission == Mission::Gemini {
                if let Some(twin_ctrl_idx) = prop.get_twin() {
                    if let Some(twin_ref) = props.get_prop(twin_ctrl_idx as usize) {
                        if twin_ref.borrow().is_attached() {
                            global.catch_count_b += 1;
                        }
                    }
                }
            } else {
                // in other constellations, increment the score if the prop's name index belongs to
                // the mission's list of applicable name indices
                if let Some(name_indices) = &mission_state.mission_config.theme_prop_names {
                    if name_indices.contains(&prop.get_name_idx()) {
                        global.catch_count_b += 1;
                    }
                }
            }
        }

        prop.attach_to_kat(&self);
        self.vol_m3 += self.attach_vol_penalty * prop.get_attach_vol_m3();

        // update collection order linked list
        if self.first_attached_prop.is_none() {
            self.first_attached_prop = Some(prop_ref.clone());
        }
        self.attached_props.push(prop_ref.clone());

        // compute the unit vector from this katamari to `prop`
        let prop_pos = prop.get_position().clone();
        let mut kat_to_prop_unit = vec3_from!(-, prop_pos, self.center);
        vec3_inplace_normalize(&mut kat_to_prop_unit);

        let mut max_ray_similarity = -1.0;
        let mut nearest_ray_idx = None;
        for (ray_idx, ray) in self.collision_rays.iter().enumerate() {
            let ray_similarity = vec3::dot(&ray.ray_local_unit, &kat_to_prop_unit);
            if ray_similarity > max_ray_similarity {
                max_ray_similarity = ray_similarity;
                nearest_ray_idx = Some(ray_idx as u16);
            }
        }

        prop.set_nearest_kat_ray_idx(nearest_ray_idx)
    }

    /// Check for surface collisions along the katamari's collision rays, which include:
    ///   - the "collision rays" pointing radially outwards from the katamari center
    ///   - the "shell" of 5 rays along the top of the katamari which point in the direction of movement
    /// offset: 0x13e70
    fn compute_surface_contacts(&mut self) {
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
        let mut moved_fast_hit_shell = false;

        if self.physics_flags.moved_fast {
            let mut shell_inits: [Vec3; 5] = Default::default();
            let mut shell_ends: [Vec3; 5] = Default::default();

            let shell_init_base = &self.last_center;
            let shell_end_base = vec3_from!(+, self.center, self.shell_vec);

            vec3::copy(&mut shell_inits[0], shell_init_base);
            vec3::add(&mut shell_inits[1], shell_init_base, &self.shell_top);
            vec3::add(&mut shell_inits[2], shell_init_base, &self.shell_left);
            vec3::add(&mut shell_inits[3], shell_init_base, &self.shell_right);
            vec3::add(&mut shell_inits[4], shell_init_base, &self.shell_bottom);

            vec3::copy(&mut shell_ends[0], &shell_end_base);
            vec3::add(&mut shell_ends[1], &shell_end_base, &self.shell_top);
            vec3::add(&mut shell_ends[2], &shell_end_base, &self.shell_left);
            vec3::add(&mut shell_ends[3], &shell_end_base, &self.shell_right);
            vec3::add(&mut shell_ends[4], &shell_end_base, &self.shell_bottom);

            self.debug_draw_shell_rays(&shell_inits, &shell_ends);

            let max_idx = match self.physics_flags.vs_attack {
                true => 1,
                false => 5,
            };

            for i in 0..max_idx {
                self.raycast_state.load_ray(&shell_inits[i], &shell_ends[i]);
                let found_hit = self
                    .raycast_state
                    .find_nearest_unity_hit(RaycastCallType::Objects, false);

                if !found_hit {
                    continue;
                }

                self.physics_flags.airborne = true;
                self.physics_flags.moved_fast_shell_hit = true;
                self.physics_flags.moved_fast_shell_hit_0x1d = true;

                let mut shell_base_pt = vec3::create();
                match i {
                    0 => (),
                    1 => {
                        vec3::copy(&mut shell_base_pt, &self.shell_top);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Top);
                    }
                    2 => {
                        vec3::copy(&mut shell_base_pt, &self.shell_left);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Left);
                    }
                    3 => {
                        vec3::copy(&mut shell_base_pt, &self.shell_right);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Right);
                    }
                    4 => {
                        vec3::copy(&mut shell_base_pt, &self.shell_bottom);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Bottom);
                    }
                    _ => {
                        panic_log!("should never happen");
                    }
                }

                if let Some(hit) = self.raycast_state.get_closest_hit_mut() {
                    let shell_ray_idx = match self.physics_flags.hit_shell_ray {
                        Some(shell_ray) => -(shell_ray as i16),
                        None => 0,
                    };
                    vec3_inplace_subtract_vec(&mut hit.impact_point, &shell_base_pt);
                    vec3_inplace_subtract_vec(&mut self.raycast_state.point1, &shell_base_pt);

                    mark_address!("0x14558");
                    let record_result = self.record_surface_contact(shell_ray_idx, None);
                    moved_fast_hit_shell = true;

                    if record_result != RecordSurfaceContactResult::ShellTop {
                        self.physics_flags.moved_fast_shell_hit = true;
                    }
                }
            }
        } else {
            // TODO_PARAM: make 0.15 a katamari param
            let SHELL_RAY_RADIUS_MULT = 0.15;

            // each shell ray has the same direction and magnitude.
            // the magnitude is a constant multiple of the current katamari radius.
            // the direction is the katamari's movement vector over the previous frame (`self.delta_pos`).
            let shell_ray_len = self.radius_cm * SHELL_RAY_RADIUS_MULT;
            let shell_initial_base = &self.last_center;
            let mut shell_end = vec3::create();
            vec3::scale_and_add(
                &mut shell_end,
                &self.center,
                &self.delta_pos_unit,
                shell_ray_len,
            );

            let mut shell_inits: [Vec3; 5] = Default::default();
            let mut shell_ends: [Vec3; 5] = Default::default();

            vec3::add(&mut shell_inits[0], &self.shell_top, shell_initial_base);
            vec3::add(&mut shell_inits[1], &self.shell_left, shell_initial_base);
            vec3::add(&mut shell_inits[2], &self.shell_right, shell_initial_base);
            vec3::add(
                &mut shell_inits[3],
                &self.shell_top_left,
                shell_initial_base,
            );
            vec3::add(
                &mut shell_inits[4],
                &self.shell_top_right,
                shell_initial_base,
            );

            vec3::add(&mut shell_ends[0], &self.shell_top, &shell_end);
            vec3::add(&mut shell_ends[1], &self.shell_left, &shell_end);
            vec3::add(&mut shell_ends[2], &self.shell_right, &shell_end);
            vec3::add(&mut shell_ends[3], &self.shell_top_left, &shell_end);
            vec3::add(&mut shell_ends[4], &self.shell_top_right, &shell_end);

            self.debug_draw_shell_rays(&shell_inits, &shell_ends);

            for i in 0..5 {
                self.raycast_state.load_ray(&shell_inits[i], &shell_ends[i]);
                let found_hit = self
                    .raycast_state
                    .find_nearest_unity_hit(RaycastCallType::Objects, false);

                if !found_hit {
                    continue;
                }

                self.physics_flags.kat_hit_surface_maybe_0xd = true;
                let mut shell_base_pt = vec3::create();

                match i {
                    0 => {
                        vec3::copy(&mut shell_base_pt, &self.shell_top);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Top);
                    }
                    1 => {
                        vec3::copy(&mut shell_base_pt, &self.shell_left);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Left);
                    }
                    2 => {
                        vec3::copy(&mut shell_base_pt, &self.shell_right);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::Right);
                    }
                    3 => {
                        vec3::copy(&mut shell_base_pt, &self.shell_top_left);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::TopLeft);
                    }
                    4 => {
                        vec3::copy(&mut shell_base_pt, &self.shell_top_right);
                        self.physics_flags.hit_shell_ray = Some(ray::ShellRay::TopRight);
                    }
                    _ => {
                        panic!();
                    }
                }

                if let Some(hit) = self.raycast_state.get_closest_hit_mut() {
                    let shell_ray_idx = -(self.physics_flags.hit_shell_ray.unwrap() as i16);
                    vec3_inplace_subtract_vec(&mut hit.impact_point, &shell_base_pt);
                    vec3_inplace_subtract_vec(&mut self.raycast_state.point1, &shell_base_pt);
                    mark_address!("0x14b32");
                    self.record_surface_contact(shell_ray_idx, None);
                }
            }
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
                if moved_fast_hit_shell && !self.last_physics_flags.airborne {
                    self.physics_flags.airborne = false;
                }
                self.record_surface_contact(ray_idx as i16, None);
            }
        }
    }

    /// Returns `None` if the surface contact arose from a shell ray.
    /// If not, then the surface contact arose from a collision ray and the returned value is
    /// type of surface that is contacted (either wall or floor).
    /// offset: 0x133d0
    fn record_surface_contact(
        &mut self,
        ray_idx: i16,
        prop: Option<PropRef>,
    ) -> RecordSurfaceContactResult {
        let hit = self.raycast_state.get_closest_hit().unwrap_or_else(|| {
            panic_log!(
                "`Katamari::record_surface_contact`: tried to record a nonexistent surface contact: {:?}", self.raycast_state
            );
        });

        let mut normal_unit = hit.normal_unit;
        vec3_inplace_zero_small(&mut normal_unit, 1e-05);

        // use the y component of the hit surface's unit normal to decide if it's a wall or floor
        let surface_type = if self.params.surface_normal_y_threshold < normal_unit[1] {
            if ray_idx == -1 || ray_idx == -5 || ray_idx == -6 {
                return RecordSurfaceContactResult::ShellTop;
            }

            self.floor_contact_ray_idxs[self.num_floor_contacts as usize] = ray_idx as i8;
            self.physics_flags.contacts_floor = true;
            RecordSurfaceContactResult::Floor
        } else {
            self.physics_flags.contacts_wall = true;
            RecordSurfaceContactResult::Wall
        };

        let dot = vec3::dot(&normal_unit, &self.raycast_state.ray_unit);
        let ray_clip_len = (1.0 - hit.impact_dist_ratio - self.params.clip_len_constant)
            * self.raycast_state.ray_len;

        let mut clip_normal = vec3::clone(&normal_unit);
        vec3_inplace_scale(&mut clip_normal, dot * ray_clip_len);

        if normal_unit[1] < -0.1 {
            let normal_angle_y = acos_f32(normal_unit[1]);
            if normal_angle_y < 1.047198 {
                self.physics_flags.contacts_down_slanted_ceiling = true;
                if self.physics_flags.contacts_prop_0xa {
                    set_y!(clip_normal, 0.0);
                }
            }
        }

        self.add_surface_contact(
            surface_type.try_into().unwrap(),
            &normal_unit,
            &clip_normal,
            ray_idx,
            prop,
        );

        surface_type
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

    /// Attempt to add a contact surface with the parameters given in the arguments.
    /// The surface won't be added if it is highly similar to an already contacted
    /// surface.
    /// offset: 0x136e0
    fn add_surface_contact(
        &mut self,
        surface_type: SurfaceType,
        normal_unit: &Vec3,
        clip_normal: &Vec3,
        ray_idx: i16,
        prop: Option<PropRef>,
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
        added_surface.normal_unit = *normal_unit;
        added_surface.clip_normal = clip_normal;
        added_surface.ray = self.raycast_state.ray;
        added_surface.contact_point = closest_hit.impact_point;
        added_surface.clip_normal_len = clip_normal_len;
        added_surface.impact_dist_ratio = closest_hit.impact_dist_ratio;
        added_surface.ray_len = self.raycast_state.ray_len;
        added_surface.ray_idx = ray_idx;
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

    /// offset: 0x13930
    fn process_surface_contacts(&mut self) {
        mark_address!("0x13930");

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
                vec3_inplace_add_vec(&mut sum_wall_normals, &wall.normal_unit);
                self.hit_flags.apply_hit_attr(wall.hit_attr);

                if wall.ray_len > max_ray_len {
                    max_ray_len = wall.ray_len;
                }
            }

            vec3::normalize(&mut self.contact_wall_normal_unit, &sum_wall_normals);
            self.compute_contact_wall_clip();
            self.climb_radius_cm = max_ray_len;
        }

        if min_ratio_ray_idx.is_none() || self.physics_flags.grounded_ray_type.is_bottom() {
            // if the primary floor contact point is from the bottom ray:
            self.fc_ray_idx = min_ratio_ray_idx;
            self.fc_ray = min_ratio_ray;
            self.fc_contact_point = min_ratio_contact_pt;

            self.fc_ray_len = match min_ratio_ray_idx {
                None => self.climb_radius_cm,
                // this is a bug in the original sim where it reads back into the katamari struct at garbage:
                // if `ray_idx == -2` -> `self.water_hit_point[3]`
                // if `ray_idx == -3` -> `self.last_num_floor_contacts` and `self.last_num_wall_contacts`,
                //                       both 2-byte ints, concatenated and coerced into a float
                Some(ray_idx) if ray_idx < 0 => 0.0,
                Some(ray_idx) => self.collision_rays[ray_idx as usize].ray_len,
            };
        } else if min_ratio_ray_idx == self.vault_ray_idx {
            // if the primary floor contact point is from the vault ray:
            self.fc_ray_idx = self.vault_ray_idx;
        } else {
            // if the primary floor contact point is from a non-bottom, non-vault ray:
            self.fc_ray_idx = min_ratio_ray_idx;
            self.fc_ray = min_ratio_ray;

            // see the above assignment to `self.fc_ray_len`; the same rationale applies here
            self.fc_ray_len = match min_ratio_ray_idx {
                None => self.climb_radius_cm,
                // this is a bug in the original sim where it reads back into the katamari struct at garbage:
                // if `ray_idx == -2` -> `self.water_hit_point[3]`
                // if `ray_idx == -3` -> `self.last_num_floor_contacts` and `self.last_num_wall_contacts`,
                //                       both 2-byte ints, concatenated and coerced into a float
                Some(ray_idx) if ray_idx < 0 => 0.0,
                Some(ray_idx) => self.collision_rays[ray_idx as usize].ray_len,
            };

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

    fn resolve_being_stuck(&mut self, mission_state: &MissionState, global: &mut GlobalState) {
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
                        self.detach_props(mission_state, global, lost_vol_mult * self.vol_m3, 0.5);
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

    /// Detach props, starting from the most recently attached, by "damaging" the
    /// attach life of props until `lost_life` is exhausted.
    /// offset: 0x26f10
    fn detach_props(
        &mut self,
        mission_state: &MissionState,
        global: &mut GlobalState,
        lost_life: f32,
        detach_speed: f32,
    ) {
        // TODO_PARAM
        let MAX_PROPS_LOST_FROM_BONK = 5;

        let mut remaining_life = lost_life;
        let mut remaining_props = MAX_PROPS_LOST_FROM_BONK;

        // TODO_PERF: don't clone this
        let mut attached_props = self.attached_props.clone();
        let mut detached_ctrl_idxs = vec![];

        for prop_ref in attached_props.iter_mut().rev() {
            let should_detach;
            let prop_vol;
            let ctrl_idx;
            {
                let mut prop = prop_ref.borrow_mut();
                let prop_attach_life = prop.get_attach_life();
                prop_vol = prop.get_compare_vol_m3();
                ctrl_idx = prop.get_ctrl_idx();

                if prop.is_disabled() {
                    continue;
                }

                should_detach = prop_attach_life > remaining_life;
                if !should_detach {
                    prop.set_attach_life(prop_attach_life - remaining_life);
                }
            }

            if should_detach {
                self.detach_prop(mission_state, global, prop_ref, detach_speed);
                detached_ctrl_idxs.push(ctrl_idx);
                remaining_life -= prop_vol;
                if remaining_life <= 0.0 {
                    return;
                }
            }

            remaining_props -= 1;
            if remaining_props < 1 {
                return;
            }
        }
    }

    /// Detach a prop from the katamari with the speed `detach_speed`.
    /// offset: 0x27000
    fn detach_prop(
        &mut self,
        mission_state: &MissionState,
        global: &mut GlobalState,
        prop_ref: &mut PropRef,
        detach_speed: f32,
    ) {
        prop_ref
            .borrow_mut()
            .detach_from_katamari(mission_state, global);

        let mut prop_init_vel = vec3::create();
        self.compute_detached_prop_init_vel(&mut prop_init_vel, prop_ref, detach_speed);

        prop_ref.borrow_mut().apply_trajectory(
            &prop_init_vel,
            PropTrajectoryType::Normal,
            self.airborne_prop_gravity,
        );

        self.detached_props_from_bonk += 1;
    }

    /// Compute the initial velocity of a prop when it is detached from the katamari.
    /// offset: 0x27170
    fn compute_detached_prop_init_vel(
        &self,
        out_prop_init_vel: &mut Vec3,
        prop_ref: &mut PropRef,
        detach_speed: f32,
    ) {
        let mut prop = prop_ref.borrow_mut();

        let kat_to_prop_lateral_unit = vec3_unit_xz!(&vec3_from!(-, prop.pos, self.center));

        // TODO_PARAM: these three constants
        let prop_speed = (detach_speed * 0.4 + 0.6) * self.max_forwards_speed * 1.63;

        let id = mat4::create();
        let mut rot_mat = mat4::create();
        // TODO_PARAM: this angle
        mat4::rotate_x(&mut rot_mat, &id, FRAC_5PI_12);

        let mut local_vel_unit = vec3::create();
        vec3::transform_mat4(&mut local_vel_unit, &[0.0, 0.0, 1.0], &rot_mat);

        *out_prop_init_vel = [
            local_vel_unit[0] * kat_to_prop_lateral_unit[0],
            -local_vel_unit[1],
            local_vel_unit[2] * kat_to_prop_lateral_unit[2],
        ];
        vec3_inplace_scale(out_prop_init_vel, prop_speed);

        // TODO_PARAM
        prop.intangible_timer = 10;
    }

    /// Update the katamari's vault and climbing state.
    /// offset: 0x14c80
    fn update_vault_and_climb(
        &mut self,
        prince: &mut Prince,
        camera: &Camera,
        global: &mut GlobalState,
        mission_state: &MissionState,
    ) {
        mark_call!("update_vault_and_climb", self.debug_should_log());

        if self.physics_flags.hit_shell_ray == Some(ray::ShellRay::Top)
            && self.physics_flags.contacts_floor
            && !self.physics_flags.contacts_wall
            && self.num_floor_contacts == 1
        {
            self.physics_flags.contacts_floor = false;
            vec3::zero(&mut self.contact_floor_clip);
        }

        // TODO_TURNTABLE: `kat_apply_turntable_contact()`
        self.apply_clip_translation();

        if self.physics_flags.in_water
            && !self.last_physics_flags.in_water
            && self.physics_flags.airborne
        {
            self.play_sound_fx(SoundId::EnterWater, 1.0, 0);
        }

        if self.physics_flags.grounded_ray_type.is_not_bottom() {
            vec3_inplace_subtract_vec(&mut self.vault_contact_point, &self.contact_wall_clip);
        }

        let mut _was_climbing = false;
        self.physics_flags.hit_ground_fast = false;
        'main: {
            if self.num_wall_contacts + self.num_floor_contacts == 0 {
                // if the katamari isn't contacting any surfaces
                _was_climbing = self.physics_flags.climbing;

                if _was_climbing && self.is_climbing_0x898 >= 1 {
                    // continue wallclimb
                    self.is_climbing_0x898 -= 1;
                    if self.physics_flags.at_max_climb_height && prince.input_avg_push_len > 0.99 {
                        break 'main;
                    }
                } else {
                    // end wallclimb
                    if _was_climbing {
                        // if a wallclimb was ongoing, initiate a cooldown for the next one
                        // and reset the wallclimb duration
                        self.wallclimb_cooldown_timer = 10;
                        self.climb_ticks = 0;
                    }
                    self.physics_flags.climbing = false;
                    self.physics_flags.at_max_climb_height = false;
                    self.climb_init_y = 0.0;
                    self.climb_max_height_duration = 0;
                }

                self.fc_ray_len = self.radius_cm;
                self.physics_flags.grounded_ray_type = GroundedRay::Bottom;
                self.vault_ray_idx = None;

                // update durations of falling and being airborne
                if !self.physics_flags.airborne {
                    self.airborne_ticks = 0;
                    self.falling_ticks = 0;
                } else {
                    self.airborne_ticks += 1;
                    if self.is_falling() {
                        self.falling_ticks += 1;
                    }
                }

                // since we're not contacting any surfaces, we're airborne
                self.physics_flags.airborne = true;

                if _was_climbing {
                    self.airborne_ticks += 1;
                    if self.is_falling() {
                        self.falling_ticks += 1;
                    }
                    self.physics_flags.airborne = false;
                }
            } else {
                // if contacting at least one surface:
                self.physics_flags.unknown_0x20 = false;
                self.update_wall_contacts(prince, camera, global, mission_state);

                if !self.physics_flags.contacts_floor
                    && self.physics_flags.contacts_wall
                    && !self.physics_flags.climbing
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

                mark_address!("0x14e8f");
                let try_init_vault_result = self.try_init_vault();
                mark_address!("0x14e97");

                match try_init_vault_result {
                    // case 1: the katamari isn't vaulting
                    TryInitVaultResult::NoVault => self.set_bottom_ray_contact(),

                    // case 2: the katamari is starting a new vault
                    TryInitVaultResult::InitVault => {
                        let ray_idx = self.vault_ray_idx.unwrap();
                        let ray_type = self.ray_type_by_idx(ray_idx);
                        self.physics_flags.grounded_ray_type = ray_type.into();

                        if ray_type == Some(KatCollisionRayType::Prop) {
                            let ray = &self.collision_rays[ray_idx as usize];
                            vec3::scale(
                                &mut self.prop_vault_ray_unit,
                                &ray.ray_local_unit,
                                ray.ray_len,
                            );
                        }

                        // reset the `vault_transform` to the identity
                        mat4::identity(&mut self.vault_transform);

                        // save a copy of the katamari's transform when the vault started
                        mat4::copy(&mut self.init_vault_transform, &self.transform);

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
                            self.play_sound_fx(SoundId::Vault, ray_len_t, 0);
                        }
                    }

                    // case 3: continuing a vault that was initialized on a previous tick
                    TryInitVaultResult::OldVault => {
                        if let Some(ray_idx) = self.vault_ray_idx {
                            // update the grounded ray type based on the vaulted ray's index
                            self.physics_flags.grounded_ray_type =
                                self.ray_type_by_idx(ray_idx).into();

                            // (??) i guess this is pushing the katamari up out of the ground
                            // if the vault ray is clipped too far into the ground?
                            if self.clip_translation[1] <= -1.0 {
                                self.vault_contact_point[1] -= self.clip_translation[1];
                            }
                        }
                    }
                }

                _was_climbing = self.physics_flags.climbing;
                if _was_climbing {
                    self.physics_flags.grounded_ray_type = GroundedRay::Bottom;
                }
            }
        }

        if self.physics_flags.airborne {
            if _was_climbing && self.is_climbing_0x898 > 0 {
                self.is_climbing_0x898 -= 1;
            } else {
                if _was_climbing {
                    self.wallclimb_cooldown_timer = 10;
                    self.climb_ticks = 0;
                }

                self.physics_flags.climbing = false;
                self.physics_flags.at_max_climb_height = false;
                self.climb_init_y = 0.0;
                self.climb_max_height_duration = 0;
            }
        }

        if self.physics_flags.hit_ground_fast {
            if !self.physics_flags.in_water {
                let play_hit_ground_sfx = match self.physics_flags.grounded_ray_type {
                    GroundedRay::Bottom => true,
                    GroundedRay::Prop => false,
                    GroundedRay::Mesh => {
                        self.collision_rays[self.vault_ray_idx.unwrap() as usize].ray_len
                            / self.radius_cm
                            > 1.05
                    }
                };

                if play_hit_ground_sfx {
                    self.play_sound_fx(SoundId::HitGround, 1.0, 0);
                }
            }
        }
    }

    /// offset: 0x15950
    fn update_wall_contacts(
        &mut self,
        prince: &mut Prince,
        camera: &Camera,
        global: &mut GlobalState,
        mission_state: &MissionState,
    ) {
        mark_call!("update_wall_contacts", self.debug_should_log());

        if self.physics_flags.climbing {
            if self.can_climb_wall_contact(prince) {
                return self.maintain_wallclimb();
            } else {
                return self.end_wall_climb();
            }
        }

        if self.speed <= 0.0 {
            return self.play_bonk_fx(false);
        }

        let fast_collision = self.physics_flags.moved_fast_shell_hit_0x1d;

        // TODO_VS: `vs_attack` check here
        let contacts_wall = self.num_wall_contacts > 0;
        let should_halve_speed = fast_collision && contacts_wall;
        let can_bonk_and_lose_props = !fast_collision || should_halve_speed;
        let landed_from_fast_fall = !(fast_collision && !contacts_wall && self.falling_ticks < 10);
        let flag_d_false_in_1p = false;

        let mut surface_normal_unit = [0.0; 3];
        let impact_similarity;
        let impact_force;
        let mut impact_volume;

        if !self.physics_flags.airborne {
            // if the katamari contacts a surface:

            // compute unit lateral velocity
            let lateral_vel_unit = vec3_unit_xz!(self.velocity.vel_accel);

            // `surface_normal_unit` is the *lateral* contact wall normal if not a fast collision,
            // and the contact floor normal otherwise.
            if !flag_d_false_in_1p || !fast_collision {
                vec3::copy(&mut surface_normal_unit, &self.contact_wall_normal_unit);
                set_y!(surface_normal_unit, 0.0);
            } else {
                vec3::copy(&mut surface_normal_unit, &self.contact_floor_normal_unit);
            }
            vec3_inplace_normalize(&mut surface_normal_unit);

            if vec3::dot(&lateral_vel_unit, &surface_normal_unit) > 0.0 {
                return;
            }

            impact_force = self.compute_impact_force();
            impact_similarity =
                self.compute_impact_similarity(&lateral_vel_unit, &surface_normal_unit);
            impact_volume = impact_force * impact_similarity;

            // TODO_VIBRATION: `kat_update_wall_contacts:169-171` (call vibration callback)

            let can_climb = self.can_climb_wall_contact(prince);

            if can_climb {
                return self.maintain_wallclimb();
            } else {
                self.end_wall_climb();
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
            } else if !contacts_wall {
                vec3::copy(&mut surface_normal_unit, &self.contact_floor_normal_unit);
            } else {
                vec3::copy(&mut surface_normal_unit, &self.contact_wall_normal_unit);
            }
            vec3_inplace_normalize(&mut surface_normal_unit);

            // if the katamari's falling velocity is within 90 degrees of to the `surface_normal_unit`, do nothing.
            if vec3::dot(&self.velocity.vel_accel_grav_unit, &surface_normal_unit) > 0.0 {
                return;
            }

            let check_a = self.physics_flags.contacts_floor && !fast_collision;
            let check_b = self.physics_flags.wheel_spin || self.airborne_ticks < 5;
            if check_a && check_b {
                set_y!(self.velocity.vel_accel, 0.0);
                vec3::zero(&mut self.init_bonk_velocity);
                return;
            }

            impact_force = self.compute_impact_force();
            impact_similarity = self.compute_impact_similarity(
                &self.velocity.vel_accel_grav_unit,
                &surface_normal_unit,
            );
            impact_volume = impact_force * impact_similarity;

            if self.physics_flags.moved_fast_shell_hit_0x14
                && self.physics_flags.grounded_ray_type.is_not_bottom()
            {
                impact_volume = 0.0;
            }

            // TODO_PARAM
            let magic_num_0x7b264 = 70.0;
            let magic_num_0x71580 = 0.1;
            let falling_tick_ratio = self.falling_ticks as f32 / magic_num_0x7b264;
            if landed_from_fast_fall {
                // TODO_VIBRATION: `kat_update_wall_contacts:218-220` (vibration)
                if !self.physics_flags.contacts_wall
                    && self.physics_flags.airborne
                    && falling_tick_ratio >= magic_num_0x71580
                {
                    if !self.physics_flags.in_water {
                        // TODO_FX: `kat_update_wall_contacts:224-246` (emit smoke)
                    }
                    self.physics_flags.hit_ground_fast = true;
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

        if vec3::len(&surface_normal_unit) <= 1e-05 {
            return;
        }

        if should_halve_speed {
            self.speed *= 0.5;
        }

        if self.physics_flags.contacts_floor || self.physics_flags.contacts_wall {
            self.airborne_ticks = 0;
            self.falling_ticks = 0;
        }

        if impact_similarity <= 0.0 {
            if impact_volume > 0.0 {
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

        // TODO_PARAM
        let param_xz_elasticity = 0.95;
        let param_min_speed_ratio = 0.3;
        let param_min_impact_similarity = 0.3;
        let param_sound_cooldown_ms = 0xa5;

        let mut play_wall_bonk_sound = false;
        let mut speed = self.speed;

        'elastic_collision: {
            if self.hit_flags.no_reaction_no_slope {
                set_y!(self.velocity.vel_accel, 0.0);
                vec3::normalize(&mut self.velocity.vel_accel_unit, &self.velocity.vel_accel);
                if !fast_collision {
                    break 'elastic_collision;
                }
            }

            let mut refl = [0.0; 3];
            vec3_reflection(
                &mut refl,
                &self.velocity.vel_accel_grav_unit,
                &surface_normal_unit,
            );
            vec3_inplace_scale(&mut refl, -1.0);

            if !self.physics_flags.contacts_wall {
                if should_halve_speed {
                } else if !self.physics_flags.vs_attack {
                    refl[0] *= param_xz_elasticity;
                    refl[1] *= self.y_elasticity;
                    refl[2] *= param_xz_elasticity;
                } else {
                    // TODO_VS: `kat_update_wall_contacts:324-326`
                }
            } else {
                let speed_ratio = self.speed / self.scaled_params.base_max_speed;

                play_wall_bonk_sound = speed_ratio > param_min_speed_ratio
                    && impact_similarity > param_min_impact_similarity
                    && global.game_time_ms - self.last_wall_bonk_game_time_ms
                        > param_sound_cooldown_ms;

                speed *= self.y_elasticity;
            }

            self.physics_flags.bonked = true;
            vec3::scale(&mut self.init_bonk_velocity, &refl, -speed);
        }

        prince.end_spin_and_boost(self);

        if !can_bonk_and_lose_props {
            return;
        }

        // TODO_PARAM
        // 166ms cooldown on bonks, which is 1/6 of a second
        let BONK_COOLDOWN_MS = 0xa6;

        // if it's been too soon since we bonked, play the FX but don't actually lose any props
        if global.game_time_ms - self.last_wall_bonk_game_time_ms < BONK_COOLDOWN_MS {
            return self.play_bonk_fx(false);
        }

        // at this point, the katamari has bonked a wall
        self.last_wall_bonk_game_time_ms = global.game_time_ms;
        self.detached_props_from_bonk = 0;

        let not_climbing = !self.physics_flags.climbing && !self.last_physics_flags.climbing;

        if not_climbing && impact_volume > 0.0 {
            // TODO_LOW: `kat_begin_screen_shake()`
            let can_lose_props = !camera.state.cam_eff_1P && !global.map_change_mode;

            if can_lose_props {
                self.lose_props_from_bonk(mission_state, global, impact_volume);
            }

            if self.detached_props_from_bonk > 0 {
                let sound_id = mission_state
                    .stage_config
                    .get_lose_prop_sound_id(self.diam_trunc_mm as u32);
                self.play_sound_fx(sound_id.into(), 1.0, 0);
                self.play_bonk_fx(false);
            }
        }

        if play_wall_bonk_sound {
            self.play_wall_bonk_sound(mission_state, impact_force)
        }

        self.play_bonk_fx(false);
    }

    fn play_wall_bonk_sound(&self, mission_state: &MissionState, impact_force: f32) {
        // TODO_LOW: return if gamemode is 4
        let sound_id = mission_state
            .stage_config
            .get_wall_bonk_sound_id(self.diam_trunc_mm as u32);
        let volume = impact_force.clamp(0.5, 1.0);
        self.play_sound_fx(sound_id.into(), volume, 0);
    }

    fn lose_props_from_bonk(
        &mut self,
        mission_state: &MissionState,
        global: &mut GlobalState,
        impact_volume: f32,
    ) {
        // TODO_PARAM
        let MIN_IMPACT_VOLUME_TO_LOSE_PROPS = 0.28;
        let _MAX_PROPS_LOST_FROM_BONK = 5;
        let MIN_IMPACT_SPEED_SCALE = 0.98;
        let LOST_LIFE_SCALE = 0.03;

        if impact_volume <= MIN_IMPACT_VOLUME_TO_LOSE_PROPS {
            return;
        }

        self.physics_flags.detaching_props = true;

        let min_impact_speed = self.base_speed * MIN_IMPACT_SPEED_SCALE;
        let impact_speed = min!(self.base_speed, self.speed);
        let extra_speed = max!(
            0.0,
            (impact_speed - min_impact_speed) / (self.base_speed - min_impact_speed)
        );

        if impact_speed <= min_impact_speed {
            return;
        }

        let impact_volume_t = inv_lerp!(impact_volume, MIN_IMPACT_VOLUME_TO_LOSE_PROPS, 1.0);
        let lost_life = LOST_LIFE_SCALE * self.vol_m3 * impact_volume_t * extra_speed;

        if mission_state.mission_config.game_type == GameType::NumThemeProps {
            // TODO_THEME: `kat_lose_props_from_bonk:44-87`
        } else {
            self.detach_props(mission_state, global, lost_life, impact_volume_t);
        }
    }

    /// Returns `true` if the katamari can climb its current wall contact (which could be either
    /// a map surface or a prop surface). This covers both when a new wallclimb could start, or
    /// when the current wallclimb should continue.
    /// offset: 0x16540
    fn can_climb_wall_contact(&mut self, prince: &Prince) -> bool {
        mark_call!("can_climb_wall_contact", self.debug_should_log());
        let mut perform_checks = true;
        let mut check_multiple_walls = true;

        if self.num_wall_contacts == 1
            && self.num_floor_contacts == 0
            && self.contact_prop.is_some()
        {
            // if all of the following hold:
            //   - the katamari is colliding with a prop
            //   - the katamari's is colliding with exactly one wall
            //   - the katamari isn't colliding with a floor
            let prop_ref = self.contact_prop.as_ref().unwrap();
            let prop = prop_ref.borrow();

            let mut aabb_min_world = vec3::create();
            vec3::transform_mat4(
                &mut aabb_min_world,
                prop.get_aabb_min_point(),
                prop.get_unattached_transform(),
            );

            // TODO_DOC: what does this condition mean
            if aabb_min_world[1] > self.bottom[1] - self.max_wallclimb_height_gain {
                self.hit_flags.wall_climb_free = true;
                perform_checks = false;
            }
        }

        if perform_checks {
            if self.hit_flags.wall_climb_disabled {
                return false;
            }
            if self.physics_flags.hit_shell_ray.is_some() {
                return false;
            }
            if self.hit_flags.small_ledge_climb {
                check_multiple_walls = false;
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
                if self.physics_flags.incline_move_type != KatInclineMoveType::Flatground {
                    return false;
                }
            }
        }

        if check_multiple_walls && self.num_wall_contacts > 1 {
            return false;
        }

        // don't start a new wall climb if the katamari doesn't currently contact a wall
        if !self.physics_flags.contacts_wall && !self.physics_flags.climbing {
            return false;
        }

        // NOTE: this deviates from the logic of the original simulation. See the
        // documentation of `most_recent_wall_normal`.
        let most_recent_wall_normal = if self.num_wall_contacts > 0 {
            self.most_recent_wall_normal = Some(self.hit_walls[0].normal_unit);
            self.hit_walls[0].normal_unit
        } else {
            self.most_recent_wall_normal.unwrap()
        };

        // check that the angle between the katamari's push velocity and the wall normal are close
        // enough to admit a wallclimb. since the wall normal is actually pointing *out* of the wall,
        // we need to throw in a negative somewhere in there.
        let similarity = vec3::dot(
            &self.velocity.push_vel_on_floor_unit,
            &most_recent_wall_normal,
        );
        let angle = acos_f32(-similarity);
        if angle > self.params.max_wallclimb_angle {
            return false;
        }

        // check that the input push and angle are suitable for a wallclimb
        if !prince.has_wallclimb_input() {
            return false;
        }

        if !self.physics_flags.climbing {
            // if the katamari isn't already wallclimbing:
            // start a new wallclimb
            self.climb_normal_unit = self.contact_wall_normal_unit;
            self.climb_speed = 0.0;
            self.physics_flags.at_max_climb_height = false;

            // TODO_DOC
            // TODO_PARAM: factor out magic number as param
            if !self.hit_flags.wall_climb_free && self.base_speed * 0.95 < self.speed {
                return false;
            }
        }

        // finally, check the similarity between the katamari's push velocity and the wall
        // normal. if the two vectors are similar enough, we can start a wallclimb.
        let mut lateral_push_vel = self.velocity.push_vel_on_floor_unit;
        set_y!(lateral_push_vel, 0.0);
        vec3_inplace_scale(&mut lateral_push_vel, -1.0);
        vec3_inplace_normalize(&mut lateral_push_vel);

        let mut lateral_wallclimb_normal = self.climb_normal_unit;
        set_y!(lateral_wallclimb_normal, 0.0);
        vec3_inplace_normalize(&mut lateral_wallclimb_normal);

        let push_to_wall_similarity = vec3::dot(&lateral_push_vel, &lateral_wallclimb_normal);
        return push_to_wall_similarity >= self.params.min_wallclimb_similarity;
    }

    /// Maintain a wallclimb state without actually moving the katamari. It's not
    /// really clear what this is for.
    /// offset: 0x16930
    fn maintain_wallclimb(&mut self) {
        mark_call!("maintain_wallclimb", self.debug_should_log());

        if !self.physics_flags.at_max_climb_height {
            self.is_climbing_0x898 = 1;
        }

        self.wallclimb_cooldown_timer = 0;

        if !self.physics_flags.climbing {
            self.climb_init_y = self.center[1];
            self.climb_init_radius = self.climb_radius_cm;
        }

        self.physics_flags.climbing = true;
        vec3::zero(&mut self.init_bonk_velocity);
        self.physics_flags.grounded_ray_type = GroundedRay::Bottom;
        self.vault_ray_idx = None;
        self.fc_ray_idx = None;
    }

    /// offset: 0x21e50
    pub fn update_climb_position(&mut self) {
        mark_call!("update_climb_position", self.debug_should_log());

        // TODO_PARAM
        let MAX_FRAMES_AT_MAX_WALLCLIMB_HEIGHT = 10;
        let WALLCLIMB_VFX_DELAY_FRAMES = 0x1e;
        let WALLCLIMB_ACCEL = 0.1;
        // the max wallclimb speed is this value times katamari diameter
        let MAX_WALLCLIMB_SPEED_DIAMS = 0.015;

        if self.physics_flags.at_max_climb_height {
            self.climb_max_height_duration += 1;
            if self.climb_max_height_duration > MAX_FRAMES_AT_MAX_WALLCLIMB_HEIGHT {
                self.end_wall_climb();
            }
            self.physics_flags.at_max_climb_height = true;
            return;
        }

        let height_gain = max!(0.0, self.center[1] - self.climb_init_y);

        if !self.hit_flags.small_ledge_climb && self.max_wallclimb_height_gain <= height_gain {
            self.climb_max_height_duration += 1;
            if self.climb_max_height_duration > MAX_FRAMES_AT_MAX_WALLCLIMB_HEIGHT {
                self.end_wall_climb();
            }
            self.physics_flags.at_max_climb_height = true;
            return;
        }

        self.climb_ticks += 1;

        if self.climb_ticks == WALLCLIMB_VFX_DELAY_FRAMES {
            static VFX_DIR: Vec3 = [0.0, 0.0, 0.0];

            self.play_vfx(VfxId::Climb, &self.center, &VFX_DIR, self.diam_cm, -1, 0);
        }

        let max_wallclimb_speed = self.diam_cm * MAX_WALLCLIMB_SPEED_DIAMS;
        self.climb_speed += WALLCLIMB_ACCEL;
        self.climb_speed = min!(self.climb_speed, max_wallclimb_speed);

        let delta_y = if !self.hit_flags.small_ledge_climb
            && self.max_wallclimb_height_gain < height_gain + self.climb_speed
        {
            self.climb_max_height_duration = 0;
            self.physics_flags.at_max_climb_height = true;
            self.max_wallclimb_height_gain - height_gain
        } else {
            self.climb_speed
        };

        self.center[1] += delta_y;
    }

    /// End the current wallclimb, if one is ongoing.
    /// offset: 0x12ca0
    fn end_wall_climb(&mut self) {
        if self.physics_flags.climbing {
            if self.is_climbing_0x898 > 0 {
                return self.is_climbing_0x898 -= 1;
            }
            self.climb_ticks = 0;
            self.wallclimb_cooldown_timer = 10;
        }

        self.physics_flags.climbing = false;
        self.physics_flags.at_max_climb_height = false;
        self.climb_init_y = 0.0;
        self.climb_max_height_duration = 0;
    }

    /// (??)
    /// offset: 0x16ed0
    fn compute_impact_force(&self) -> f32 {
        if !self.physics_flags.moved_fast_shell_hit_0x1d && self.physics_flags.airborne {
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

        // TODO_PARAM
        return ((speed_ratio - 0.25) * 4.0).clamp(0.0, 1.0);
    }

    /// (??)
    /// offset: 0x16df0
    fn compute_impact_similarity(&self, kat_vel: &Vec3, surface_normal: &Vec3) -> f32 {
        if self.last_physics_flags.climbing {
            return 0.0;
        }

        if self.physics_flags.vs_attack && self.physics_flags.contacts_wall {
            return 1.0;
        }

        let impact_vel = if !self.physics_flags.airborne || !self.physics_flags.contacts_floor {
            &kat_vel
        } else {
            // if airborne and contacting a floor, force the impact to be straight
            &VEC3_Y_NEG
        };

        let similarity = vec3::dot(impact_vel, surface_normal);

        if similarity >= 0.0 {
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
        if self.fc_ray_idx.is_none() || self.fc_ray_idx.unwrap() < 0 {
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
        self.vault_ray_len_radii = vault_ray_len_radii;
        self.vault_ray_idx = self.fc_ray_idx;
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

        let mut fc_proj_floor = [0.0; 3];
        let mut fc_rej_floor = [0.0; 3];
        vec3_projection(
            &mut fc_proj_floor,
            &mut fc_rej_floor,
            &fc_ray_unit,
            &self.contact_floor_normal_unit,
        );
        vec3_inplace_zero_small(&mut fc_rej_floor, 1e-05);

        // transform the angle between the rejections:
        //   [0, PI/2] -> [1, 0]
        //   [PI/2, PI] -> 0

        let rej_similarity = vec3::dot(&vel_rej_floor, &fc_rej_floor); // [-1, 1]
                                                                       // let rej_angle = acos_f32(rej_similarity); // [PI, 0]
                                                                       // self.vault_rej_angle_t = inv_lerp_clamp!(rej_angle, 0.0, FRAC_PI_2);
        self.vault_rej_angle_t = if rej_similarity < 1.0 {
            let rej_angle = if rej_similarity > -1.0 {
                acos_f32(rej_similarity)
            } else {
                PI
            };
            if rej_angle > FRAC_PI_2 {
                0.0
            } else {
                (FRAC_PI_2 - rej_angle) / FRAC_PI_2
            }
        } else {
            1.0
        };

        // (??) set the initial vault speed
        if FRAC_PI_90 < rej_similarity {
            let speed_t =
                inv_lerp_clamp!(self.speed, self.max_forwards_speed, self.max_boost_speed);
            let ray_len_t = inv_lerp_clamp!(self.fc_ray_len, self.radius_cm, self.max_ray_len);
            let ray_len_k = lerp!(ray_len_t, 1.0, self.params.vault_tuning_0x7b208);
            let k = lerp!(speed_t, ray_len_k, 1.0);

            let mut vel_reflect_floor = [0.0; 3];
            vec3_reflection(&mut vel_reflect_floor, &fc_rej_floor, &vel_rej_floor);
            vec3_inplace_scale(&mut vel_reflect_floor, -1.0);

            let __old_vel = self.velocity.vel_accel;

            let mut vel_accel_unit = [0.0; 3];
            vec3::lerp(
                &mut vel_accel_unit,
                &vel_rej_floor,
                &vel_reflect_floor,
                1.0 - k,
            );
            vec3_inplace_normalize(&mut vel_accel_unit);

            vec3::scale(&mut self.velocity.vel_accel, &vel_accel_unit, self.speed);
        }

        return TryInitVaultResult::InitVault;
    }

    fn another_end_wallclimb(&mut self) {
        self.physics_flags.climbing = false;
        self.physics_flags.at_max_climb_height = false;
        self.climb_init_y = 0.0;
        self.climb_max_height_duration = 0;
    }

    /// offset: 0x169a0
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
                    vec3::normalize(&mut self.stuck_btwn_walls_clip_maybe, &self.delta_pos_unit);
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
