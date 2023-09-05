use std::{cell::RefCell, rc::Rc};

use gl_matrix::{
    common::{Mat4, Vec3, Vec4},
    mat4, vec3,
};

use crate::{
    collision::mesh::TriGroup,
    constants::{UNITY_TO_SIM_SCALE, VEC3_Y_POS},
    debug::DEBUG_CONFIG,
    delegates::{has_delegates::HasDelegates, Delegates},
    macros::{panic_log, vec3_from},
    math::{vec3_inplace_normalize, vec3_inplace_zero_small},
};

use super::{
    hit_attribute::HitAttribute,
    mesh::{Mesh, TriVertex},
};

const IMPACT_EPS: f32 = 0.0001;

/// A triangle hit by a raycast.
#[derive(Debug, Default, Clone, Copy)]
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

/// Stores a single ray so that multiple raycasts can be performed on it.
/// offset: 0x1941e0 (it's allocated in the heap, but this offset holds a pointer to it)
#[derive(Debug, Default)]
pub struct RaycastState {
    // BEGIN fields not in the original simulation
    delegates: Option<Rc<RefCell<Delegates>>>,

    /// For some reason this was a global vector instead of part of the raycast state...
    /// offset: 0xb3230
    triangle_hit_point: Vec3,

    zone_mesh: Option<Rc<Mesh>>,
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
    pub tri_hits: Vec<RaycastTriHit>,

    /// Data computed as a side effect of calling  `ray_hits_triangle`.
    /// offset: 0x7f8
    pub ray_to_triangle_hit: RaycastTriHit,

    /// The index in `hit_tris` of the closest triangle that was hit, if any.
    /// offset: 0x858
    pub closest_hit_idx: Option<u8>,
}

impl HasDelegates for RaycastState {
    fn get_delegates_ref(&self) -> Option<&crate::delegates::DelegatesRef> {
        self.delegates.as_ref()
    }

    fn set_delegates_ref(&mut self, delegates_ref: &crate::delegates::DelegatesRef) {
        self.delegates = Some(delegates_ref.clone());
    }
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
    /// offset: 0x10350
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

    /// Get a reference to the closest hit from the `hit_tris` list.
    pub fn get_closest_hit(&self) -> Option<&RaycastTriHit> {
        self.closest_hit_idx
            .map(|idx| self.tri_hits.get(idx as usize))
            .flatten()
    }

    /// Get a mutable reference to the closest hit from the `hit_tris` list.
    pub fn get_closest_hit_mut(&mut self) -> Option<&mut RaycastTriHit> {
        self.closest_hit_idx
            .map(|idx| self.tri_hits.get_mut(idx as usize))
            .flatten()
    }

