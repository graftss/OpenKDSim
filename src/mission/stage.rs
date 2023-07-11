use crate::macros::{panic_log, rescale};

/// A stage is a map (notably: House, Town, World).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Stage {
    House,
    Town,
    World,
    Ending,
    VsMode,
    Tutorial,
    Unknown(u8),
}

impl Default for Stage {
    fn default() -> Self {
        Self::House
    }
}

impl Into<u8> for Stage {
    fn into(self) -> u8 {
        match self {
            Stage::House => 1,
            Stage::Town => 2,
            Stage::World => 3,
            Stage::Ending => 9,
            Stage::VsMode => 10,
            Stage::Tutorial => 12,
            Self::Unknown(x) => x,
        }
    }
}

impl From<u8> for Stage {
    fn from(value: u8) -> Self {
        match value {
            // Some entries in the `MissionConfig` table created by `read_mission_config_0x60_table`
            // have their `stage` field set to 0. This only occurs for a table entry that's unused.
            0 => Self::Unknown(0),
            1 => Self::House,
            2 => Self::Town,
            3 => Self::World,
            9 => Self::Ending,
            10 => Self::VsMode,
            12 => Self::Tutorial,
            _ => {
                panic_log!("Encountered unknown stage: {}", value);
            }
        }
    }
}
use crate::{
    constants::NUM_STAGES,
    macros::{inv_lerp_clamp, lerp, read_f32},
};
use gl_matrix::common::Vec3;
use lazy_static::lazy_static;

static SC_FLIP_PARAMS_TABLE: &'static [u8] = include_bytes!("bin/stage_config_flip_params.bin");
static SC_ELASTICITY_TABLE: &'static [u8] = include_bytes!("bin/stage_config_elasticity.bin");
static SC_AIRBORNE_PROP_GRAVITY_TABLE: &'static [u8] =
    include_bytes!("bin/stage_config_airborne_prop_gravity.bin");

/// Flip duration is computed by lerping the katamari's diameter between
/// stage-specific minimum and maximum diameters.
/// offset: 0x60300
#[derive(Debug, Default, Clone)]
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

        // TODO_VS: vs mode royal warps
    ];
}

/// The katamari position and prince facing angle resulting from a royal warp.
#[derive(Debug)]
pub struct RoyalWarpDest {
    /// The katamari position after a royal warp.
    pub kat_pos: Vec3,

    /// The prince facing angle after a royal warp.
    pub prince_angle: f32,
}

/// All possible royal warp positions in a stage. The royal warp position
/// varies with the loaded area of the stage.
/// TODO_VS: in vs mode, the two players have different royal warp destinations,
/// which can't be encoded in this structure.
#[derive(Debug)]
pub struct StageRoyalWarps {
    pub area_dests: Vec<RoyalWarpDest>,
}

/// Katamari "bounciness" (which I'm calling "elasticity" for grownup reasons)
/// is a linear function of diameter, with a different linear function defined
/// for each stage.
/// This is one of the tricks they use to make the katamari seem "weighty"
/// when you're big in Town and especially in World.
#[derive(Debug, Default, Clone)]
pub struct StageKatElasticity {
    /// The minimum diameter where the increasing elasticity scaling starts.
    /// offset: 0x0
    pub min_diam_cm: f32,

    /// The minimum diameter where the increasing elasticity scaling starts.
    /// offset: 0x4
    pub max_diam_cm: f32,

    /// The elasticity at the minimum diameter.
    /// offset: 0x8
    pub min_diam_elasticity: f32,

    /// The elasticity at the maximum diameter.
    /// offset: 0xc
    pub max_diam_elasticity: f32,
}

impl StageKatElasticity {
    /// The width of one struct in the binary table file.
    pub const WIDTH: usize = 0x10;

