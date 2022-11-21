use crate::constants::NUM_NAME_PROPS;

pub type MonoDataPtr = Option<usize>;

const NUM_MONO_DATA_PROP_PTRS: usize = 11;

const NIL: u64 = 0x206c696e204c494e;

/// Pointers into mono data for one type of prop.
#[derive(Debug, Default, Clone, Copy)]
pub struct MonoDataPropPtrs {
    ptrs: [MonoDataPtr; NUM_MONO_DATA_PROP_PTRS],
}

impl MonoDataPropPtrs {
    pub fn get_collision_mesh(&self) -> MonoDataPtr {
        self.ptrs[7]
    }

    pub fn get_vault_points(&self) -> MonoDataPtr {
        self.ptrs[8]
    }
}

#[derive(Debug, Default)]
pub struct MonoData {
    pub zones: MonoDataPtr,
    pub areas: [MonoDataPtr; 5],
    pub props: Vec<MonoDataPropPtrs>,
}

// this is necessary to make `static_init` happy about having a `MonoData` field in
// the global `GameState` struct.

macro_rules! md_read_u32 {
    ($md: ident, $offset: expr) => {
        *($md.offset($offset).cast::<u32>().as_ref().unwrap())
    };
}

macro_rules! md_follow_offset {
    ($md: ident, $offset: expr) => {
        $md.offset(md_read_u32!($md, $offset).try_into().unwrap())
    };
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
            let mut prop_offsets: [MonoDataPtr; NUM_MONO_DATA_PROP_PTRS] =
                [None; NUM_MONO_DATA_PROP_PTRS];

            for (ptr_idx, ptr) in prop_offsets.iter_mut().enumerate() {
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

            self.props.push(MonoDataPropPtrs { ptrs: prop_offsets });
        }
    }
}
