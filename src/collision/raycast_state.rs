use std::{cell::RefCell, rc::Rc};

use gl_matrix::{
    common::{Mat4, Vec3},
    mat2::create,
    mat4, vec3,
};

use crate::{
    constants::{UNITY_TO_SIM_SCALE, VEC3_Y_POS},
    delegates::Delegates,
    macros::{panic_log, temp_debug_log, vec3_from},
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
    // BEGIN fields not in the original simulation
    delegates: Option<Rc<RefCell<Delegates>>>,

    /// For some reason this was a global vector instead of part of the raycast state...
    /// offset: 0xb3230
    ray_to_triangle_hit_point: Vec3,
    // END fields not in the original simulation
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
    pub ray_len: f32,

    /// (unused) The length squared of the collision ray from p0 to p1.
    /// offset: 0x124
    // pub ray_len_sq: f32,

    /// The number of triangles hit by the raycast.
    /// offset: 0x234
    pub num_hit_tris: u8,

    /// The triangles hit by the raycast.
    /// offset: 0x238
    pub hit_tris: Vec<RaycastTriHit>,

    /// Data computed as a side effect of calling  `ray_hits_triangle`.
    /// offset: 0x7f8
    pub ray_to_triangle_hit: RaycastTriHit,

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
        self.ray = vec3_from!(-, point1, point0);
        vec3::normalize(&mut self.ray_unit, &self.ray);
        mat4::look_at(&mut self.ray_lookat, &point0, &self.ray, &VEC3_Y_POS);

        // cached data that the simulation computes but doesn't appear to ever use
        // mat4::invert(&mut self.ray_lookat_inv, &self.ray_lookat);
        // mat4::transpose(&mut self.ray_lookat_t, &self.ray_lookat);
        // vec3::transform_mat4(&self.ray_lookat_times_p1, &point1, &self.ray_lookat);

        // this seems fishy since the `hit_dist` field is later used for the distance
        // from `point0` to the `impact_point`, but whatever...
        self.ray_len = vec3::length(&self.ray);
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
            self.point0[0] / scale,
            self.point0[1] / scale,
            self.point0[2] / scale,
            self.point1[0] / scale,
            self.point1[1] / scale,
            self.point1[2] / scale,
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

                // read the normal vector from unity and compute its unit
                get_impact_normal(hit_idx, &mut norm_x, &mut norm_y, &mut norm_z);
                let mut impact_normal_unit = [norm_x, norm_y, norm_z];
                vec3_inplace_zero_small(&mut impact_normal_unit, IMPACT_EPS);
                vec3_inplace_normalize(&mut impact_normal_unit);

                // compute impact distance
                let ray_len = vec3::distance(&self.point0, &self.point1);
                let hit_to_p1_len = vec3::distance(&self.point1, &impact_point);
                let impact_dist = ray_len - hit_to_p1_len;

                // write hit to the raycast state at index 0 (it will be the only stored hit)
                self.closest_hit_idx = Some(0);
                self.ray_len = ray_len;
                self.hit_tris.push(RaycastTriHit {
                    impact_point: impact_point.clone(),
                    normal_unit: impact_normal_unit.clone(),
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

    /// offset: 0x11d70
    pub fn ray_hits_triangle(&mut self, triangle: &[Vec3; 3], transform: &Option<Mat4>) -> f32 {
        let EPS = 0.000001;

        let [mut p0, mut p1, mut p2] = triangle.clone();

        if let Some(mat) = transform {
            vec3::transform_mat4(&mut p0, &triangle[0], mat);
            vec3::transform_mat4(&mut p1, &triangle[1], mat);
            vec3::transform_mat4(&mut p2, &triangle[2], mat);
        }

        println!("trangle={:?}", triangle);

        // just naively copy this i guess
        let p0p1 = vec3_from!(-, p1, p0);
        let p0p2 = vec3_from!(-, p2, p0);
        let ray = vec3_from!(-, self.point1, self.point0);
        let ray_len = vec3::length(&ray);

        let d0 = p0p2[2] * ray[1] - p0p2[1] * ray[2];
        let d1 = p0p2[0] * ray[2] - p0p2[2] * ray[0];
        let d2 = p0p2[1] * ray[0] - p0p2[0] * ray[1];
        let d = d1 * p0p1[1] + d0 * p0p1[0] + d2 * p0p1[2];

        println!("d={}", d);

        // if d is (almost) 0, the ray is parallel (enough) to the plane of the triangle (to admit defeat)
        if d > -EPS && d < EPS {
            return 0.0;
        }

        let d_inv = 1.0 / d;

        let p0r0 = vec3_from!(-, self.point0, p0);
        let dt = (p0r0[1] * d1 + p0r0[0] * d0 + p0r0[2] * d2) * d_inv;

        println!("dt={}", dt);

        if dt < 0.0 || dt > 1.0 {
            return 0.0;
        }

        let x0 = p0r0[1] * p0p1[2] - p0r0[2] * p0p1[1];
        let x1 = p0r0[2] * p0p1[0] - p0r0[0] * p0p1[2];
        let x2 = p0r0[0] * p0p1[1] - p0r0[1] * p0p1[0];
        let du = (x0 * ray[0] + x1 * ray[1] + x2 * ray[2]) * d_inv;

        println!("du={}", du);

        if du < 0.0 || (du + dt) > 1.0 {
            return 0.0;
        }

        let dv = (x0 * p0p2[0] + x1 * p0p2[1] + x2 * p0p2[2]) * d_inv;

        println!("dv={}, d0={}", dv, d0);

        if dv <= EPS || dv > ray_len
        /* something else here maybe */
        {
            return 0.0;
        }

        // compute the point at which the stored ray hits the input triangle
        let t = dv * ray_len;
        vec3::scale_and_add(
            &mut self.ray_to_triangle_hit_point,
            &self.point0,
            &self.ray_unit,
            t,
        );

        println!("t={}", t);

        if t < 0.0 || t > self.ray_len {
            return 0.0;
        }

        self.ray_to_triangle_hit.impact_point = self.ray_to_triangle_hit_point;
        // TODO: ?? what is this doing
        if transform.is_none() {
            self.ray_to_triangle_hit.impact_point[2] = t;
        }

        let l0 = p0p2[2] * p0p1[1] - p0p2[1] * p0p1[2];
        let l1 = p0p2[0] * p0p1[2] - p0p2[2] * p0p1[0];
        let l2 = p0p2[1] * p0p1[0] - p0p2[0] * p0p1[1];
        let mut normal_unit = [l0, l1, l2];
        vec3_inplace_normalize(&mut normal_unit);
        vec3::scale(
            &mut self.ray_to_triangle_hit.normal_unit,
            &normal_unit,
            -1.0,
        );

        return t;
    }
}

#[cfg(test)]
mod tests {
    use super::RaycastState;

    #[test]
    fn test_triangle_collision() {
        let triangle = [
            [-2.6363091468811, 0.020901577547193, -4.1993327140808],
            [-2.6363091468811, -6.4182171821594, 4.274188041687],
            [-2.6363091468811, -6.4182171821594, -4.199333190918],
        ];
        let ray = [
            [-26.963624954224, -24.175483703613, 17.492353439331],
            [-25.76362991333, -25.552066802979, 15.770400047302],
        ];
        let result = [
            [-26.267070770264, -24.974540710449, 1.4569648504257],
            [0.70710015296936, 2.0945599032984e-007, 0.70711332559586],
        ];
        let t = 1.46;

        let mut raycast_state = RaycastState::default();
        raycast_state.load_ray(&ray[0], &ray[1]);
        raycast_state.ray_hits_triangle(&triangle, &None);
        println!("result: {:?}", raycast_state.ray_to_triangle_hit);
    }
}
