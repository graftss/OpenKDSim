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

pub struct StageConfig {
    pub flip_params: StageFlipParams,
}

impl StageConfig {
    pub fn get(mission: Stage) -> &'static StageConfig {
        &STAGE_CONFIGS[mission as usize]
    }

    fn read_from_data(configs: &mut StageConfigTable) {
        Self::read_flip_params(configs);
    }

    fn read_flip_params(configs: &mut StageConfigTable) {
        for (stage_idx, config) in configs.iter_mut().enumerate() {
            let base = stage_idx * StageFlipParams::WIDTH;
            config.flip_params.min_flip_ticks = read_f32!(SC_FLIP_PARAMS_TABLE, base + 0x0);
            config.flip_params.min_diam_cm = read_f32!(SC_FLIP_PARAMS_TABLE, base + 0x4);
            config.flip_params.max_flip_ticks = read_f32!(SC_FLIP_PARAMS_TABLE, base + 0x8);
            config.flip_params.max_diam_cm = read_f32!(SC_FLIP_PARAMS_TABLE, base + 0xc);
        }
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
