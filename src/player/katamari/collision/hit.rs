use std::fmt::Debug;

use gl_matrix::common::Vec3;
use serde::{Deserialize, Serialize};

use crate::{collision::hit_attribute::HitAttribute, props::prop::PropRef};

/// Describes a collision between a katamari collision ray and another
/// surface (which could be on either a prop or the map)
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct SurfaceHit {
    /// (??)
    /// offset: 0x0
    pub clip_normal_len: f32,

    /// The length of the katamari collision ray making the contact.
    /// offset: 0x4
    pub ray_len: f32,

    /// The index of the katamari collision ray making the contact.
    /// offset: 0x8
    pub ray_idx: i16,

    /// The position along the ray where the hit occured, rescaled to be
    /// in the interval [0, 1], where 0 means the ray's initial point
    /// and 1 means its endpoint.
    /// offset: 0xc
    pub impact_dist_ratio: f32,

    /// The vector from the ray initial point to its endpoint.
    /// offset: 0x10
    pub ray: Vec3,

    /// The contacted surface's unit normal vector.
    /// offset: 0x20
    pub normal_unit: Vec3,

    /// (??)
    /// offset: 0x30
    pub clip_normal: Vec3,

    /// The point on the surface that was contacted.
    /// offset: 0x40
    pub contact_point: Vec3,

    /// The type of surface that was contacted.
    /// offset: 0x50
    pub hit_attr: HitAttribute,

    /// If the contact surface belongs to a prop collision mesh, this
    /// is a pointer to that prop.
    /// offset: 0x58
    // TODO_SERIAL: replace `PropRef` with u16 control index
    #[serde(skip)]
    pub prop: Option<PropRef>,
}

impl Debug for SurfaceHit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prop = self
            .prop
            .as_ref()
            .map(|prop_ref| prop_ref.borrow().get_ctrl_idx());

        f.debug_struct("SurfaceHit")
            .field("clip_normal_len", &self.clip_normal_len)
            .field("ray_len", &self.ray_len)
            .field("ray_idx", &self.ray_idx)
            .field("impact_dist_ratio", &self.impact_dist_ratio)
            .field("ray", &self.ray)
            .field("normal_unit", &self.normal_unit)
            .field("clip_normal", &self.clip_normal)
            .field("contact_point", &self.contact_point)
            .field("hit_attr", &self.hit_attr)
            .field("prop", &prop)
            .finish()
    }
}
