use std::fmt::Display;

use gl_matrix::common::Vec3;

use crate::{collision::hit_attribute::HitAttribute, macros::md_read};

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

impl Mesh {
    /// Parse a triangle mesh from mono data.
    /// The argument `mono_data` should point to the first byte of the triangle mesh.
    pub unsafe fn from_raw(mono_data: *const u8) -> Mesh {
        // parse the number of sectors (mesh offset 0)
        let num_sectors = md_read!(mono_data, u8, 0);

        // for each sector:
        let mut sectors = vec![];
        for sector_idx in 0..num_sectors as isize {
            // parse the offset where sector starts
            let sector_offset = md_read!(mono_data, u32, sector_idx * 4 + 4) as isize;

            // parse the sector's AABB
            let aabb_offset = sector_offset + sector_idx * 0x18 + 8;
            let mut aabb = Aabb {
                min: md_read!(mono_data, Vec3, aabb_offset),
                max: md_read!(mono_data, Vec3, aabb_offset + 12),
            };
            aabb.negate_coords();

            // parse the offset of the first triangle group
            let mut tri_group_offset = md_read!(mono_data, u32, sector_idx * 4 + 8) as isize;

            // parse the contiguous list of triangle groups in the sector
            let mut tri_groups = vec![];
            loop {
                // parse the 1-byte header before the vertex list, which encodes:
                //   - whether the vertex list is a triangle strip or not
                //   - the number of encoded vertices following the header
                let vertex_header: u32 = md_read!(mono_data, u8, tri_group_offset).into();

                let is_tri_strip = vertex_header & 0x80 != 0;
                let num_vertices = if is_tri_strip {
                    (vertex_header & 0x7f) + 2
                } else {
                    (vertex_header & 0x7f) * 3
                };

                // parse the list of vertices
                let mut vertices = vec![];
                let mut vertex_offset = tri_group_offset + 8;
                for _ in 0..num_vertices as isize {
                    vertices.push(TriVertex {
                        point: md_read!(mono_data, Vec3, vertex_offset),
                        metadata: md_read!(mono_data, u32, vertex_offset + 0xc),
                    });
                    vertex_offset += 0x10;
                }

                // create the triangle group
                tri_groups.push(TriGroup {
                    is_tri_strip,
                    vertices,
                });

                // update the triangle group offset to the beginning of the next group
                tri_group_offset += 0xc + 0x10 * (num_vertices as isize);

                // parse the vifcode to detect the end of the sector
                let vif = md_read!(mono_data, u8, tri_group_offset - 1);
                if vif == 0x97 {
                    break;
                }
            }

            sectors.push(MeshSector { aabb, tri_groups });
        }

        Mesh { sectors }
    }
}
