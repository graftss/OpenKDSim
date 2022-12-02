use std::collections::HashMap;

use gl_matrix::common::Vec3;
use lazy_static::lazy_static;

use crate::{
    constants::{NUM_MISSIONS, PI},
    macros::{panic_log, read_bool, read_f32, read_u16, read_u8, rescale, temp_debug_log},
    math::vec3_inplace_scale,
    mission::GameType,
    player::{
        camera::CameraState, constants::MAX_PLAYERS, katamari::scaled_params::KatScaledParams,
    },
    util::vec3_from_le_bytes,
};

use super::{stage::Stage, Mission};

static MC_0X60_TABLE: &'static [u8] = include_bytes!("bin/mission_config_0x60_table.bin");
static MC_SCALING_PARAMS_TABLE: &'static [u8] =
    include_bytes!("bin/mission_config_scaling_params.bin");
static MC_CAMERA_PARAMS_TABLE: &'static [u8] =
    include_bytes!("bin/mission_config_camera_params.bin");

/// Data controlling the mission-specific volume penalty to attached objects.
/// The penalty is a piecewise-linear function of the katamari's diameter.
/// This piecewise function is encoded as a sequence of control points, each
/// determining a certain volume penalty at a certain katamari diameter.
/// offset: 0x5fa20
#[derive(Debug, Default, Clone)]
pub struct VolPenaltyCtrlPt {
    /// The katamari diameter at which this penalty applies.
    /// offset: 0x0
    pub diam_cm: f32,

    /// The penalty applied at the above katamari diameter.
    /// offset: 0x4
    pub penalty: f32,
}

impl VolPenaltyCtrlPt {
    /// Convert a list of floats to a list of volume penalty control points.
    pub fn from_bytes(raw_data: &Vec<f32>) -> Vec<VolPenaltyCtrlPt> {
        let mut result = vec![];

        for chunk in raw_data.chunks(2) {
            result.push(VolPenaltyCtrlPt {
                diam_cm: chunk[0],
                penalty: chunk[1],
            });
        }

        result
    }

    /// Sanity-check the ray float array making up a mission's volume penalty control points.
    pub fn validate_mission_ctrl_pts(raw_data: &Vec<f32>) {
        if raw_data.len() % 2 != 0 {
            panic_log!("Invalid data length passed to `VolPenaltyCtrlPt::from_bytes`.");
        } else if raw_data[0] != 0.0 {
            // by forcing the first control point diameter to be 0.0, we can guarantee that
            // the katamari's diameter will never be smaller than all control points.
            panic_log!("Invalid data passed to `VolPenaltyCtrlPt::from_bytes`: first element should be `0.0`.");
        }
    }
}

