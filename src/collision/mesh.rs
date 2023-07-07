use std::fmt::Display;

use gl_matrix::common::Vec3;

use crate::collision::hit_attribute::HitAttribute;

use super::aabb::Aabb;

#[derive(Debug, Default, Clone, Copy)]
pub struct TriVertex {
    pub point: Vec3,
    pub metadata: u32,
}

impl Display for TriVertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let metadata_str = if self.metadata == 0 {
            "".to_owned()
        } else {
            format!("[{:?}] ", HitAttribute::from(self.metadata as i32))
        };

        write!(f, "{metadata_str}{:?}", self.point)
    }
}

#[derive(Debug, Default, Clone)]
pub struct TriGroup {
    // If true, the triangle group is encoded as a "triangle strip"
    pub is_tri_strip: bool,
    pub vertices: Vec<TriVertex>,
}

impl Display for TriGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, vertex) in self.vertices.iter().enumerate() {
            writeln!(f, "      Vertex {idx}: {vertex}")?;
        }
        Ok(())
    }
}

// A mesh sector is a sequence of triangle groups, contained within an AABB.
// Collision with the sector can be tested by first checking collision with the AABB
// interior first, and then the triangle groups second.
#[derive(Debug, Default, Clone)]
pub struct MeshSector {
    pub aabb: Aabb,
    pub tri_groups: Vec<TriGroup>,
}

impl Display for MeshSector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, tri_group) in self.tri_groups.iter().enumerate() {
            writeln!(
                f,
                "    TriGroup {idx} (tristrip={})\n    {}",
                tri_group.is_tri_strip, self.aabb
            )?;
            write!(f, "{tri_group}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Mesh {
    pub sectors: Vec<MeshSector>,
}

impl Display for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let num_sectors = self.sectors.len();
        writeln!(f, "Mesh [{num_sectors} sectors]:")?;
        for (idx, sector) in self.sectors.iter().enumerate() {
            writeln!(f, "  Sector {idx}:")?;
            write!(f, "{sector}")?;
        }

        Ok(())
    }
}
