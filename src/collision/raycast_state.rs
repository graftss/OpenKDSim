use std::{cell::RefCell, rc::Rc};

use gl_matrix::{
    common::{Mat4, Vec3},
    mat4, vec3,
};

use crate::{
    constants::{UNITY_TO_SIM_SCALE, VEC3_Y_POS},
    delegates::Delegates,
    macros::{panic_log, vec3_from},
    math::{vec3_inplace_normalize, vec3_inplace_zero_small},
};

use super::hit_attribute::HitAttribute;

const IMPACT_EPS: f32 = 0.0001;

/// A triangle hit by a raycast.
#[derive(Debug, Default)]
pub struct RaycastTriHit {
    /// The point on the triangle that was hit.
    /// offset: 0x0
    pub impact_point: Vec3,

    /// The unit normal of the hit triangle.
    /// offset: 0x10
    pub normal_unit: Vec3,

    /// The three points of the triangle.
    /// offset: 0x20
    pub tri: [Vec3; 3],

    /// Metadata of the triangle, which can be either a hit attribute (for
    /// katamari-surface collisions) or a zone index (for prop-surface collisions).
    /// offset: 0x50
    pub metadata: i32,

    /// The ratio of the hit distance to the ray length.
    /// offset: 0x54
    pub impact_dist_ratio: f32,

    /// The distance from the ray initial point to the hit point.
    /// offset: 0x58
    pub impact_dist: f32,
}

/// TODO: Encodes data about a single raycast.
/// offset: 0x1941e0 (it's allocated in the heap, but this is a pointer to it)
#[derive(Debug, Default)]
pub struct RaycastState {
    delegates: Option<Rc<RefCell<Delegates>>>,

    /// Initial point of the collision ray.
    /// offset: 0x0
    pub point0: Vec3,

    /// Final point of the collision ray.
    /// offset: 0x10
    pub point1: Vec3,

    /// The collision ray vector (i.e. the vector from `point0` to `point1`).
    /// offset: 0x20
    pub ray: Vec3,

    /// The collision ray unit vector.
    /// offset: 0x30
    pub ray_unit: Vec3,

    /// The "look at" matrix from p0 to p1.
    /// offset: 0x40
    pub ray_lookat: Mat4,

    /// (unused) The inverse "look at" matrix from p0 to p1 (which I think means it's
    /// the look at matrix from p1 to p0).
    /// offset: 0x80
    // pub ray_lookat_inv: Mat4,

    /// (unused) The transpose of the "look at" matrix from p0 to p1.
    /// offset: 0xc0
    // pub ray_lookat_t: Mat4,

    /// (unused) The `ray_lookat` matrix times `point1`.
    /// offset: 0x110
    // pub ray_lookat_times_p1: Vec3,

    /// The distance from p0 to the computed impact point.
    /// offset: 0x120
    pub hit_dist: f32,

    /// (unused) The length squared of the collision ray from p0 to p1.
    /// offset: 0x124
    // pub ray_len_sq: f32,

    /// The number of triangles hit by the raycast.
    /// offset: 0x234
    pub num_hit_tris: u8,

    /// The triangles hit by the raycast.
    /// offset: 0x238
    pub hit_tris: Vec<RaycastTriHit>,

    /// The index in `hit_tris` of the closest triangle that was hit, if any.
    /// offset: 0x858
    pub closest_hit_idx: Option<u8>,
}

pub enum RaycastCallType {
    Objects,
    Stage,
    Water,
}

impl Into<i32> for RaycastCallType {
    fn into(self) -> i32 {
        match self {
            RaycastCallType::Objects => 0,
            RaycastCallType::Stage => 1,
            RaycastCallType::Water => 2,
        }
    }
}

impl RaycastState {
    pub fn set_delegates(&mut self, delegates: &Rc<RefCell<Delegates>>) {
        self.delegates = Some(delegates.clone());
    }

    /// Load a ray into the raycast state for further collision checks.
    pub fn load_ray(&mut self, point0: &Vec3, point1: &Vec3) {
        self.point0 = point0.clone();
        self.point1 = point1.clone();
        self.ray = vec3_from!(-, point0, point1);
        vec3::normalize(&mut self.ray_unit, &self.ray);
        mat4::look_at(&mut self.ray_lookat, &point0, &self.ray, &VEC3_Y_POS);

        // cached data that the simulation computes but doesn't appear to ever use
        // mat4::invert(&mut self.ray_lookat_inv, &self.ray_lookat);
        // mat4::transpose(&mut self.ray_lookat_t, &self.ray_lookat);
        // vec3::transform_mat4(&self.ray_lookat_times_p1, &point1, &self.ray_lookat);

        // this seems fishy since the `hit_dist` field is later used for the distance
        // from `point0` to the `impact_point`, but whatever...
        self.hit_dist = vec3::length(&self.ray);
    }