lazy_static! {
    /// Handwritten mission-specific volume penalties because whatever it's too annoying to
    /// write a script to import the stupid pointer table in the mod.
    static ref MC_VOL_PENALTY_CTRL_PTS: HashMap<Mission, Vec<f32>> = {
        let mut result = HashMap::new();
        result.insert(Mission::MAS1, vec![0.0, 1.0, 10.0, 1.0, 15.0, 0.5, 20.0, 0.2, 30.0, 0.1]);
        result.insert(Mission::MAS2, vec![0.0, 1.0, 10.0, 1.0, 20.0, 0.9, 30.0, 0.5, 40.0, 0.1]);
        result.insert(Mission::MAS3, vec![0.0, 1.0, 50.0, 1.0, 75.0, 0.5, 100.0, 0.2, 160.0, 0.01]);
        result.insert(Mission::MAS4, vec![0.0, 1.0, 50.0, 1.0, 100.0, 0.6, 110.0, 0.4, 120.0, 0.25, 125.0, 0.15, 130.0, 0.1, 135.0, 0.0, 160.0, 0.0]);
        result.insert(Mission::MAS5, vec![0.0, 1.0, 150.0, 1.0, 200.0, 0.5, 250.0, 0.2, 300.0, 0.1, 500.0, 0.01]);
        result.insert(Mission::MAS6, vec![0.0, 1.0, 50.0, 1.0, 100.0, 1.0, 200.0, 0.9, 300.0, 0.9, 500.0, 0.7, 600.0, 0.7, 700.0, 0.6, 800.0, 0.1]);
        result.insert(Mission::MAS7, vec![0.0, 1.0, 50.0, 1.1, 100.0, 1.0, 300.0, 1.0, 500.0, 1.0, 600.0, 1.0, 700.0, 1.0, 800.0, 1.0, 900.0, 0.6, 1000.0, 0.2, 1100.0, 0.1]);
        result.insert(Mission::MAS8, vec![0.0, 1.0, 10.0, 1.1, 20.0, 1.2, 50.0, 1.0, 1000.0, 1.0, 1200.0, 0.8, 2000.0, 0.7, 2500.0, 0.6, 3000.0, 0.3]);
        result.insert(Mission::MAS9, vec![0.0, 1.0, 50.0, 1.0, 300.0, 1.2, 1000.0, 1.0, 2000.0, 0.6, 3000.0, 0.8, 4000.0, 0.8, 4500.0, 0.6, 500.0, 0.4, 5500.0, 0.2, 6000.0, 0.1]);
        result.insert(Mission::MTM,  vec![0.0, 1.0, 50.0, 1.0, 100.0, 1.0, 1000.0, 1.0, 10000.0, 1.1, 20000.0, 0.9, 30000.0, 0.9, 40000.0, 1.0, 50000.0, 1.0, 60000.0, 1.0, 70000.0, 0.8, 80000.0, 0.7, 90000.0, 0.5 ]);
        result.insert(Mission::Cancer, vec![0.0, 1.0, 10.0, 1.0, 30.0, 1.0, 50.0, 0.5, 60.0, 0.3, 65.0, 0.1, 70.0, 0.0]);
        result.insert(Mission::Cygnus, vec![0.0, 1.0, 13.0, 0.2, 30.0, 0.2, 50.0, 0.01, 100.0, 0.01, 160.0, 0.01]);
        result.insert(Mission::Corona, vec![0.0, 1.0, 40.0, 0.7, 70.0, 0.6, 100.0, 0.5, 150.0, 0.4, 200.0, 0.3, 500.0, 0.3]);
        result.insert(Mission::Pisces, vec![0.0, 1.0, 80.0, 0.9, 100.0, 0.8, 120.0, 0.7, 170.0, 0.6, 220.0, 0.5, 250.0, 0.3, 300.0, 0.2]);
        result.insert(Mission::Virgo,  vec![0.0, 1.0, 100.0, 0.95, 120.0, 0.85, 150.0, 0.6, 180.0, 0.5, 220.0, 0.4, 250.0, 0.3, 300.0, 0.2, 350.0, 0.1, 450.0, 0.05]);
        result.insert(Mission::Ursa,   vec![0.0, 1.0, 10.0, 1.0, 20.0, 1.0, 50.0, 1.0, 100.0, 1.0, 160.0, 1.0]);
        result.insert(Mission::Gemini, vec![0.0, 1.0, 10.0, 1.0, 20.0, 1.0, 50.0, 1.0, 100.0, 0.8, 150.0, 0.8, 200.0, 0.8, 300.0, 0.8, 400.0, 0.6, 500.0, 0.2]);
        result.insert(Mission::Taurus, vec![0.0, 1.0, 100.0, 1.0, 200.0, 1.0, 300.0, 1.0, 400.0, 1.0, 500.0, 1.0]);
        result.insert(Mission::NorthStar, vec![0.0, 1.0, 1000.0, 0.8, 1150.0, 0.5, 1300.0, 0.1]);
        result.insert(Mission::Eternal1, vec![0.0, 1.0, 50.0, 1.0, 100.0, 0.6, 100.0, 0.4, 120.0, 0.25, 125.0, 0.15, 130.0, 0.1, 135.0, 0.0, 160.0, 0.0]);
        result.insert(Mission::Eternal2, vec![0.0, 1.0, 10.0, 1.1, 20.0, 1.2, 50.0, 1.0, 1000.0, 1.0, 1200.0, 0.8, 2000.0, 0.7, 2500.0, 0.5, 3000.0, 0.3]);
        result.insert(Mission::Eternal3, vec![0.0, 1.0, 50.0, 1.0, 100.0, 1.0, 1000.0, 1.0, 10000.0, 1.1, 20000.0, 0.9, 30000.0, 0.9, 40000.0, 1.0, 50000.0, 1.0, 60000.0, 0.9, 70000.0, 0.7, 80000.0, 0.4, 90000.0, 0.05]);
        result
    };

    /// Handwritten mission-specific max scaled sizes.
    /// When the katamari grows above a mission's max scaled size, its scaled params stop
    /// changing: all sizes larger than this size yield the same scaled params.
    static ref MC_MAX_SCALED_SIZE: Vec<f32> = vec![
        100.0, 14.0, 30.0, 85.0, 50.0, 150.0, 250.0, 600.0, 1500.0, 5000.0, 100000.0, 80.0, 100.0, 30.0, 1000.0, 1000.0, 1000.0, 250.0, 30000.0, 30000.0, 200.0, 1000.0, 100.0, 1500.0, 100000.0, 230.0, 100.0, 100.0, 100.0, 500.0, 100.0, 500.0, 100.0,
    ];
}

