use gl_matrix::{common::Vec3, mat4, vec3};

use crate::{
    constants::VEC3_ZERO,
    macros::{inv_lerp_clamp, lerp, temp_debug_log, vec3_from},
    math::{vec3_inplace_scale, vec3_inplace_zero_small},
    player::katamari::Katamari,
    props::prop::WeakPropRef,
};

use super::mesh::KAT_MESHES;

/// The extra "shell" collision rays which extend along the top half of the katamari.
/// (see https://discord.com/channels/232268612285497345/805240416894713866/842591732229996544)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellRay {
    TopCenter = 1,
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
        vec3::copy(&mut self.endpoint, &VEC3_ZERO);
        vec3::copy(&mut self.ray_local, &VEC3_ZERO);
        vec3::copy(&mut self.kat_to_endpoint, &VEC3_ZERO);
        vec3::copy(&mut self.ray_local_unit, &VEC3_ZERO);
        vec3::copy(&mut self.prop_ray_local_unit, &VEC3_ZERO);
        self.ray_len = rad_cm;
        self.prop = None;
        self.contacts_surface = false;
    }
}

pub type KatCollisionRays = Vec<KatCollisionRay>;

impl Katamari {
    /// Resets the katamari's collision rays to their initial state.
    /// This is only called at the start of a mission and after a royal warp.
    pub fn reset_collision_rays(&mut self) {
        let rad_cm = self.radius_cm;

        // TODO: make this not hardcoded
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
        self.orient_mesh_rays();

        // TODO: `kat_update_rays_with_attached_props()`

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
            let ray_rescale_t = inv_lerp_clamp!(speed, max_boost_spd, max_fwd_spd);

            self.max_ray_len = self.radius_cm * self.params.max_ray_len_radii;

            // keep a running sum of the total ray length as we iterate over all rays, which
            // will be used to find the average length of mesh rays.
            let mut total_mesh_ray_len = 0.0;

            for (ray_idx, ray) in self.collision_rays.iter_mut().enumerate() {
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
                vec3_inplace_scale(&mut tmp, radius);
                vec3::subtract(&mut bottom_ray.endpoint, &self.center, &tmp);
            } else {
                // otherwise if the katamari is airborne, the bottom ray is straight down
                let down: Vec3 = [0.0, -radius, 0.0];
                vec3::add(&mut bottom_ray.endpoint, &self.center, &down);
            }
        } else {
            // else if the katamari is climbing a wall:
            // TODO: `kat_orient_mesh_rays:125-147`
        }

        bottom_ray.ray_len = radius;
        vec3_inplace_zero_small(&mut bottom_ray.endpoint, 0.0001);
        vec3::subtract(
            &mut bottom_ray.kat_to_endpoint,
            &bottom_ray.endpoint,
            &self.center,
        );
        vec3::normalize(&mut bottom_ray.ray_local_unit, &bottom_ray.kat_to_endpoint);

        // TODO: `kat_orient_mesh_rays:164-174` (orient other shell rays)

        // orient the mesh collision rays
        self.num_mesh_rays = 0;

        // orient each mesh point using the katamari's rotation matrix
        for (i, mesh_point) in mesh_points.points.iter().enumerate() {
            let mesh_ray = &mut self.collision_rays[i + 1];
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
