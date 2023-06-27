use std::{
    cell::RefCell,
    rc::{Rc},
};

use gl_matrix::{common::Vec3, mat4, vec3};

use crate::{
    constants::VEC3_ZERO,
    macros::{inv_lerp_clamp, lerp, set_translation, vec3_from},
    math::{vec3_inplace_normalize, vec3_inplace_scale, vec3_inplace_zero_small},
    player::katamari::Katamari,
    props::{
        config::{NamePropConfig},
        prop::{Prop, WeakPropRef},
    },
};

use super::mesh::KAT_MESHES;

/// The extra "shell" collision rays which extend along the top half of the katamari.
/// (see https://discord.com/channels/232268612285497345/805240416894713866/842591732229996544)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellRay {
    Top = 1,
    Left = 2,
    Right = 3,
    Bottom = 4,
    TopLeft = 5,
    TopRight = 6,
}

/// The different types of rays making up the katamari's collision.
/// `Bottom`: the single ray extending directly downwards from the katamari's center.
///           this ray is used to make sure the katamari moves smoothly along the ground
///           when nothing has been picked up to make the katamari's shape oblong.
/// `Mesh`: one of the normal rays comprising the katamari's boundary mesh.
///         picking up an object will extend the mesh ray nearest to where the object was attached.
/// `Prop`: if a prop with a vault point is collected, the katamari will gain a collision ray
///         corresponding to that prop in the direction of one of its vault points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KatCollisionRayType {
    Bottom = 0,
    Mesh = 1,
    Prop = 2,
}

#[derive(Debug, Default, Clone)]
pub struct KatCollisionRay {
    /// The endpoint relative to the katamari
    /// offset: 0x0
    pub endpoint: Vec3,

    /// (??)
    /// offset: 0x10
    pub ray_local: Vec3,

    /// The vector from the katamari center to the ray `endpoint`
    /// offset: 0x20
    pub kat_to_endpoint: Vec3,

    /// The unit vector from the katamari center to the ray endpoint.
    /// offset: 0x30
    pub ray_local_unit: Vec3,

    /// Zero if mesh ray, 0x30 vector if prop ray
    /// offset: 0x40
    pub prop_ray_local_unit: Vec3,

    /// If this ray is induced by a prop, points to that prop
    /// offset: 0x50
    pub prop: Option<WeakPropRef>,

    /// Length of the ray.
    /// offset: 0x58
    pub ray_len: f32,

    /// True if this ray contacts a surface
    /// offset: 0x5d
    pub contacts_surface: bool,
}

impl KatCollisionRay {
    /// Reset the collision ray.
    pub fn reset(&mut self, rad_cm: f32) {
        vec3::zero(&mut self.endpoint);
        vec3::zero(&mut self.ray_local);
        vec3::zero(&mut self.kat_to_endpoint);
        vec3::zero(&mut self.ray_local_unit);
        vec3::zero(&mut self.prop_ray_local_unit);
        self.ray_len = rad_cm;
        self.prop = None;
        self.contacts_surface = false;
    }
}

pub type KatCollisionRays = Vec<KatCollisionRay>;

// pub struct KatCollisionRays {
//     pub bottom: KatCollisionRay,
//     pub mesh: Vec<KatCollisionRay>,
//     pub prop: Vec<KatCollisionRay>,
// }

#[derive(Debug, Default)]
struct PropVaultPoint {
    pub ray_unit: Vec3,
    pub length: f32,
    pub prop: WeakPropRef,
}

impl Katamari {
    /// Resets the katamari's collision rays to their initial state.
    /// This is only called at the start of a mission and after a royal warp.
    pub fn reset_collision_rays(&mut self) {
        let rad_cm = self.radius_cm;

        // TODO_PARAM: make this not hardcoded
        let num_mesh_rays = 18;
        self.collision_rays.resize_with(
            1 + num_mesh_rays + self.max_prop_rays as usize,
            Default::default,
        );

        // reset all collision rays.
        for ray in self.collision_rays.iter_mut() {
            ray.reset(rad_cm);
        }

        self.physics_flags.grounded_ray_type = None;
        self.avg_mesh_ray_len = rad_cm;
        self.vault_ray_idx = None;

        self.update_collision_rays();
        self.last_collision_rays = self.collision_rays.clone();
    }

