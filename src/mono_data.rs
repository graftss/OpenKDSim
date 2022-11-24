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
pub struct TriVertex {
    pub point: Vec3,
    pub metadata: u32,
}

#[derive(Debug, Default)]
pub struct TriGroup {
    pub aabb: Aabb,
    pub is_tri_strip: bool,
    pub vertices: Vec<TriVertex>,
}

#[derive(Debug, Default)]
pub struct TriMesh {
    pub tri_groups: Vec<TriGroup>,
}

pub type MonoDataPtr = Option<usize>;

const NUM_MONO_DATA_PROP_PTRS: usize = 11;

/// The sequence of bytes in the monodata that indicates a null value.
/// Thus, any offset pointing to this sequence can be seen as a null pointer.
const NIL: u64 = 0x206c696e204c494e;

/// Pointers into mono data for one type of prop.
#[derive(Debug, Default)]
pub struct PropMonoData {
    ptrs: [MonoDataPtr; NUM_MONO_DATA_PROP_PTRS],
    triangle_mesh: Option<TriMesh>,
    vault_points: Option<Vec<Vec3>>,
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
            triangle_mesh: ptrs[7].map(|ptr| PropMonoData::parse_triangle_mesh(ptr as *const u8)),
            vault_points: ptrs[8]
                .map(|ptr| PropMonoData::parse_vault_points(ptr as *const u8, name_idx)),
        }
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
    unsafe fn parse_triangle_mesh(mono_data: *const u8) -> TriMesh {
        let mut tri_groups = vec![];

        // parse the number of triangle groups (mesh offset 0)
        let num_groups = md_read!(mono_data, u8, 0);

        // parse the offset where AABBs are laid out (mesh offset 4)
        let aabbs_offset = md_read!(mono_data, u32, 4) as isize;

        // for each triangle group:
        for i in 0..num_groups as isize {
            // parse the triangle group's AABB ()
            let aabb_offset = i * 0x18 + aabbs_offset + 8;
            let aabb = Aabb {
                min: md_read!(mono_data, Vec3, aabb_offset),
                max: md_read!(mono_data, Vec3, aabb_offset + 12),
            };

            // parse the offset where the vertex header is
            let vertex_header_offset = md_read!(mono_data, u32, i * 4 + 8) as isize;

            // parse the 1-byte header before the vertex list, which encodes:
            //   - whether the vertex list is a triangle strip or not
            //   - the number of encoded vertices following the header
            let vertex_header = md_read!(mono_data, u8, vertex_header_offset);

            let is_tri_strip = vertex_header & 0x80 != 0;
            let num_vertices = if is_tri_strip {
                (vertex_header & 0x7f) + 2
            } else {
                (vertex_header & 0x7f) * 2
            };

            let mut vertices = vec![];
            let mut vertex_offset = vertex_header_offset + 8;

            for _ in 0..num_vertices as isize {
                vertices.push(TriVertex {
                    point: md_read!(mono_data, Vec3, vertex_offset),
                    metadata: md_read!(mono_data, u32, vertex_offset + 12),
                });
                vertex_offset += 0x10;
            }

            tri_groups.push(TriGroup {
                aabb,
                is_tri_strip,
                vertices,
            });
        }

        TriMesh { tri_groups }
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
