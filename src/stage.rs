use gl_matrix::common::Vec3;
use lazy_static::lazy_static;

use crate::{
    constants::NUM_STAGES,
    macros::{inv_lerp_clamp, lerp, panic_log, read_f32},
};

static SC_FLIP_PARAMS_TABLE: &'static [u8] = include_bytes!("data/stage_config_flip_params.bin");

/// A stage is a map.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Stage {
    House = 1,
    Town = 2,
    World = 3,
    Ending = 9,
    VsMode = 10,
    Tutorial = 12,
}

impl Into<u32> for Stage {
    fn into(self) -> u32 {
        match self {
            Stage::House => 1,
            Stage::Town => 2,
            Stage::World => 3,
            Stage::Ending => 9,
            Stage::VsMode => 10,
            Stage::Tutorial => 12,
        }
    }
}

impl From<u32> for Stage {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::House,
            2 => Self::Town,
            3 => Self::World,
            9 => Self::Ending,
            10 => Self::VsMode,
            12 => Self::Tutorial,
            _ => {
                panic_log!("encountered unknown `Stage` value: {}", value);
            }
        }
    }
}

/// Flip duration is computed by lerping the katamari's diameter between
/// stage-specific minimum and maximum diameters.
/// offset: 0x60300
#[derive(Debug)]
pub struct StageFlipParams {
    /// The minimum flip duration, which occurs when the katamari has diameter
    /// at most `min_diam_cm`.
    /// offset: 0x0
    pub min_flip_ticks: f32,

    /// The katamari diameter at which the min flip duration occurs.
    /// offset: 0x4
    pub min_diam_cm: f32,

    /// The maximum flip duration, which occurs when the katamari has diameter
    /// at least `max_diam_cm`.
    /// offset: 0x8
    pub max_flip_ticks: f32,

    /// The katamari diameter at which the max flip duration occurs.
    /// offset: 0xc
    pub max_diam_cm: f32,
}

impl StageFlipParams {
    /// The width of an entry in the `StageFlipParams` table.
    pub const WIDTH: usize = 0x10;

    /// Compute the flip duration from these params when the katamari has
    /// diameter `diam_cm`.
    pub fn get_duration(&self, diam_cm: f32) -> u32 {
        let t = inv_lerp_clamp!(diam_cm, self.min_diam_cm, self.max_diam_cm);

        // note that we're implicitly taking the floor here, by casting to `u32`
        lerp!(t, self.min_flip_ticks, self.max_flip_ticks) as u32
    }
}

lazy_static! {
    // initialize the royal warp destinations for the stages anyone cares about.
    // note that as is often the case with hardcoded static positions, the
    // katamari positions are *negative* what they are in the simulation.
    // (because the unity coordinate system is negative the simulation's
    // for some reason.)
    static ref STAGE_ROYAL_WARPS: [StageRoyalWarps; 3] = [
        // house royal warps
        StageRoyalWarps {
            area_dests: vec![
                RoyalWarpDest { prince_angle: 0.0, kat_pos: [0.0, 0.0, 0.0] },
                RoyalWarpDest { prince_angle: 0.0, kat_pos: [0.0, 0.0, 0.0] },
                RoyalWarpDest { prince_angle: 0.0, kat_pos: [-600.0, 0.0, 180.0] },
                RoyalWarpDest { prince_angle: 0.0, kat_pos: [-600.0, 0.0, 180.0] },
            ]
        },

        // town royal warps
        StageRoyalWarps {
            area_dests: vec![
                RoyalWarpDest { prince_angle: f32::from_bits(0x3f490fdb), kat_pos: [-11600.0, 1240.0, -980.0] },
                RoyalWarpDest { prince_angle: f32::from_bits(0x3fc90fdb), kat_pos: [-7350.0, 300.0, -1200.0] },
                RoyalWarpDest { prince_angle: f32::from_bits(0xbfc90fdb), kat_pos: [800.0, 1200.0, -750.0] },
                RoyalWarpDest { prince_angle: 0.0, kat_pos: [800.0, 1200.0, -750.0] },
            ]
        },

        // world royal warps
        StageRoyalWarps {
            area_dests: vec![
                RoyalWarpDest { prince_angle: f32::from_bits(0x3f91361e), kat_pos: [-500.0, 100.0, 4480.0] },
                RoyalWarpDest { prince_angle: f32::from_bits(0x3f91361e), kat_pos: [-500.0, 100.0, 4480.0] },
                RoyalWarpDest { prince_angle: f32::from_bits(0xbfa78d36), kat_pos: [-56000.0, 3500.0, 28000.0] },
                RoyalWarpDest { prince_angle: f32::from_bits(0xbfc90fdb), kat_pos: [60000.0, 30000.0, 0.0] },
                RoyalWarpDest { prince_angle: f32::from_bits(0xbfc90fdb), kat_pos: [60000.0, 30000.0, 0.0] },
            ]
        },

        // TODO: vs mode royal warps
    ];
}