    /// Update the katamari's collision rays
    pub fn update_collision_rays(&mut self) {
        self.last_collision_rays = self.collision_rays.clone();
        // self.debug_log_clip_data("0x1af26");
        self.orient_mesh_rays();
        // self.debug_log_clip_data("0x1af2e");

        self.update_rays_with_attached_props();

        if !self.physics_flags.wheel_spin {
            // if not spinning:
            let base_max_spd = self.scaled_params.base_max_speed;
            let speed = self.speed;

            let max_boost_spd =
                base_max_spd * self.scaled_params.max_boost_speed * self.params.boost_speed_mult;
            let max_fwd_spd = base_max_spd
                * self.scaled_params.max_forwards_speed
                * self.params.forwards_speed_mult;

            // map speed to [0.0, 1.0], where
            //   - [0, max_fwd_spd] -> 1.0
            //   - [max_fwd_spd, max_boost_spd] -> [1.0, 0.0] via an inv lerp
            //   - [max_boost_spd, +inf] -> 0.0.
            // this value is used to smoothly rescale the katamari's collision ray lengths
            // from the katamari radius (when boosting) to the rays' true lengths (when moving slowly)
            let ray_rescale_t = 1.0 - inv_lerp_clamp!(speed, max_fwd_spd, max_boost_spd);

            self.max_ray_len = self.radius_cm * self.params.max_ray_len_radii;

            // keep a running sum of the total ray length as we iterate over all rays, which
            // will be used to find the average length of mesh rays.
            let mut total_mesh_ray_len = 0.0;

            for (ray_idx, ray) in self.collision_rays.iter_mut().enumerate() {
                if ray_idx == 0 {
                    continue;
                }
                let adjusted_ray_len = lerp!(ray_rescale_t, self.radius_cm, ray.ray_len);
                if adjusted_ray_len > self.max_ray_len {
                    ray.ray_len = self.max_ray_len;
                }

                vec3::scale_and_add(
                    &mut ray.endpoint,
                    &self.center,
                    &ray.ray_local_unit,
                    adjusted_ray_len,
                );
                vec3_inplace_zero_small(&mut ray.endpoint, 0.0001);
                vec3::subtract(&mut ray.kat_to_endpoint, &ray.endpoint, &self.center);

                if ray_idx < self.num_mesh_rays as usize {
                    total_mesh_ray_len += adjusted_ray_len;
                }
            }

            let ground_type = self.physics_flags.grounded_ray_type;
            if ground_type != Some(KatCollisionRayType::Bottom) && ground_type.is_some() {
                // if the ground contact isn't the bottom ray:
                if let Some(ray_idx) = self.vault_ray_idx {
                    // update the vault ray's endpoint to be the place that ray contacts the ground,
                    // rather than its actual endpoint.
                    // (i think this is because the ray will generally be slightly clipped through
                    // the floor, and we want the player to pivot exactly on the floor).
                    let vault_ray = &mut self.collision_rays[ray_idx as usize];

                    vault_ray.endpoint = self.vault_contact_point;
                    vec3::subtract(
                        &mut vault_ray.kat_to_endpoint,
                        &self.vault_contact_point,
                        &self.center,
                    );
                    vec3::normalize(&mut vault_ray.ray_local_unit, &vault_ray.kat_to_endpoint);
                }
            }

            self.avg_mesh_ray_len = total_mesh_ray_len / self.num_mesh_rays as f32;
            self.larger_avg_mesh_ray_len =
                self.avg_mesh_ray_len * self.params.increased_collision_radius_mult;
        } else {
            // if spinning, set the length of each ray to the katamari's radius
            for ray in self.collision_rays.iter_mut() {
                ray.ray_len = self.radius_cm;
                ray.prop = None;
            }
            self.set_bottom_ray_contact();
        }
    }