/// Each mission has a list of control points which give the katamari's `KatScaledParams`
/// values at a specific size. During the mission, the katamari's scaled params are computed
/// by linearly interpolating between the `KatScaledCtrlPt` values with sizes just smaller
/// and just bigger than the katamari's actual size.
/// offset: 0x60180 (mission-indexed table of pointers to variable-length control point lists)
#[derive(Debug, Default, Clone)]
pub struct KatScaledParamsCtrlPt {
    /// The diameter at which the given params apply.
    /// offset: 0x0
    pub diam_cm: f32,

    /// The params that apply at the given diameter.
    /// offset: 0x4
    pub params: KatScaledParams,
}

impl KatScaledParamsCtrlPt {
    const WIDTH: usize = 0x54;

    /// Read the `KatScaledParamsCtrlPt` values for each mission from the
    /// ragged table of control points extracted from the simulation.
    /// The end of a mission's control point list is detected from a control
    /// point's size being `-1.0`.
    fn from_bytes(raw_data: &[u8]) -> Vec<Vec<KatScaledParamsCtrlPt>> {
        let mut result = vec![];
        let mut mission_ctrl_pts = vec![];

        for chunk in raw_data.chunks(Self::WIDTH) {
            let diam_cm = read_f32!(chunk, 0);

            if diam_cm < 0.0 {
                result.push(mission_ctrl_pts);
                mission_ctrl_pts = vec![];
            } else {
                mission_ctrl_pts.push(KatScaledParamsCtrlPt {
                    diam_cm,
                    params: KatScaledParams {
                        base_max_speed: read_f32!(chunk, 0x4),
                        accel_grav: read_f32!(chunk, 0x8),
                        brake_forwards_force: read_f32!(chunk, 0xc),
                        brake_backwards_force: read_f32!(chunk, 0x10),
                        brake_sideways_force: read_f32!(chunk, 0x14),
                        brake_boost_force: read_f32!(chunk, 0x18),
                        max_forwards_speed: read_f32!(chunk, 0x1c),
                        max_backwards_speed: read_f32!(chunk, 0x20),
                        max_sideways_speed: read_f32!(chunk, 0x24),
                        max_boost_speed: read_f32!(chunk, 0x28),
                        push_uphill_accel: read_f32!(chunk, 0x2c),
                        not_push_uphill_accel: read_f32!(chunk, 0x30),
                        forwards_accel: read_f32!(chunk, 0x34),
                        backwards_accel: read_f32!(chunk, 0x38),
                        sideways_accel: read_f32!(chunk, 0x3c),
                        boost_accel: read_f32!(chunk, 0x40),
                        prince_offset: read_f32!(chunk, 0x44),
                        alarm_closest_range: read_f32!(chunk, 0x48),
                        alarm_closer_range: read_f32!(chunk, 0x4c),
                        alarm_close_range: read_f32!(chunk, 0x50),
                    },
                })
            }
        }

        result
    }
}