/// The katamari position and prince facing angle resulting from a royal warp.
pub struct RoyalWarpDest {
    /// The katamari position after a royal warp.
    pub kat_pos: Vec3,

    /// The prince facing angle after a royal warp.
    pub prince_angle: f32,
}

/// All possible royal warp positions in a stage. The royal warp position
/// varies with the loaded area of the stage.
/// TODO: in vs mode, the two players have different royal warp destinations,
/// which can't be encoded in this structure.
pub struct StageRoyalWarps {
    pub area_dests: Vec<RoyalWarpDest>,
}

pub struct StageConfig {
    flip_params: StageFlipParams,
    royal_warps: Option<&'static StageRoyalWarps>,
}

impl StageConfig {
    /// Get this stage's flip duration as a function of the katamari's current diameter (in cm).
    pub fn get_flip_duration(&self, diam_cm: f32) -> u32 {
        self.flip_params.get_duration(diam_cm)
    }

    pub fn has_royal_warp_dests(&self) -> bool {
        self.royal_warps.is_some()
    }

    /// Get this stage's royal warp destination as a function of the currently loaded area.
    pub fn get_royal_warp_dest(&self, area: usize) -> Option<&RoyalWarpDest> {
        self.royal_warps
            .map(|warps| warps.area_dests.get(area))
            .flatten()
    }
}

impl StageConfig {
    pub fn get(stage: Stage) -> &'static StageConfig {
        &STAGE_CONFIGS[stage as usize]
    }

    /// Initialize the stage config table from static data.
    fn read_from_data(configs: &mut StageConfigTable) {
        Self::read_flip_params(configs);
    }

    /// Read the flip params file into the stage config table.
    fn read_flip_params(configs: &mut StageConfigTable) {
        for (stage_idx, config) in configs.iter_mut().enumerate() {
            let base = stage_idx * StageFlipParams::WIDTH;
            config.flip_params.min_flip_ticks = read_f32!(SC_FLIP_PARAMS_TABLE, base + 0x0);
            config.flip_params.min_diam_cm = read_f32!(SC_FLIP_PARAMS_TABLE, base + 0x4);
            config.flip_params.max_flip_ticks = read_f32!(SC_FLIP_PARAMS_TABLE, base + 0x8);
            config.flip_params.max_diam_cm = read_f32!(SC_FLIP_PARAMS_TABLE, base + 0xc);
        }
    }

    /// Read the handwritten royal warps table into the stage config table.
    fn read_royal_warps(configs: &mut StageConfigTable) {
        configs[1].royal_warps = Some(&STAGE_ROYAL_WARPS[0]);
        configs[2].royal_warps = Some(&STAGE_ROYAL_WARPS[1]);
        configs[3].royal_warps = Some(&STAGE_ROYAL_WARPS[2]);
    }
}

pub type StageConfigTable = [StageConfig; NUM_STAGES];

lazy_static! {
    pub static ref STAGE_CONFIGS: StageConfigTable = unsafe {
        let mut configs: [StageConfig; NUM_STAGES] = std::mem::zeroed();
        StageConfig::read_from_data(&mut configs);
        configs
    };
}
