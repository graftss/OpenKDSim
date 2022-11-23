use gl_matrix::{common::Vec4, vec4};

use crate::{
    constants::VEC4_ZERO,
    math::{vec4_scale_inplace, vec4_zero_small_inplace},
    prop::WeakPropRef,
};

use super::{mesh::KAT_MESHES, Katamari};

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
    pub endpoint: Vec4,

    /// (??)
    /// offset: 0x10
    pub ray_local: Vec4,

    /// The vector from the katamari center to the ray `endpoint`
    /// offset: 0x20
    pub kat_to_endpoint: Vec4,

    /// The unit vector from the katamari center to the ray endpoint.
    /// offset: 0x30
    pub ray_local_unit: Vec4,

    /// Zero if mesh ray, 0x30 vector if prop ray
    /// offset: 0x40
    pub prop_ray_local_unit: Vec4,

    /// If this ray is induced by a prop, points to that prop
    /// offset: 0x50
    pub prop: Option<WeakPropRef>,

    /// Length of the ray.
    /// offset: 0x58
    pub length: f32,

    /// True if this ray contacts a surface
    /// offset: 0x5d
    pub contacts_surface: bool,
}

impl KatCollisionRay {
    /// Reset the collision ray.
    pub fn reset(&mut self, rad_cm: f32) {
        vec4::copy(&mut self.endpoint, &VEC4_ZERO);
        vec4::copy(&mut self.ray_local, &VEC4_ZERO);
        vec4::copy(&mut self.kat_to_endpoint, &VEC4_ZERO);
        vec4::copy(&mut self.ray_local_unit, &VEC4_ZERO);
        vec4::copy(&mut self.prop_ray_local_unit, &VEC4_ZERO);
        self.length = rad_cm;
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

        self.physics_flags.pivot_ray = Some(KatCollisionRayType::Bottom);
        self.average_ray_len = rad_cm;
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
            // TODO `kat_update_collision_rays:60-214`
        } else {
            // if spinning, set the length of each ray to the katamari's radius
            for ray in self.collision_rays.iter_mut() {
                ray.length = self.radius_cm;
                ray.prop = None;
            }
            // TODO: `kat_set_bottom_ray_contact()`.
        }
    }

    /// Orient the bottom and mesh collision rays.
    pub fn orient_mesh_rays(&mut self) {
        let mesh_points = &KAT_MESHES[self.mesh_index as usize];
        let mut tmp = VEC4_ZERO;
        let radius = self.radius_cm;

        // orient the bottom collision ray
        let bottom_ray = &mut self.collision_rays[0];
        if !self.physics_flags.climbing_wall {
            // if the katamari isn't climbing a wall:
            if !self.physics_flags.airborne {
                // if the katamari is grounded, the bottom ray is in the direction of the
                // contact floor surface's normal
                vec4::copy(&mut tmp, &self.contact_floor_normal_unit);
                vec4_scale_inplace(&mut tmp, -1.0 * radius);
                vec4::subtract(&mut bottom_ray.endpoint, &self.center, &tmp);
            } else {
                // otherwise if the katamari is airborne, the bottom ray is straight down
                let up: Vec4 = [0.0, radius, 0.0, 0.0];
                vec4::subtract(&mut bottom_ray.endpoint, &self.center, &up);
            }
        } else {
            // else if the katamari is climbing a wall:
            // TODO: `kat_orient_mesh_rays:125-147`
        }
        bottom_ray.length = radius;
        vec4_zero_small_inplace(&mut bottom_ray.endpoint, 0.0001);
        vec4::subtract(
            &mut bottom_ray.kat_to_endpoint,
            &bottom_ray.endpoint,
            &self.center,
        );
        vec4::normalize(&mut bottom_ray.ray_local_unit, &bottom_ray.kat_to_endpoint);

        // TODO: `kat_orient_mesh_rays:164-174`

        // orient the mesh collision rays
        self.num_mesh_rays = 0;
        let rotation_mat = &self.rotation_mat;

        // orient each mesh point using the katamari's rotation matrix
        for (i, mesh_point) in mesh_points.points.iter().enumerate() {
            let mesh_ray = &mut self.collision_rays[i];
            vec4::transform_mat4(&mut mesh_ray.ray_local, mesh_point, &rotation_mat);
            vec4::normalize(&mut mesh_ray.ray_local_unit, &mesh_ray.ray_local);
            mesh_ray.length = radius;
            self.num_mesh_rays += 1;
        }

        self.first_prop_ray_index = self.num_mesh_rays + 1;

        // reset each prop ray
        for prop_ray in self.collision_rays[self.first_prop_ray_index as usize..].iter_mut() {
            prop_ray.reset(radius);
        }
    }
}