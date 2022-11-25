use gl_matrix::{
    common::{Vec3},
    vec3,
};

use crate::constants::AABB_TRIANGULATION;

use super::mesh::{Mesh, MeshSector, TriGroup, TriVertex};

#[derive(Debug, Default, Clone)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// Compute the side lengths of this AABB.
    pub fn size(&self) -> Vec3 {
        let Self { min, max } = self;

        [max[0] - min[0], max[1] - min[1], max[2] - min[2]]
    }

    /// Enumerate the 8 corner points of this AABB.
    pub fn compute_vertices(&self) -> Vec<Vec3> {
        let Self { min, max } = self;

        vec![
            [min[0], min[1], min[2]],
            [min[0], max[1], min[2]],
            [max[0], min[1], min[2]],
            [max[0], max[1], min[2]],
            [max[0], min[1], max[2]],
            [max[0], max[1], max[2]],
            [min[0], min[1], max[2]],
            [min[0], max[1], max[2]],
        ]
    }

    /// Compute the bounding radius of this AABB, which is defined by the
    /// maximum distance from (0,0,0) to a corner of the AABB.
    pub fn compute_radius(&self) -> f32 {
        let mut max_coords = [0.0; 3];
        for i in 0..3 {
            max_coords[i] = f32::max(self.min[i].abs(), self.max[i].abs());
        }
        vec3::length(&max_coords)
    }

    /// Triangulate this AABB into a mesh.
    pub fn compute_mesh(&self, aabb_vertices: &Vec<Vec3>) -> Mesh {
        // enumerate the vertices of the triangulated AABB
        let mut vertices = vec![];
        for v_i in AABB_TRIANGULATION {
            vertices.push(TriVertex {
                metadata: 1,
                point: aabb_vertices[v_i as usize],
            });
        }

        let sector = MeshSector {
            aabb: self.clone(),
            tri_groups: vec![TriGroup {
                is_tri_strip: false,
                vertices,
            }],
        };

        Mesh {
            sectors: vec![sector],
        }
    }
}