    fn get_kat_elasticity(&self, diam_cm: f32) -> f32 {
        rescale!(
            diam_cm,
            self.min_diam_cm,
            self.max_diam_cm,
            self.min_diam_elasticity,
            self.max_diam_elasticity
        )
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AirbornePropGravityCtrlPt {
    /// The katamari diameter at which this prop gravity is applied.
    /// offset: 0x0
    pub diam_cm: f32,

    /// The prop gravity applied at the given katamari diameter.
    pub gravity: f32,
}

/// The gravity imparted on airborne props is a piecewise linear function of the
/// katamari's diameter. The particular piecewise linear function is encoded as a
/// list of `StageAirbornePropGravity` control points, which varies with the stage.
#[derive(Debug, Default, Clone)]
pub struct StageAirbornePropGravity {
    pub ctrl_pts: Vec<AirbornePropGravityCtrlPt>,
}

impl StageAirbornePropGravity {
    pub const WIDTH: usize = 0x8;

    pub fn get_airborne_prop_gravity(&self, diam_cm: f32) -> f32 {
        for (i, ctrl_pt) in self.ctrl_pts.iter().enumerate() {
            if diam_cm <= ctrl_pt.diam_cm {
                // find the first control point with a diameter larger than `diam_cm`.
                // Then compute the result by interpolating `diam_cm` between this control
                // point's diameter (which is just bigger than `diam_cm`) and the previous
                // control point's diameter (which is just smaller than `diam_cm`).
                if i == 0 {
                    return ctrl_pt.gravity;
                } else {
                    let last_pt = &self.ctrl_pts[i - 1];
                    return rescale!(
                        diam_cm,
                        last_pt.diam_cm,
                        ctrl_pt.diam_cm,
                        last_pt.gravity,
                        ctrl_pt.gravity
                    );
                }
            }
        }

        1.0
    }

    pub fn from_bytes(raw_data: &[u8]) -> Vec<StageAirbornePropGravity> {
        let mut result = vec![];
        let mut mission_ctrl_pts = vec![];

        for chunk in raw_data.chunks(Self::WIDTH) {
            let diam_cm = read_f32!(chunk, 0);

            if diam_cm < 0.0 {
                result.push(StageAirbornePropGravity {
                    ctrl_pts: mission_ctrl_pts,
                });
                mission_ctrl_pts = vec![];
            } else {
                mission_ctrl_pts.push(AirbornePropGravityCtrlPt {
                    diam_cm,
                    gravity: read_f32!(chunk, 4),
                });
            }
        }

        result
    }
}

#[derive(Debug, Default, Clone)]
pub struct StageSizeSoundIds {
    pub cutoff_diam_mm: u32,
    pub collect_prop: u16,
    pub lose_prop: u16,
    pub wall_bonk: u16,
}

#[derive(Debug, Default, Clone)]
pub struct StageSoundIds {
    pub ctrl_pts: Vec<StageSizeSoundIds>,
}

impl StageSoundIds {
    fn find_size_idx(&self, kat_diam_mm: u32) -> Option<usize> {
        for (idx, ctrl_pt) in self.ctrl_pts.iter().enumerate() {
            if kat_diam_mm > ctrl_pt.cutoff_diam_mm {
                return Some(idx);
            }
        }

        None
    }

    fn get_collect_prop_sound_id(&self, kat_diam_mm: u32) -> u16 {
        // fall back to small sounds if all else fails
        static DEFAULT_RESULT: u16 = 4;

        self.find_size_idx(kat_diam_mm)
            .map_or(DEFAULT_RESULT, |idx| self.ctrl_pts[idx].collect_prop)
    }

    fn get_lose_prop_sound_id(&self, kat_diam_mm: u32) -> u16 {
        // fall back to small sounds if all else fails
        static DEFAULT_RESULT: u16 = 16;

        self.find_size_idx(kat_diam_mm)
            .map_or(DEFAULT_RESULT, |idx| self.ctrl_pts[idx].lose_prop)
    }

    fn get_wall_bonk_sound_id(&self, kat_diam_mm: u32) -> u16 {
        // fall back to small sounds if all else fails
        static DEFAULT_RESULT: u16 = 1;

        self.find_size_idx(kat_diam_mm)
            .map_or(DEFAULT_RESULT, |idx| self.ctrl_pts[idx].wall_bonk)
    }
}

lazy_static! {
    static ref STAGE_SOUND_IDS: [StageSoundIds; 3] = [
        // house sound ids
        StageSoundIds {
            ctrl_pts: vec![
                StageSizeSoundIds { cutoff_diam_mm: 500, collect_prop: 7, lose_prop: 18, wall_bonk: 23 },
                StageSizeSoundIds { cutoff_diam_mm: 200, collect_prop: 4, lose_prop: 17, wall_bonk: 22 },
                StageSizeSoundIds { cutoff_diam_mm: 0, collect_prop: 1, lose_prop: 16, wall_bonk: 21 },
            ]
        },

        // town sound ids
        StageSoundIds {
            ctrl_pts: vec![
                StageSizeSoundIds { cutoff_diam_mm: 6000, collect_prop: 10, lose_prop: 19, wall_bonk: 24 },
                StageSizeSoundIds { cutoff_diam_mm: 1500, collect_prop: 7, lose_prop: 18, wall_bonk: 23 },
                StageSizeSoundIds { cutoff_diam_mm: 450, collect_prop: 4, lose_prop: 17, wall_bonk: 22 },
                StageSizeSoundIds { cutoff_diam_mm: 0, collect_prop: 1, lose_prop: 16, wall_bonk: 21 },
            ]
        },

        // world sound ids
        StageSoundIds {
            ctrl_pts: vec![
                StageSizeSoundIds { cutoff_diam_mm: 61000, collect_prop: 13, lose_prop: 20, wall_bonk: 25 },
                StageSizeSoundIds { cutoff_diam_mm: 12000, collect_prop: 10, lose_prop: 19, wall_bonk: 24 },
                StageSizeSoundIds { cutoff_diam_mm: 3000, collect_prop: 7, lose_prop: 18, wall_bonk: 23 },
                StageSizeSoundIds { cutoff_diam_mm: 0, collect_prop: 4, lose_prop: 17, wall_bonk: 22 },
            ]
        },

        // TODO_VS: vs mode royal warps
    ];
}

#[derive(Debug, Default, Clone)]
pub struct StageConfig {
    stage_idx: u8,
    flip_params: Option<StageFlipParams>,
    royal_warps: Option<&'static StageRoyalWarps>,
    elasticity: Option<StageKatElasticity>,
    airborne_prop_gravity: Option<StageAirbornePropGravity>,
    sound_ids: Option<&'static StageSoundIds>,
}

impl StageConfig {
    /// Get this stage's flip duration as a function of the katamari's current diameter (in cm).
    pub fn get_flip_duration(&self, diam_cm: f32) -> u32 {
        self.flip_params
            .as_ref()
            .unwrap_or_else(|| {
                panic_log!("error reading stage flip params");
            })
            .get_duration(diam_cm)
    }

    /// Returns `true` if this stage has royal warp destinations defined.
    pub fn has_royal_warp_dests(&self) -> bool {
        self.royal_warps.is_some()
    }

    /// Get this stage's royal warp destination as a function of the currently loaded area.
    pub fn get_royal_warp_dest(&self, area: usize) -> Option<&RoyalWarpDest> {
        self.royal_warps
            .map(|warps| warps.area_dests.get(area))
            .flatten()
    }

    /// Compute the elasticity of a katamari with diameter `diam_cm` in this stage.
    pub fn get_kat_elasticity(&self, diam_cm: f32) -> f32 {
        self.elasticity
            .as_ref()
            .unwrap_or_else(|| {
                panic_log!("error reading stage elasticity: stage {}", self.stage_idx);
            })
            .get_kat_elasticity(diam_cm)
    }

    pub fn get_airborne_prop_gravity(&self, diam_cm: f32) -> f32 {
        self.airborne_prop_gravity
            .as_ref()
            .unwrap_or_else(|| {
                panic_log!(
                    "error reading airborne prop gravity: stage {}",
                    self.stage_idx
                );
            })
            .get_airborne_prop_gravity(diam_cm)
    }

    pub fn get_base_collect_object_sound_id(&self, kat_diam_mm: u32) -> u16 {
        self.sound_ids
            .unwrap_or_else(|| {
                panic_log!(
                    "error reading base collect object sound id: stage {}",
                    self.stage_idx
                );
            })
            .get_collect_prop_sound_id(kat_diam_mm)
    }

    pub fn get_lose_prop_sound_id(&self, kat_diam_mm: u32) -> u16 {
        self.sound_ids
            .unwrap_or_else(|| {
                panic_log!(
                    "error reading base collect object sound id: stage {}",
                    self.stage_idx
                );
            })
            .get_lose_prop_sound_id(kat_diam_mm)
    }

    pub fn get_wall_bonk_sound_id(&self, kat_diam_mm: u32) -> u16 {
        self.sound_ids
            .unwrap_or_else(|| {
                panic_log!(
                    "error reading base collect object sound id: stage {}",
                    self.stage_idx
                );
            })
            .get_wall_bonk_sound_id(kat_diam_mm)
    }
}

impl StageConfig {
    pub fn get(out: &mut StageConfig, stage_idx: u8) {
        out.clone_from(STAGE_CONFIGS.get(stage_idx as usize).unwrap());
    }

    /// Initialize the stage config table from static data.
    fn read_from_data(configs: &mut StageConfigTable) {
        Self::read_flip_params(configs);
        Self::read_royal_warps(configs);
        Self::read_elasticity(configs);
        Self::read_airborne_prop_gravity(configs);
        Self::read_sound_ids(configs);

        for (stage_idx, config) in configs.iter_mut().enumerate() {
            config.stage_idx = stage_idx as u8;
        }
    }

    /// Read the flip params file into the stage config table.
    fn read_flip_params(configs: &mut StageConfigTable) {
        for (stage_idx, config) in configs.iter_mut().enumerate() {
            let base = stage_idx * StageFlipParams::WIDTH;

            config.flip_params = Some(StageFlipParams {
                min_flip_ticks: read_f32!(SC_FLIP_PARAMS_TABLE, base + 0x0),
                min_diam_cm: read_f32!(SC_FLIP_PARAMS_TABLE, base + 0x4),
                max_flip_ticks: read_f32!(SC_FLIP_PARAMS_TABLE, base + 0x8),
                max_diam_cm: read_f32!(SC_FLIP_PARAMS_TABLE, base + 0xc),
            });
        }
    }

    /// Read the handwritten royal warps table into the stage config table.
    fn read_royal_warps(configs: &mut StageConfigTable) {
        configs[1].royal_warps = Some(&STAGE_ROYAL_WARPS[0]);
        configs[2].royal_warps = Some(&STAGE_ROYAL_WARPS[1]);
        configs[3].royal_warps = Some(&STAGE_ROYAL_WARPS[2]);
    }

    fn read_elasticity(configs: &mut StageConfigTable) {
        for (stage_idx, chunk) in SC_ELASTICITY_TABLE
            .chunks(StageKatElasticity::WIDTH)
            .enumerate()
        {
            configs[stage_idx].elasticity = Some(StageKatElasticity {
                min_diam_cm: read_f32!(chunk, 0x0),
                max_diam_cm: read_f32!(chunk, 0x4),
                min_diam_elasticity: read_f32!(chunk, 0x8),
                max_diam_elasticity: read_f32!(chunk, 0xc),
            });
        }
    }

    fn read_airborne_prop_gravity(configs: &mut StageConfigTable) {
        let table = StageAirbornePropGravity::from_bytes(SC_AIRBORNE_PROP_GRAVITY_TABLE);

        for (config, gravity) in configs.iter_mut().zip(table) {
            config.airborne_prop_gravity = Some(gravity);
        }
    }

    fn read_sound_ids(configs: &mut StageConfigTable) {
        configs[1].sound_ids = Some(&STAGE_SOUND_IDS[0]);
        configs[2].sound_ids = Some(&STAGE_SOUND_IDS[1]);
        configs[3].sound_ids = Some(&STAGE_SOUND_IDS[2]);
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