    fn update_rays_with_attached_props(&mut self) {
        // TODO_PARAM
        let MAX_PROP_EFFECTS = 0xc;
        let MAX_VAULT_POINTS = 0xc;
        let NERFED_DANGLING_BOY_EFFECT = 0.8;

        // TODO_ENDING: `kat_update_rays_with_attached_props:137-143` (actually compute this based on game state)
        let prop_rays_enabled = true;

        // TODO: isn't this literally just `self.transform`
        let mut kat_transform = self.rotation_mat.clone();
        set_translation!(kat_transform, self.center);

        if prop_rays_enabled {
            let mut remaining_prop_effects = MAX_PROP_EFFECTS;

            // first, grow mesh collision rays that have attached objects nearby to them.

            // note that attached props are iterated over in reverse order, i.e. most recently
            // collected props are processed first
            for prop_ref in self.attached_props.iter().rev() {
                let mut prop = prop_ref.borrow_mut();

                // early return for disabled and unattached props, which shouldn't affect
                // the katamari's collision rays
                if prop.is_disabled() || !prop.is_attached() {
                    continue;
                }

                prop.update_transform_when_attached(&kat_transform);

                // early return if the maximum number of prop effects on the
                // katamari's mesh have already been made
                if remaining_prop_effects == 0 {
                    continue;
                }

                remaining_prop_effects -= 1;

                // compute the unit vector from the katamari to `prop`
                let kat_to_prop = vec3_from!(-, prop.get_position(), self.center);
                let mut kat_to_prop_unit = kat_to_prop.clone();
                vec3_inplace_normalize(&mut kat_to_prop_unit);

                let mut max_similarity = -1.0;
                let mut nearest_ray_idx = None;

                let mesh_rays_slice = &self.collision_rays[1..=self.num_mesh_rays as usize];
                for (idx, mesh_ray) in mesh_rays_slice.iter().enumerate() {
                    let similarity = vec3::dot(&mesh_ray.ray_local_unit, &kat_to_prop_unit);
                    if similarity > max_similarity {
                        max_similarity = similarity;
                        // add 1 to `idx` since we skipped the collision ray of index 0, which is the bottom ray
                        nearest_ray_idx = Some(idx as usize + 1);
                    }
                }

                if max_similarity <= 0.0 {
                    return;
                }

                let kat_to_prop_len = vec3::len(&kat_to_prop);
                let mut increased_ray_len = kat_to_prop_len * max_similarity;

                // name index 0x58b is "dangling boy"; this nerfs its effect on growing the katamari
                // when it's collected
                if prop.get_name_idx() == 0x58b {
                    increased_ray_len *= NERFED_DANGLING_BOY_EFFECT
                }

                if let Some(closest_ray) = self.collision_rays.get_mut(nearest_ray_idx.unwrap()) {
                    if closest_ray.ray_len < increased_ray_len {
                        closest_ray.ray_len = increased_ray_len;
                    }
                }
            }

            // compute vault points from attached props
            let mut remaining_vault_points = MAX_VAULT_POINTS;
            let mut prop_vault_points = vec![];

            for prop_ref in self.attached_props.iter().rev() {
                let prop = prop_ref.borrow_mut();
                let name_idx = prop.get_name_idx();
                let vault_points = prop.get_mono_data().map(|md| &md.vault_points).unwrap();

                // don't make a vault point if we're out of vault points
                if remaining_vault_points == 0 {
                    break;
                }

                // don't make a vault point if the prop is disabled, unattached, or has no vault points
                if prop.is_disabled() || !prop.is_attached() || vault_points.is_none() {
                    continue;
                }

                // at this point, we are committing to making a vault point for `prop`
                remaining_vault_points -= 1;

                // ignore vault points on "golf nuf", "golfer guy", and "dangling boy"
                if name_idx == 0x210 || name_idx == 0x591 || name_idx == 0x58b {
                    continue;
                }

                let num_vault_points = NamePropConfig::get(name_idx).num_vault_pts;

                // if `vault_points` is some, then `num_vault_points` should be >0, so this
                // check might be redundant
                if num_vault_points == 0 {
                    continue;
                }

                let prop_transform = prop.get_attached_transform();
                let mut world_vault_point = vec3::create();
                let mut kat_to_vault_point = vec3::create();
                let mut max_vault_point_dist = 0.0;
                let mut prop_vault_point = PropVaultPoint::default();
                prop_vault_point.prop = Rc::<RefCell<Prop>>::downgrade(prop_ref);

                // iterate over all vault points to find the one furthest from the katamari center
                for vault_point in vault_points.as_ref().unwrap() {
                    // compute the vault point in world space
                    vec3::transform_mat4(&mut world_vault_point, &vault_point, &prop_transform);
                    // compute the vector from the katamari center to the vault point
                    vec3::subtract(&mut kat_to_vault_point, &world_vault_point, &self.center);

                    let vault_point_dist = vec3::length(&kat_to_vault_point);
                    if vault_point_dist > max_vault_point_dist {
                        max_vault_point_dist = vault_point_dist;
                        vec3::normalize(&mut prop_vault_point.ray_unit, &kat_to_vault_point);
                        prop_vault_point.length = vault_point_dist;
                    }
                }

                prop_vault_points.push(prop_vault_point);
            }

            if prop_vault_points.len() == 0 {
                return;
            }

            for (idx, prop_vault_point) in prop_vault_points.iter().enumerate() {
                let mut ray = &mut self.collision_rays[self.first_prop_ray_index as usize + idx];
                vec3::scale(
                    &mut ray.kat_to_endpoint,
                    &prop_vault_point.ray_unit,
                    prop_vault_point.length,
                );
                vec3::copy(&mut ray.ray_local, &ray.kat_to_endpoint);
                ray.prop_ray_local_unit = prop_vault_point.ray_unit;
                ray.ray_local_unit = prop_vault_point.ray_unit;
                ray.prop = Some(prop_vault_point.prop.clone());
                ray.ray_len = prop_vault_point.length;
            }
        } else {
            for prop_ref in self.attached_props.iter_mut() {
                let mut prop = prop_ref.borrow_mut();
                if prop.is_disabled() {
                    continue;
                }
                prop.update_transform_when_attached(&kat_transform);
            }
        }
    }