    /// Use the unity delegates to find the nearest map hit along the cached ray.
    pub fn find_nearest_unity_hit(
        &mut self,
        call_type: RaycastCallType,
        include_objects: bool,
    ) -> bool {
        let scale = UNITY_TO_SIM_SCALE;

        self.tri_hits.clear();

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
                self.tri_hits.push(RaycastTriHit {
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

    /// If this raycast's state cached ray hits the triangle `triangle`
    /// Adapted from https://en.m.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    /// offset: 0x11d70
    fn ray_hits_triangle(
        &mut self,
        triangle: &[Vec3; 3],
        transform: &Mat4,
        tri_in_world_space: bool,
    ) -> Option<f32> {
        let EPS = 0.000001;

        let [mut p0, mut p1, mut p2] = &triangle;

        if !tri_in_world_space {
            vec3::transform_mat4(&mut p0, &triangle[0], transform);
            vec3::transform_mat4(&mut p1, &triangle[1], transform);
            vec3::transform_mat4(&mut p2, &triangle[2], transform);
        }

        let edge1 = vec3_from!(-, p1, p0);
        let edge2 = vec3_from!(-, p2, p0);
        let ray = vec3_from!(-, self.point1, self.point0);

        let mut h = vec3::create();
        vec3::cross(&mut h, &ray, &edge2);
        let a = vec3::dot(&edge1, &h);

        // detect if the ray is parallel to the triangle
        if a > -EPS && a < EPS {
            return None;
        }

        let f = 1.0 / a;
        let s = vec3_from!(-, self.point0, p0);
        let u = f * vec3::dot(&s, &h);

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let mut q = vec3::create();
        vec3::cross(&mut q, &s, &edge1);
        let v = f * vec3::dot(&ray, &q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * vec3::dot(&edge2, &q);

        if t > EPS && t < 1.0 {
            vec3::scale_and_add(&mut self.triangle_hit_point, &self.point0, &ray, t);
            let ray_len = vec3::length(&ray);
            let p0_to_impact_len = t * ray_len;
            self.ray_to_triangle_hit.impact_point[2] = p0_to_impact_len;

            // compute unit normal of the triangle
            let mut normal_unit = vec3::create();
            vec3::cross(&mut normal_unit, &edge2, &edge1);
            vec3_inplace_normalize(&mut normal_unit);
            vec3::copy(&mut self.ray_to_triangle_hit.normal_unit, &normal_unit);

            // if the triangle's normal and the ray have a positive dot product, that means
            // they're going in the same direction. we don't want a collision in those cases,
            // since it leads to e.g. getting stuck on "walls" (triangles whose normal points down,
            // meaning they arent floors) on the bottom of thin props when we roll over them.
            let ray_dot_tri_normal = vec3::dot(&normal_unit, &ray);
            if ray_dot_tri_normal > 0.0 {
                return None;
            }

            static HIT_COLOR: Vec4 = [1.0, 0.0, 1.0, 1.0];
            if let Some(delegates) = &self.delegates {
                delegates
                    .borrow_mut()
                    .debug_draw
                    .draw_point(&self.triangle_hit_point, &HIT_COLOR);
            }

            Some(p0_to_impact_len)
        } else {
            None
        }
    }

    /// Returns the number of triangles in `mesh` hit by the ray.
    /// `transform` is the transform matrix of the `mesh`.
    /// offset: 0x10da0
    pub fn ray_hits_mesh(
        &mut self,
        mesh: &Mesh,
        transform: &Mat4,
        ray_in_mesh_coords: bool,
    ) -> i32 {
        let mut local_p0 = self.point0.clone();
        let mut local_p1 = self.point1.clone();

        // the original simulation doesn't seem to use the ray-to-aabb intersection point, but it
        // still computes it. so whatever
        let mut aabb_collision_out = vec3::create();

        // if the collision ray isn't already in the mesh's coordinate space, multiply
        // the endpoints of the collision ray by the inverse of the mesh's `transform`.
        if !ray_in_mesh_coords {
            let mut transform_inv = mat4::create();

            mat4::invert(&mut transform_inv, &transform);
            vec3::transform_mat4(&mut local_p0, &self.point0, &transform_inv);
            vec3::transform_mat4(&mut local_p1, &self.point1, &transform_inv);
        }

        // iterate over the mesh sectors, checking if the ray meets each sector's AABB
        let mut hit_aabbs = vec![];
        let mut hit_any_aabb = false;
        for sector in mesh.sectors.iter() {
            // the original simulation multiplies all the AABB coordinates by -1 here, but we
            // already did that once when the AABBs were parsed (in `PropMonoData::parse_aabbs`)
            // so that we don't have to do it over and over again here.
            let hit_aabb = ray_hits_aabb(
                &local_p0,
                &local_p1,
                &sector.aabb.min,
                &sector.aabb.max,
                &mut aabb_collision_out,
            );
            hit_aabbs.push(hit_aabb);
            hit_any_aabb |= hit_aabb;

            if !hit_aabb {
                continue;
            }

            // if there was an aabb hit, attempt to do some debug collision drawing
            // for the prop mesh that was hit
            if DEBUG_CONFIG.draw_collided_prop_aabb_hits {
                self.debug_draw_collided_aabb_hits(&aabb_collision_out, transform);
            }
        }

        self.num_hit_tris = 0;
        if !hit_any_aabb {
            return 0;
        }

        // iterate over the sectors again, this time refining the successful AABB collisions with
        // more precise triangle mesh collisions

        // compute the nearest triangle collision to the ray as the triangle whose distance
        // from the ray's initial point is smallest
        let mut min_tri_hit_dist = self.ray_len;
        let mut min_tri_hit_point = vec3::create();
        self.tri_hits.clear();

        for (sector_idx, sector) in mesh.sectors.iter().enumerate() {
            if !hit_aabbs[sector_idx] {
                continue;
            }
            for tri_group in sector.tri_groups.iter() {
                if tri_group.is_tri_strip {
                    // For a triangle strip sector, the normals of successive triangles would have
                    // normals pointing in opposite directions if we always oriented the triangle
                    // from three vertices.
                    // To account for this, we reverse the orientation of every other triangle.
                    // To reverse the orientation, we reverse the order of the first two vertices
                    // for the purposes of the ray-triangle collision algorithm.
                    let mut reverse_orientation = false;

                    for vertices in tri_group.vertices.windows(3) {
                        let mut triangle = if !reverse_orientation {
                            [vertices[0].point, vertices[1].point, vertices[2].point]
                        } else {
                            [vertices[1].point, vertices[0].point, vertices[2].point]
                        };
                        reverse_orientation = !reverse_orientation;

                        // TODO_REFACTOR: this negating should probably be done when the mono data is parsed??
                        for i in 0..3 {
                            for j in 0..3 {
                                triangle[i][j] *= -1.0;
                            }
                        }

                        let tri_hit_result =
                            self.ray_hits_triangle(&triangle, transform, ray_in_mesh_coords);
                        if let Some(tri_hit_dist) = tri_hit_result {
                            // if we hit the triangle:

                            if DEBUG_CONFIG.draw_collided_prop_tris {
                                self.debug_draw_collided_tri_hit(
                                    vertices.try_into().unwrap(),
                                    transform,
                                );
                            }

                            // TODO_DOC
                            // finish copying data to the triangle (this should probably be
                            // in `ray_hits_triangle`)
                            let mut hit = self.ray_to_triangle_hit;
                            hit.metadata = vertices[2].metadata as i32;
                            for i in 0..3 {
                                vec3::copy(&mut hit.tri[i], &vertices[i].point);
                            }

                            // save the hit to the raycast state
                            self.tri_hits.push(hit);

                            // update the minimum distance
                            if tri_hit_dist < min_tri_hit_dist {
                                self.closest_hit_idx = Some(self.tri_hits.len() as u8 - 1);
                                min_tri_hit_dist = tri_hit_dist;
                                vec3::copy(&mut min_tri_hit_point, &hit.impact_point);
                            }
                            self.num_hit_tris += 1;
                        }
                    }
                } else {
                    for vertices in tri_group.vertices.chunks_exact(3) {
                        let mut triangle =
                            [vertices[0].point, vertices[1].point, vertices[2].point];
                        for i in 0..3 {
                            for j in 0..3 {
                                triangle[i][j] *= -1.0;
                            }
                        }

                        let tri_hit_result =
                            self.ray_hits_triangle(&triangle, transform, ray_in_mesh_coords);
                        if let Some(tri_hit_dist) = tri_hit_result {
                            // if we hit the triangle:

                            // finish copying data to the triangle (this should probably be in `ray_hits_triangle`)
                            let mut hit = self.ray_to_triangle_hit;
                            hit.metadata = vertices[2].metadata as i32;
                            for i in 0..3 {
                                vec3::copy(&mut hit.tri[i], &vertices[i].point);
                            }

                            // save the hit to the raycast state
                            self.tri_hits.push(hit);

                            // update the minimum distance
                            if tri_hit_dist < min_tri_hit_dist {
                                self.closest_hit_idx = Some(self.tri_hits.len() as u8 - 1);
                                min_tri_hit_dist = tri_hit_dist;
                                vec3::copy(&mut min_tri_hit_point, &hit.impact_point);
                            }
                            self.num_hit_tris += 1;
                        }
                    }
                }
            }
        }

        // early return if no mesh triangles were hit
        if self.num_hit_tris == 0 {
            return 0;
        }

        // let impact_dist_ratio = self.get_closest_hit().unwrap().impact_dist_ratio;

        for hit in self.tri_hits.iter_mut() {
            // TODO_DOC: undo goofy hack where `ray_hits_triangle` misuses the z coordinate of the
            // impact point to store the impact distance. why?? who knows
            let impact_dist = hit.impact_point[2];

            hit.impact_dist = impact_dist;
            hit.impact_dist_ratio = impact_dist / self.ray_len;

            // TODO_DOC: no clue what this is doing
            if ray_in_mesh_coords {
                hit.impact_point = min_tri_hit_point
            } else {
                let mut normal_unit = hit.normal_unit;
                vec3_inplace_zero_small(&mut normal_unit, 0.00001);

                let ray_dot_normal = vec3::dot(&normal_unit, &self.ray_unit);
                let t = (1.0 - hit.impact_dist_ratio - 0.0005) * self.ray_len;

                vec3::scale_and_add(
                    &mut hit.impact_point,
                    &self.point1,
                    &normal_unit,
                    t * ray_dot_normal,
                );
            }
        }

        self.num_hit_tris as i32
    }

    fn debug_draw_collided_aabb_hits(&self, aabb_collision_out: &Vec3, transform: &Mat4) {
        static AABB_HIT_COLOR: Vec4 = [0.7, 1.0, 0.3, 1.0];

        if let Some(delegates) = &self.delegates {
            let mut my_delegates = delegates.borrow_mut();

            let mut world_point = vec3::create();
            vec3::transform_mat4(&mut world_point, &aabb_collision_out, &transform);

            my_delegates
                .debug_draw
                .draw_point(&world_point, &AABB_HIT_COLOR);
        }
    }

    fn debug_draw_collided_tri_hit(&self, triangle: &[TriVertex; 3], transform: &Mat4) {
        static TRIANGLE_HIT_COLOR: Vec4 = [0.8, 0.4, 0.3, 0.5];

        if let Some(delegates) = &self.delegates {
            let mut my_delegates = delegates.borrow_mut();

            let tri_group = TriGroup {
                is_tri_strip: false,
                vertices: triangle.to_vec(),
            };

            my_delegates
                .debug_draw
                .draw_tri_group(&tri_group, transform, &TRIANGLE_HIT_COLOR);
        }
    }

    /// Checks if the ray of length `2 * dist` straight down from `pos` intersects a zone.
    /// If it does, returns that zone's id.
    /// offset: 0x36d70
    pub fn find_zone_below_point(
        &mut self,
        pos: &Vec3,
        dist: f32,
        transform: &Mat4,
        zone_mesh: &Mesh,
    ) -> Option<u8> {
        let below_pos = vec3_from!(-, pos, [0.0, dist + dist, 0.0]);
        self.load_ray(&pos, &below_pos);

        if self.ray_hits_zone(transform, zone_mesh) != 0 {
            self.get_closest_hit().map(|hit| hit.metadata as u8)
        } else {
            None
        }
    }

    /// offset: 0x276e0
    fn ray_hits_zone(&mut self, transform: &Mat4, zone_mesh: &Mesh) -> i32 {
        self.ray_hits_mesh(zone_mesh, transform, true)
    }
}

/// Returns `true` if the line segment from `p0` to `p1` meets the AABB with opposite corner points
/// `aabb_min` and `aabb_max`.
/// The `out` writes an intersection point if one exists - possibly the one furthest from `p0`.
/// `out` doesn't seem to be used by the simulation.
/// offset: 0x106b0
fn ray_hits_aabb(p0: &Vec3, p1: &Vec3, aabb_min: &Vec3, aabb_max: &Vec3, out: &mut Vec3) -> bool {
    let [min_x, min_y, min_z] = *aabb_min;
    let [max_x, max_y, max_z] = *aabb_max;
    let [p0x, p0y, p0z] = *p0;
    let [p1x, p1y, p1z] = *p1;

    // if the entire range of some coordinate plane along the ray p0->p1 doesn't coincide
    // with the box's corresponding coordinate range, we already know there's no intersection
    if (p1x < min_x && p0x < min_x) || (p1x > max_x && p0x > max_x) {
        return false;
    }
    if (p1y < min_y && p0y < min_y) || (p1y > max_y && p0y > max_y) {
        return false;
    }
    if (p1z < min_z && p0z < min_z) || (p1z > max_z && p0z > max_z) {
        return false;
    }

    // if `p0` is inside the box, then that's the intersection point
    if p0x > min_x && p0y > min_y && p0z > min_x && p0x < max_x && p0y < max_y && p0z < max_z {
        vec3::copy(out, &p0);
        return true;
    }

    let min_p0 = vec3_from!(-, p0, aabb_min);
    let min_p1 = vec3_from!(-, p1, aabb_min);
    let p0_p1 = vec3_from!(-, p1, p0);

    if min_p0[0] * min_p1[0] < 0.0 && min_p0[0] != min_p1[0] {
        let t = min_p0[0] / (min_p0[0] - min_p1[0]);
        vec3::scale_and_add(out, p0, &p0_p1, t);

        if min_z < out[2] && out[2] < max_z && min_y < out[1] && out[1] < max_y {
            return true;
        }
    }

    if min_p1[1] * min_p0[1] < 0.0 && min_p0[1] != min_p1[1] {
        let t = min_p0[1] / (min_p0[1] - min_p1[1]);
        vec3::scale_and_add(out, p0, &p0_p1, t);

        if min_z < out[2] && out[2] < max_z && min_x < out[0] && out[0] < max_x {
            return true;
        }
    }

    if min_p0[2] * min_p1[2] < 0.0 && min_p0[2] != min_p1[2] {
        let t = min_p0[2] / (min_p0[2] - min_p1[2]);
        vec3::scale_and_add(out, p0, &p0_p1, t);

        if min_x < out[0] && out[0] < max_x && min_y < out[1] && out[1] < max_y {
            return true;
        }
    }

    let max_p0 = vec3_from!(-, p0, aabb_max);
    let max_p1 = vec3_from!(-, p1, aabb_max);

    if max_p0[0] * max_p1[0] < 0.0 && max_p0[0] != max_p1[0] {
        let t = max_p0[0] / (max_p0[0] - max_p1[0]);
        vec3::scale_and_add(out, p0, &p0_p1, t);

        if min_z < out[2] && out[2] < max_z && min_y < out[1] && out[1] < max_y {
            return true;
        }
    }

    if max_p0[1] * max_p1[1] < 0.0 && max_p0[1] != max_p1[1] {
        let t = max_p0[1] / (max_p0[1] - max_p1[1]);
        vec3::scale_and_add(out, p0, &p0_p1, t);

        if min_z < out[2] && out[2] < max_z && min_x < out[0] && out[0] < max_x {
            return true;
        }
    }

    if max_p0[2] * max_p1[2] < 0.0 && max_p0[2] != max_p1[2] {
        let t = max_p0[2] / (max_p0[2] - max_p1[2]);
        vec3::scale_and_add(out, p0, &p0_p1, t);

        if min_x < out[0] && out[0] < max_x && min_y < out[1] && out[1] < max_y {
            return true;
        }
    }

    false
}

// #[cfg(test)]
// mod tests {
//     use super::RaycastState;

//     #[test]
//     fn test_triangle_collision() {
//         let triangle = [
//             [-2.6363091468811, 0.020901577547193, -4.1993327140808],
//             [-2.6363091468811, -6.4182171821594, 4.274188041687],
//             [-2.6363091468811, -6.4182171821594, -4.199333190918],
//         ];
//         let ray = [
//             [-26.963624954224, -24.175483703613, 17.492353439331],
//             [-25.76362991333, -25.552066802979, 15.770400047302],
//         ];
//         let result = [
//             [-26.267070770264, -24.974540710449, 1.4569648504257],
//             [0.70710015296936, 2.0945599032984e-007, 0.70711332559586],
//         ];
//         let t = 1.46;

//         let mut raycast_state = RaycastState::default();
//         raycast_state.load_ray(&ray[0], &ray[1]);
//         // raycast_state.ray_hits_triangle(&triangle, &None);
//         println!("result: {:?}", raycast_state.ray_to_triangle_hit);
//     }
// }
