use std::rc::Rc;

use gl_matrix::common::Vec3;

use crate::{
    collision::{aabb::Aabb, mesh::Mesh},
    constants::NUM_NAME_PROPS,
    macros::{max, md_follow_offset, md_read, min},
    props::config::NamePropConfig,
};

#[derive(Debug, Default)]
pub struct PropAabbs {
    /// The first element is the AABB of the prop.
    /// Remaining elements are the AABBs of the prop's subobjects, if it has any.
    aabbs: Vec<Aabb>,
}

impl PropAabbs {
    /// Parses the AABBs of the prop and its subobjects (if it has any) from a raw pointer.
    unsafe fn from_raw(mono_data: *const u8) -> PropAabbs {
        let num_aabbs = md_read!(mono_data, u8, 0);
        let mut aabbs = vec![];

        for i in 0..num_aabbs as isize {
            let aabb_offset = md_read!(mono_data, u32, i * 4 + 4) as isize;
            let min = md_read!(mono_data, Vec3, aabb_offset);
            let max = md_read!(mono_data, Vec3, aabb_offset + 0x10);

            aabbs.push(Aabb { min, max });
        }

        PropAabbs { aabbs }
    }

    /// Get the prop's AABB.
    pub fn get_prop_aabb(&self) -> &Aabb {
        &self.aabbs[0]
    }

    /// Get the AABB of the `subobj_idx`-th subobject.
    pub fn get_subobject_aabb(&self, subobj_idx: i32) -> Option<&Aabb> {
        self.aabbs.get(1 + subobj_idx as usize)
    }

    /// Compute the "root" AABB which encloses both the prop and all of its subobjects.
    pub fn get_root_aabb(&self) -> Aabb {
        // TODO_SUBOBJECT: rotate the AABB sector of each subobject separately
        static SUBOBJECT_ROT: Vec3 = [0.0; 3];

        let mut root_max = [0.0; 3];
        let mut root_min = [0.0; 3];
        for aabb in self.aabbs.iter() {
            root_min[0] = min!(root_min[0], aabb.min[0]);
            root_min[1] = min!(root_min[1], aabb.min[1]);
            root_min[2] = min!(root_min[2], aabb.min[2]);
            root_max[0] = max!(root_max[0], aabb.max[0]);
            root_max[1] = max!(root_max[1], aabb.max[1]);
            root_max[2] = max!(root_max[2], aabb.max[2]);
        }

        Aabb {
            min: root_min,
            max: root_max,
        }
    }
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
    pub aabbs: Option<Rc<PropAabbs>>,
    pub collision_mesh: Option<Rc<Mesh>>,
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
            aabbs: ptrs[0]
                .map(|ptr| PropAabbs::from_raw(ptr as *const u8))
                .map(Rc::new),
            collision_mesh: ptrs[7]
                .map(|ptr| Mesh::from_raw(ptr as *const u8))
                .map(Rc::new),
            vault_points: ptrs[8]
                .map(|ptr| PropMonoData::parse_vault_points(ptr as *const u8, name_idx)),
        }
    }

    /// Parse a list of vault points from mono data.
    /// Vault points are encoded as a contiguous array of `Vec4` values where each
    /// `w` component is 1.0 (which can be ignored, resulting in a list of `Vec3` points).
    unsafe fn parse_vault_points(mono_data: *const u8, name_idx: usize) -> Vec<Vec3> {
        let mut result = vec![];
        let num_vault_pts = NamePropConfig::get(name_idx as u16).num_vault_pts;

        for i in 0..num_vault_pts as isize {
            result.push(md_read!(mono_data, Vec3, i * 0x10));
        }

        result
    }
}

#[derive(Debug, Default, Clone)]
pub struct MonoData {
    pub zone_ptr: MonoDataPtr,
    pub zone_mesh: Mesh,
    pub area_ptrs: [MonoDataPtr; 5],
    pub props: Vec<Rc<PropMonoData>>,
}

impl MonoData {
    pub unsafe fn init_from_raw(&mut self, mono_data: *const u8) {
        // read zone pointer
        let zone_ptr = md_follow_offset!(mono_data, 0x4) as usize;
        self.zone_ptr = Some(zone_ptr);

        // parse zone into mesh
        self.zone_mesh = Mesh::from_raw(zone_ptr as *const u8);

        // read area pointers
        self.area_ptrs = [
            Some(md_follow_offset!(mono_data, 0x14) as usize),
            Some(md_follow_offset!(mono_data, 0x18) as usize),
            Some(md_follow_offset!(mono_data, 0x1c) as usize),
            Some(md_follow_offset!(mono_data, 0x20) as usize),
            Some(md_follow_offset!(mono_data, 0x24) as usize),
        ];

        for name_idx in 0..NUM_NAME_PROPS {
            let prop_data = PropMonoData::new(mono_data, name_idx);
            self.props.push(Rc::new(prop_data));
        }
    }
}