    pub fn set_bottom_ray_contact(&mut self) {
        self.physics_flags.grounded_ray_type = Some(KatCollisionRayType::Bottom);
        self.vault_ray_idx = None;
        self.fc_ray_idx = None;
        self.fc_ray = None;
        self.fc_contact_point = None;
        mat4::identity(&mut self.vault_transform);
        self.vault_ticks = 0;
    }

    /// Orient the bottom and mesh collision rays.
    pub fn orient_mesh_rays(&mut self) {
        let mesh_points = &KAT_MESHES[self.mesh_index as usize];
        let mut tmp = VEC3_ZERO;
        let radius = self.radius_cm;

        // orient the bottom collision ray
        let bottom_ray = &mut self.collision_rays[0];
        if !self.physics_flags.climbing_wall {
            // if the katamari isn't climbing a wall:
            if !self.physics_flags.airborne {
                // if the katamari is grounded, the bottom ray is in the direction of the
                // contact floor surface's normal
                vec3::copy(&mut tmp, &self.contact_floor_normal_unit);
                vec3_inplace_scale(&mut tmp, -radius);
                vec3::add(&mut bottom_ray.endpoint, &self.center, &tmp);
            } else {
                // otherwise if the katamari is airborne, the bottom ray is straight down
                let down: Vec3 = [0.0, -radius, 0.0];
                vec3::add(&mut bottom_ray.endpoint, &self.center, &down);
            }
        } else {
            // else if the katamari is climbing a wall:
            // TODO_CLIMB: `kat_orient_mesh_rays:125-147`
        }

        bottom_ray.ray_len = radius;
        vec3_inplace_zero_small(&mut bottom_ray.endpoint, 0.0001);
        vec3::subtract(
            &mut bottom_ray.ray_local,
            &bottom_ray.endpoint,
            &self.center,
        );
        vec3::normalize(&mut bottom_ray.ray_local_unit, &bottom_ray.ray_local);
        vec3::subtract(
            &mut bottom_ray.kat_to_endpoint,
            &bottom_ray.endpoint,
            &self.center,
        );

        // orient the mesh collision rays
        self.num_mesh_rays = 0;

        // orient each mesh point using the katamari's rotation matrix
        for (mesh_ray_idx, mesh_point) in mesh_points.points.iter().enumerate() {
            let mesh_ray = &mut self.collision_rays[mesh_ray_idx + 1];
            vec3::transform_mat4(&mut mesh_ray.ray_local, mesh_point, &self.rotation_mat);
            vec3::normalize(&mut mesh_ray.ray_local_unit, &mesh_ray.ray_local);
            mesh_ray.ray_len = radius;
            self.num_mesh_rays += 1;
        }

        self.first_prop_ray_index = self.num_mesh_rays + 1;

        // reset each prop ray
        for prop_ray in self.collision_rays[self.first_prop_ray_index as usize..].iter_mut() {
            prop_ray.reset(radius);
        }
    }

    /// Returns the type of the collision ray at index `ray_idx` if it exists.
    /// If there is no ray at that index, returns `None`.
    pub fn ray_type_by_idx(&self, ray_idx: u16) -> Option<KatCollisionRayType> {
        if ray_idx == 0 {
            Some(KatCollisionRayType::Bottom)
        } else if ray_idx < self.first_prop_ray_index {
            Some(KatCollisionRayType::Mesh)
        } else {
            self.collision_rays
                .get(ray_idx as usize)
                .map(|_| KatCollisionRayType::Prop)
        }
    }
}
