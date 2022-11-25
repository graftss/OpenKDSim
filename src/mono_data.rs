use gl_matrix::common::Vec3;

use crate::{constants::NUM_NAME_PROPS, name_prop_config::NamePropConfig};

macro_rules! md_read {
    ($md: ident, $type: ty, $offset: expr) => {
        *($md.offset($offset).cast::<$type>().as_ref().unwrap())
    };

    ($md: ident, $type: ty, $offset: expr) => {
        *($md.offset($offset).cast::<$type>().as_ref().unwrap())
    };
}

macro_rules! md_follow_offset {
    ($md: ident, $offset: expr) => {
        $md.offset(md_read!($md, u32, $offset).try_into().unwrap())
    };
}

#[derive(Debug, Default)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Debug, Default)]
pub struct PropAabbs {
    /// The first element is the AABB of the prop.
    /// Remaining elements are the AABBs of the prop's subobjects, if it has any.
    aabbs: Vec<Aabb>,
}

impl PropAabbs {
    /// Get the prop's AABB.
    pub fn get_prop_aabb(&self) -> &Aabb {
        &self.aabbs[0]
    }

    /// Get the AABB of the `subobj_idx`-th subobject.
    pub fn get_subobject_aabb(&self, subobj_idx: i32) -> Option<&Aabb> {
        self.aabbs.get(1 + subobj_idx as usize)
    }
}

#[derive(Debug, Default)]
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
pub struct PropMesh {
    pub sectors: Vec<MeshSector>,
}

pub type MonoDataPtr = Option<usize>;

const NUM_MONO_DATA_PROP_PTRS: usize = 11;

/// The sequence of bytes in the monodata that indicates a null value.
/// Thus, any offset pointing to this sequence can be seen as a null pointer.
const NIL: u64 = 0x206c696e204c494e;

/// Pointers into mono data for one type of prop.
#[derive(Debug, Default)]
pub struct PropMonoData {
    pub ptrs: [MonoDataPtr; NUM_MONO_DATA_PROP_PTRS],
    pub aabbs: Option<PropAabbs>,
    pub triangle_mesh: Option<PropMesh>,
    pub vault_points: Option<Vec<Vec3>>,
}

impl PropMonoData {
    pub unsafe fn new(mono_data: *const u8, name_idx: usize) -> PropMonoData {
        let mut ptrs: [MonoDataPtr; NUM_MONO_DATA_PROP_PTRS] = [None; NUM_MONO_DATA_PROP_PTRS];

        for (ptr_idx, ptr) in ptrs.iter_mut().enumerate() {
            let offset: i32 = 0x28 + (name_idx as i32) * 0x2c + (ptr_idx as i32) * 0x4;
            let try_ptr = md_follow_offset!(mono_data, offset.try_into().unwrap());

            // detect the weird null pointers in the mono data here.
            // if an offset points to null, represent it in our mono data as `None`.
            if *(try_ptr.cast::<u64>().as_ref().unwrap()) == NIL {
                *ptr = None;
            } else {
                *ptr = Some(try_ptr as usize);
            }
        }

        PropMonoData {
            ptrs,
            aabbs: ptrs[0].map(|ptr| PropMonoData::parse_aabbs(ptr as *const u8)),
            triangle_mesh: ptrs[7].map(|ptr| PropMonoData::parse_mesh(ptr as *const u8)),
            vault_points: ptrs[8]
                .map(|ptr| PropMonoData::parse_vault_points(ptr as *const u8, name_idx)),
        }
    }

    /// Parses the AABBs of the prop and its subobjects (if it has any).
    unsafe fn parse_aabbs(mono_data: *const u8) -> PropAabbs {
        let num_aabbs = md_read!(mono_data, u8, 0);
        let mut aabbs = vec![];

        for i in 0..num_aabbs as isize {
            let aabb_offset = md_read!(mono_data, u32, i * 4 + 4) as isize;
            aabbs.push(Aabb {
                min: md_read!(mono_data, Vec3, aabb_offset),
                max: md_read!(mono_data, Vec3, aabb_offset + 0x10),
            });
        }

        PropAabbs { aabbs }
    }

    /// Parse a list of vault points from mono data.
    /// Vault points are encoded as a contiguous array of `Vec4` values where each
    /// `w` component is 1.0 (which can be ignored, resulting in a list of `Vec3` points).
    unsafe fn parse_vault_points(mono_data: *const u8, name_idx: usize) -> Vec<Vec3> {
        let mut result = vec![];
        let num_vault_pts = NamePropConfig::get(name_idx as i32).num_vault_pts;

        for i in 0..num_vault_pts as isize {
            result.push(md_read!(mono_data, Vec3, i * 0x10));
        }

        result
    }

    /// Parse a triangle mesh from mono data.
    /// The argument `mono_data` should point to the first byte of the triangle mesh.
    unsafe fn parse_mesh(mono_data: *const u8) -> PropMesh {
        // parse the number of sectors (mesh offset 0)
        let num_sectors = md_read!(mono_data, u8, 0);

        // for each sector:
        let mut sectors = vec![];
        for sector_idx in 0..num_sectors as isize {
            // parse the offset where sector starts
            let sector_offset = md_read!(mono_data, u32, sector_idx * 4 + 4) as isize;

            // parse the sector's AABB
            let aabb_offset = sector_offset + sector_idx * 0x18 + 8;
            let aabb = Aabb {
                min: md_read!(mono_data, Vec3, aabb_offset),
                max: md_read!(mono_data, Vec3, aabb_offset + 12),
            };

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

        PropMesh { sectors }
    }
}

#[derive(Debug, Default)]
pub struct MonoData {
    pub zones: MonoDataPtr,
    pub areas: [MonoDataPtr; 5],
    pub props: Vec<PropMonoData>,
}

impl MonoData {
    pub unsafe fn init(&mut self, mono_data: *const u8) {
        // read zone pointer
        self.zones = Some(md_follow_offset!(mono_data, 0x4) as usize);

        // read area pointers
        self.areas = [
            Some(md_follow_offset!(mono_data, 0x14) as usize),
            Some(md_follow_offset!(mono_data, 0x18) as usize),
            Some(md_follow_offset!(mono_data, 0x1c) as usize),
            Some(md_follow_offset!(mono_data, 0x20) as usize),
            Some(md_follow_offset!(mono_data, 0x24) as usize),
        ];

        for name_idx in 0..NUM_NAME_PROPS {
            self.props.push(PropMonoData::new(mono_data, name_idx));
        }
    }
}