/// A control point that determines how the camera should be positioned at a specific
/// katamari size. The actual position is determined by lerping the values of the
/// two control points on either side of the katamari's actual size.
#[derive(Debug, Default, Clone, Copy)]
pub struct CamScaledCtrlPt {
    /// The minimum katamari diameter at which this control point takes effect.
    pub diam_cm: f32,

    /// The control point's camera position (relative to katamari center).
    pub kat_to_pos: Vec3,

    /// The control point's camera target (relative to katamari center).
    pub kat_to_target: Vec3,

    /// The max height that the prince reaches after an R1 jump.
    pub jump_r1_height: f32,
}

impl CamScaledCtrlPt {
    const WIDTH: usize = 0x28;

    /// Read the `KatScaledParamsCtrlPt` values for each mission from the
    /// ragged table of control points extracted from the simulation.
    /// The end of a mission's control point list is detected from a control
    /// point's size being `-1.0`.
    fn from_bytes(raw_data: &[u8]) -> Vec<Vec<CamScaledCtrlPt>> {
        let mut result = vec![];
        let mut mission_ctrl_pts = vec![];

        for chunk in raw_data.chunks(Self::WIDTH) {
            let diam_cm = read_f32!(chunk, 0);

            if diam_cm < 0.0 {
                result.push(mission_ctrl_pts);
                mission_ctrl_pts = vec![];
            } else {
                mission_ctrl_pts.push(CamScaledCtrlPt {
                    diam_cm,
                    kat_to_pos: [
                        read_f32!(chunk, 0x4),
                        read_f32!(chunk, 0x8),
                        read_f32!(chunk, 0xc),
                    ],
                    kat_to_target: [
                        read_f32!(chunk, 0x14),
                        read_f32!(chunk, 0x18),
                        read_f32!(chunk, 0x1c),
                    ],
                    jump_r1_height: read_f32!(chunk, 0x24),
                });
            }
        }

        result
    }
}

/// Constant, mission-specific data.
#[derive(Debug, Default, Clone)]
pub struct MissionConfig {
    /// List of control points describing how the katamari's `attach_vol_penalty`
    /// changes as the katamari grows in size in the mission.
    pub vol_penalty_ctrl_pts: Option<Vec<VolPenaltyCtrlPt>>,

    /// List of control points describing how the katamari's `KatScaledParams`
    /// change as the katamari grows in size.
    pub scaled_params_ctrl_pts: Option<Vec<KatScaledParamsCtrlPt>>,

    /// The katamari size at which scaled params stop growing.    
    pub scaled_params_max_size: f32,

    /// List of control points describing how the camera should be positioned
    /// and oriented as the katamari grows in size.
    pub camera_params_ctrl_pts: Option<Vec<CamScaledCtrlPt>>,

    // mission config 0x60 table
    // offset: 0x5f7a0
    // size: 0x60
    /// The stage (i.e. the map) in which the mission takes place.
    /// offset: 0x0
    pub stage: Stage,

    /// If true, props can be smaller without being destroyed at alpha 0.
    /// offset: 0x2
    pub keep_smaller_props_alive: bool,

    /// The initial position of each katamari.
    /// offset: 0x8
    pub init_kat_pos: [Vec3; MAX_PLAYERS],

