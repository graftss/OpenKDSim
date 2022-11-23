use gl_matrix::{common::Vec4, vec4};

use crate::{constants::VEC4_ZERO, prop::WeakPropRef};

use super::Katamari;

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
pub enum KatRay {
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

    /// The vector from the kat center to the ray `endpoint`
    /// offset: 0x20
    pub kat_to_endpoint: Vec4,

    /// Unit rescaled 0x10 vector
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

impl Katamari {
    pub fn init_collision_rays(&mut self) {}

    pub fn reset_collision_rays(&mut self) {}
}