    /// Read the closest hit from the `hit_tris` list.
    pub fn get_closest_hit(&self) -> Option<&RaycastTriHit> {
        self.closest_hit_idx
            .map(|idx| self.hit_tris.get(idx as usize))
            .flatten()
    }

    /// Use the unity delegates to find the nearest map hit along the cached ray.
    pub fn find_nearest_unity_hit(
        &mut self,
        call_type: RaycastCallType,
        include_objects: bool,
    ) -> bool {
        let scale = UNITY_TO_SIM_SCALE;

        self.hit_tris.clear();

        let delegates = self
            .delegates
            .as_ref()
            .unwrap_or_else(|| {
                panic_log!("no delegates defined for `RaycastState`.");
            })
            .borrow();
        let do_hit = delegates.do_hit.unwrap_or_else(|| {
            panic_log!("no `do_hit` delegate defined.");
        });
        let get_hit_count = delegates.get_hit_count.unwrap_or_else(|| {
            panic_log!("no `get_hit_count` delegate defined.");
        });
        let get_hit_attribute = delegates.get_hit_attribute.unwrap_or_else(|| {
            panic_log!("no `get_hit_attribute` delegate defined.");
        });
        let get_impact_point = delegates.get_impact_point.unwrap_or_else(|| {
            panic_log!("no `get_impact_point` delegate defined.");
        });
        let get_impact_normal = delegates.get_impact_normal.unwrap_or_else(|| {
            panic_log!("no `get_impact_normal` delegate defined.");
        });

        let do_hit_result = do_hit(
            self.point0[0] / UNITY_TO_SIM_SCALE,
            self.point0[1] / UNITY_TO_SIM_SCALE,
            self.point0[2] / UNITY_TO_SIM_SCALE,
            self.point1[0] / UNITY_TO_SIM_SCALE,
            self.point1[1] / UNITY_TO_SIM_SCALE,
            self.point1[2] / UNITY_TO_SIM_SCALE,
            call_type.into(),
            888,
            include_objects.into(),
        );

        if include_objects || do_hit_result >= 0 {
            let mut hit_attr_i32 = 0;
            let mut pt_x = 0.0;
            let mut pt_y = 0.0;
            let mut pt_z = 0.0;
            let mut norm_x = 0.0;
            let mut norm_y = 0.0;
            let mut norm_z = 0.0;

            for hit_idx in 0..get_hit_count() {
                get_hit_attribute(hit_idx, &mut hit_attr_i32);
                let hit_attr: HitAttribute = hit_attr_i32.into();

                // find the first hit without a `KingWarp` or `Jump` hit attribute.
                // since unity hits are sorted by their increasing length, this will be the
                // closest such hit.
                if hit_attr == HitAttribute::KingWarp || hit_attr == HitAttribute::Jump {
                    continue;
                }

                // read and process impact point from unity
                get_impact_point(hit_idx, &mut pt_x, &mut pt_y, &mut pt_z);
                let mut impact_point = [pt_x * scale, pt_y * scale, pt_z * scale];
                vec3_inplace_zero_small(&mut impact_point, IMPACT_EPS);

                // read and process unit normal vector from unity
                get_impact_normal(hit_idx, &mut norm_x, &mut norm_y, &mut norm_z);
                let mut impact_normal = [norm_x * scale, norm_y * scale, norm_z * scale];
                vec3_inplace_zero_small(&mut impact_normal, IMPACT_EPS);
                vec3_inplace_normalize(&mut impact_normal);

                // compute impact distance
                let ray_len = vec3::distance(&self.point0, &self.point1);
                let hit_to_p1_len = vec3::distance(&self.point1, &impact_point);
                let impact_dist = ray_len - hit_to_p1_len;

                // write hit to the raycast state at index 0 (it will be the only stored hit)
                self.closest_hit_idx = Some(0);
                self.hit_dist = impact_dist;
                self.hit_tris.push(RaycastTriHit {
                    impact_point: impact_point.clone(),
                    normal_unit: impact_normal.clone(),
                    tri: Default::default(),
                    metadata: hit_attr_i32,
                    impact_dist_ratio: impact_dist / ray_len,
                    impact_dist,
                });

                // i guess this is because we want to just ignore the water surface for
                // collision purposes
                return hit_attr != HitAttribute::WaterSurface;
            }
        }

        false
    }
}
