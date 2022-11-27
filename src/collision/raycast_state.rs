use gl_matrix::common::{Mat4, Vec3};

/// A triangle hit by a raycast.
#[derive(Debug, Default)]
pub struct RaycastTriHit {
    /// The point on the triangle that was hit.
    /// offset: 0x0
    pub hit_point: Vec3,

    /// The unit normal of the hit triangle.
    /// offset: 0x10
    pub tri_normal_unit: Vec3,

    /// The three points of the triangle.
    /// offset: 0x20
    pub tri: [Vec3; 3],

    /// Metadata of the triangle, which can be either a hit attribute (for
    /// katamari-surface collisions) or a zone index (for prop-surface collisions).
    /// offset: 0x50
    pub metadata: i32,

    /// The ratio of the hit distance to the ray length.
    /// offset: 0x54
    pub hit_dist_ratio: f32,

    /// The distance from the ray initial point to the hit point.
    /// offset: 0x58
    pub hit_dist: f32,
}

/// TODO: Encodes data about a single raycast.
/// offset: 0x1941e0 (it's allocated in the heap, but this is a pointer to it)
#[derive(Debug, Default)]
pub struct RaycastState {
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

    /// The inverse "look at" matrix from p0 to p1 (which I think means it's
    /// the look at matrix from p1 to p0).
    /// offset: 0x80
    pub ray_lookat_inv: Mat4,

    /// The transpose of the "look at" matrix from p0 to p1.
    /// offset: 0xc0
    pub ray_lookat_t: Mat4,

    /// (??)
    /// offset: 0x110
    pub ray_lookat_times_p1: Vec3,

    /// The length of the collision ray from p0 to p1.
    /// offset: 0x120
    pub ray_len: f32,

    /// The length squared of the collision ray from p0 to p1.
    /// offset: 0x124
    pub ray_len_sq: f32,

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
