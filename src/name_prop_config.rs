use lazy_static::lazy_static;

const NUM_NAME_PROPS: usize = 1718;

static NP_0X30_TABLE: &'static [u8] = include_bytes!("data/name_prop_0x30_table.bin");
static NP_MONO_DATA_OFFSETS: &'static [u8] = include_bytes!("data/name_prop_mono_data_offsets.bin");

pub struct NamePropConfig {
    ///////////////////////////////////////////
    // Begin `name_prop_0x30_table.bin` fields.
    ///////////////////////////////////////////

    // Offsets are relative to each prop's 0x30-width entry in that table.

    /// (??)
    /// offset: 0x8
    pub compare_vol_mult: f32,

    /// The amount of the prop's volume that is added to the katamari after attachment.
    /// offset: 0xc
    pub attach_vol_mult: f32,

    /// If true, the prop should not ever adjust its pitch angle.
    /// offset: 0x10
    pub lock_pitch: bool,

    /// If true, the prop can flee from the player.
    /// offset: 0x12
    pub can_flee: bool,

    /// The prop's number of vault points.
    /// offset: 0x15
    pub num_vault_pts: u8,

    /// The prop's movement speed index, both along paths and when fleeing.
    /// The value is an index into another table (TODO: where is it)
    /// offset: 0x17
    pub move_speed_idx: u8,

    /// The prop's type of innate motion action (e.g. spherical objects do an inelastic collision
    /// when collided with, "fight" objects spin around, etc).
    /// offset: 0x18
    pub innate_motion_type: u8,

    /// If true, the prop can't wobble when hit by the player.
    /// offset: 0x1b
    pub cannot_wobble: bool,

    /// If true, the prop's triangulated AABB is used as its collision mesh.
    /// offset: 0x1d
    pub use_aabb_as_collision_mesh: bool,

    /// (??) If true, the prop tracks the distance from itself to the katamari.
    /// offset: 0x1e
    pub kat_proximity_aware: bool,

    /// If true, the prop is a fish (and should do the constant bouncing motion).
    /// offset: 0x1f
    pub is_fish: bool,

    /// If true, the prop may be knocked airborne instead of collected by the katamari (if it's moving).
    /// offset: 0x20
    pub can_be_airborne: bool,

    /// The prop's scream SFX index.
    /// offset: 0x21
    pub scream_sfx_idx: u8,

    /// (??) the name index of a constant parent prop
    /// offset: 0x22
    pub const_parent_name_idx: u16,

    /// If true, plays the "treasure" FX on collection (e.g. the crown and trophy props)
    /// offset: 0x24
    pub has_treasure_fx: bool,

    /// If true, the prop is a "dummy hit" object.
    /// offset: 0x25
    pub is_dummy_hit: bool,

    /// If true, the prop can be collected from further away (by virtue of pretending the katamari
    /// is bigger than it actually is when checking for the kat-prop collision).
    /// Used to make flat objects (e.g. "welcome mat") easier to collect when laying flat on the ground.
    /// offset: 0x26
    pub collect_from_further: bool,

    /// If true, the prop is an unhatched egg and will need to transform when collected.
    /// offset: 0x27
    pub is_unhatched_egg: bool,

    /////////////////////////////////////////
    // End `name_prop_0x30_table.bin` fields.
    /////////////////////////////////////////

    /// (??) Used by `GetMonoDataOffset`.
    /// Read from `name_prop_mono_data_offsets.bin`.
    pub mono_data_offset_idx: u16,
}

impl NamePropConfig {
    pub fn mono_data_offset_exists(&self) -> bool {
        self.mono_data_offset_idx != u16::MAX
    }
}

impl NamePropConfig {
    pub fn get(name_idx: i32) -> &'static NamePropConfig {
        &NAME_PROP_CONFIGS[name_idx as usize]
    }

    pub fn read_from_data(configs: &mut [NamePropConfig; NUM_NAME_PROPS]) {
        Self::read_name_prop_0x30_table(configs);
        Self::read_name_prop_mono_data_offsets(configs);
    }

    /// Copy the `name_prop_0x30_table` file into the `NamePropConfig` array.
    pub fn read_name_prop_0x30_table(configs: &mut [NamePropConfig; NUM_NAME_PROPS]) {
        let ENTRY_SIZE = 0x30;
        for (name_idx, config) in configs.iter_mut().enumerate() {
            // it's fine
            let base = name_idx * ENTRY_SIZE + 0x8;

            config.compare_vol_mult = f32::from_le_bytes(NP_0X30_TABLE[base+0x0..base+0x4].try_into().unwrap());
            config.attach_vol_mult = f32::from_le_bytes(NP_0X30_TABLE[base+0x4..base+0x8].try_into().unwrap());
            config.lock_pitch = u8::from_le(NP_0X30_TABLE[base+0x8]) != 0;
            config.can_flee = u8::from_le(NP_0X30_TABLE[base+0x9]) != 0;
            config.num_vault_pts = u8::from_le(NP_0X30_TABLE[base+0xd]);
            config.move_speed_idx = u8::from_le(NP_0X30_TABLE[base+0xf]);
            config.innate_motion_type = u8::from_le(NP_0X30_TABLE[base+0x10]);
            config.cannot_wobble = u8::from_le(NP_0X30_TABLE[base+0x13]) != 0;
            config.use_aabb_as_collision_mesh = u8::from_le(NP_0X30_TABLE[base+0x15]) != 0;
            config.kat_proximity_aware = u8::from_le(NP_0X30_TABLE[base+0x16]) != 0;
            config.is_fish = u8::from_le(NP_0X30_TABLE[base+0x17]) != 0;
            config.can_be_airborne = u8::from_le(NP_0X30_TABLE[base+0x18]) != 0;
            config.scream_sfx_idx = u8::from_le(NP_0X30_TABLE[base+0x19]);
            config.const_parent_name_idx = u16::from_le_bytes(NP_0X30_TABLE[base+0x1a..base+0x1c].try_into().unwrap());
            config.has_treasure_fx = u8::from_le(NP_0X30_TABLE[base+0x1c]) != 0;
            config.is_dummy_hit = u8::from_le(NP_0X30_TABLE[base+0x1d]) != 0;
            config.collect_from_further = u8::from_le(NP_0X30_TABLE[base+0x1e]) != 0;
            config.is_unhatched_egg = u8::from_le(NP_0X30_TABLE[base+0x1f]) != 0;
        }
    }

    /// Copy the `name_prop_mono_data_offsets.bin` file into the `NamePropConfig` array.
    pub fn read_name_prop_mono_data_offsets(configs: &mut [NamePropConfig; NUM_NAME_PROPS]) {
        for (name_idx, config) in configs.iter_mut().enumerate() {
            let offset = name_idx * 2;
            config.mono_data_offset_idx = u16::from_le_bytes(NP_MONO_DATA_OFFSETS[offset..offset + 2].try_into().unwrap());
        }
    }
}

lazy_static! {
    pub static ref NAME_PROP_CONFIGS: [NamePropConfig; NUM_NAME_PROPS] = unsafe {
        let mut configs: [NamePropConfig; NUM_NAME_PROPS] = std::mem::zeroed();
        NamePropConfig::read_from_data(&mut configs);
        configs
    };
}