    /// The initial facing angle of each prince.
    /// offset: 0x28
    pub init_prince_angle: [f32; MAX_PLAYERS],

    /// The initial katamari diameter.
    /// offset: 0x30
    pub init_diam_cm: f32,

    /// The goal katamari diameter (which may not be applicable).
    /// offset: 0x34
    pub goal_diam_cm: f32,

    /// The clear objective of the mission, encoded as a `GameType`.
    /// offset: 0x40
    pub game_type: GameType,

    /// A list of name indices corresponding to the "theme objects" of the
    /// mission, if the mission is the `NumThemeProps` gametype.
    /// offset: 0x48
    pub theme_prop_names: Option<Vec<u16>>,

    /// The number of props to clear the mission in the `ClearNumProps` game type.
    /// offset: 0x58
    pub num_props_to_clear: u16,
}

impl MissionConfig {
    pub fn get(out: &mut MissionConfig, mission_idx: u8) {
        out.clone_from(MISSION_CONFIGS.get(mission_idx as usize).unwrap());
    }

    /// Compute the mission's volume penalty for a katamari of diameter `diam_cm`.
    /// offset: 0x1bd50 (lines 349-375)
    pub fn get_vol_penalty(&self, diam_cm: f32) -> f32 {
        if let Some(ctrl_pts) = &self.vol_penalty_ctrl_pts {
            for i in 0..ctrl_pts.len() - 1 {
                let next = &ctrl_pts[i + 1];
                if next.diam_cm > diam_cm {
                    // `next` is the control point just after the current diameter.
                    // `last` is the contrl point just before the current diameter.
                    let last = &ctrl_pts[i];

                    return rescale!(
                        diam_cm,
                        last.diam_cm,
                        next.diam_cm,
                        last.penalty,
                        next.penalty
                    );
                }
            }

            // if the katamari is bigger than the last control point's diameter,
            // just use the last control point's penalty.
            ctrl_pts.last().unwrap().penalty;
        }

        // use a penalty of 1.0 if the mission doesn't have any control points
        1.0
    }

    /// Writes the scaled params of a katamari with diameter `diam_cm` to `params`.
    pub fn get_kat_scaled_params(&self, params: &mut KatScaledParams, mut diam_cm: f32) {
        if let Some(ctrl_pts) = &self.scaled_params_ctrl_pts {
            // for the purposes of scaled params, cap the katamari's diameter at the mission's
            // max size.
            if diam_cm > self.scaled_params_max_size {
                diam_cm = self.scaled_params_max_size;
            }

            for (i, ctrl_pt) in ctrl_pts.iter().enumerate() {
                if ctrl_pt.diam_cm > diam_cm {
                    // if we find a control point for a size bigger than the katamari's size,
                    // take the previous control point as the point just smaller than the katamari.
                    // (if the first point was already to smaller, just use it as both min and
                    //  max. the interpolation will just copy the params of the first point.)
                    let (min_pt, max_pt) = match i {
                        0 => (ctrl_pt, ctrl_pt),
                        _ => (&ctrl_pts[i - 1], ctrl_pt),
                    };

                    params.interpolate_from(diam_cm, min_pt, max_pt);
                    break;
                }
            }
        }
    }

    pub fn get_camera_ctrl_point(&self, camera_state: &mut CameraState, diam_cm: f32) {
        if let Some(ctrl_pts) = &self.camera_params_ctrl_pts {
            let mut used_idx = 0;
            for (i, ctrl_pt) in ctrl_pts.iter().enumerate() {
                if diam_cm <= ctrl_pt.diam_cm {
                    // find the first control point with a diameter larger than `diam_cm`, then
                    // use the control point before that.
                    used_idx = i - 1;
                    break;
                }
            }

            // write the selected control point's offsets to the camera state
            camera_state.kat_offset_ctrl_pt_idx = used_idx as u8;
            camera_state.set_kat_offsets(&ctrl_pts[used_idx]);
        }
    }
}

