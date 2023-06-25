use gl_matrix::common::Vec3;

use super::aabb::Aabb;

#[derive(Debug, Default, Clone, Copy)]
pub struct TriVertex {
    pub point: Vec3,
    pub metadata: u32,
}

#[derive(Debug, Default)]
pub struct TriGroup {
    // If true, the triangle group is encoded as a "triangle strip"
    pub is_tri_strip: bool,
    pub vertices: Vec<TriVertex>,
}

// A mesh sector is a sequence of triangle groups, contained within an AABB.
// Collision with the sector can be tested by first checking collision with the AABB
// interior first, and then the triangle groups second.
#[derive(Debug, Default)]
pub struct MeshSector {
    pub aabb: Aabb,
    pub tri_groups: Vec<TriGroup>,
}

#[derive(Debug, Default)]
pub struct Mesh {
    pub sectors: Vec<MeshSector>,
}