/// Initialize the `MissionConfig` table `configs`.
fn read_from_data(configs: &mut [MissionConfig; NUM_MISSIONS]) {
    read_mission_config_0x60_table(configs);
    read_vol_penalty_ctrl_pts(configs);
    read_scaled_params_ctrl_pts(configs);
    read_scaled_max_sizes(configs);
    read_camera_params_ctrl_pts(configs);
}

/// Read the binary "mission config 0x60" table from the simulation into
/// the `MissionConfig` table `configs`.
fn read_mission_config_0x60_table(configs: &mut [MissionConfig; NUM_MISSIONS]) {
    let table = MC_0X60_TABLE;
    let WIDTH: usize = 0x60;

    for (mission_idx, config) in configs.iter_mut().enumerate() {
        let base = mission_idx * WIDTH;

        config.stage = read_u8!(table, base).into();
        config.keep_smaller_props_alive = read_bool!(table, base + 0x2);

        for (i, init_pos) in config.init_kat_pos.iter_mut().enumerate() {
            vec3_from_le_bytes(init_pos, &table, base + 0x8 + i * 0x10);

            // The simulation positions are negative of what Unity expects, so we negate them
            // in advance here.
            vec3_inplace_scale(init_pos, -1.0);
        }

        for (i, angle) in config.init_prince_angle.iter_mut().enumerate() {
            *angle = read_f32!(table, base + 0x28 + i * 4);

            // Again, the simulation are negative of what Unity expects, so negate the angle here.
            *angle += PI;
        }

        config.init_diam_cm = read_f32!(table, base + 0x30);
        config.goal_diam_cm = read_f32!(table, base + 0x34);
        config.game_type = read_u8!(table, base + 0x40).try_into().unwrap();
        config.num_props_to_clear = read_u16!(table, base + 0x58);
    }
}

/// Read the handwritten volume penalty control point table into the
/// `MissionConfig` table `configs`.
fn read_vol_penalty_ctrl_pts(configs: &mut [MissionConfig; NUM_MISSIONS]) {
    for (mission_idx, config) in configs.iter_mut().enumerate() {
        if let Some(raw_data) = MC_VOL_PENALTY_CTRL_PTS.get(&(mission_idx as u8).into()) {
            VolPenaltyCtrlPt::validate_mission_ctrl_pts(raw_data);
            config.vol_penalty_ctrl_pts = Some(VolPenaltyCtrlPt::from_bytes(raw_data));
        }
    }
}

fn read_scaled_params_ctrl_pts(configs: &mut [MissionConfig; NUM_MISSIONS]) {
    let parsed_scaled_params = KatScaledParamsCtrlPt::from_bytes(MC_SCALING_PARAMS_TABLE);

    // println!("{:?}", parsed_scaled_params);
    for (config, params) in configs.iter_mut().zip(parsed_scaled_params) {
        config.scaled_params_ctrl_pts = Some(params);
    }
}

fn read_scaled_max_sizes(configs: &mut [MissionConfig; NUM_MISSIONS]) {
    for (config, max_size) in configs.iter_mut().zip(MC_MAX_SCALED_SIZE.iter()) {
        config.scaled_params_max_size = *max_size;
    }
}

fn read_camera_params_ctrl_pts(configs: &mut [MissionConfig; NUM_MISSIONS]) {
    let parsed_scaled_params = CamScaledCtrlPt::from_bytes(MC_CAMERA_PARAMS_TABLE);

    // println!("{:?}", parsed_scaled_params);
    for (config, params) in configs.iter_mut().zip(parsed_scaled_params) {
        config.camera_params_ctrl_pts = Some(params);
    }
}

lazy_static! {
    static ref MISSION_CONFIGS: [MissionConfig; NUM_MISSIONS] = unsafe {
        let mut configs: [MissionConfig; NUM_MISSIONS] = std::mem::zeroed();
        read_from_data(&mut configs);
        configs
    };
}
