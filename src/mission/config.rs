use std::{collections::HashMap, f32::consts::PI};

use gl_matrix::common::Vec3;
use lazy_static::lazy_static;

use crate::{
    constants::NUM_MISSIONS,
    macros::{panic_log, read_bool, read_f32, read_u16, read_u8, rescale},
    math::vec3_inplace_scale,
    mission::GameType,
    player::constants::MAX_PLAYERS,
    util::vec3_from_le_bytes,
};

use super::Mission;

static MC_0X60_TABLE: &'static [u8] = include_bytes!("bin/mission_config_0x60_table.bin");

/// Data controlling the mission-specific volume penalty to attached objects.
/// The penalty is a piecewise-linear function of the katamari's diameter.
/// This piecewise function is encoded as a sequence of control points, each
/// determining a certain volume penalty at a certain katamari diameter.
/// offset: 0x5fa20
#[derive(Debug)]
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
    pub fn from_floats(raw_data: &Vec<f32>) -> Vec<VolPenaltyCtrlPt> {
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
            panic_log!("Invalid data length passed to `VolPenaltyCtrlPt::from_floats`.");
        } else if raw_data[0] != 0.0 {
            // by forcing the first control point diameter to be 0.0, we can guarantee that
            // the katamari's diameter will never be smaller than all control points.
            panic_log!("Invalid data passed to `VolPenaltyCtrlPt::from_floats`: first element should be `0.0`.");
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
}

/// Constant, mission-specific data.
#[derive(Debug)]
pub struct MissionConfig {
    pub vol_penalty_ctrl_pts: Option<Vec<VolPenaltyCtrlPt>>,

    // mission config 0x60 table
    // offset: 0x5f7a0
    // size: 0x60
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
    pub theme_prop_names: Option<Box<Vec<u16>>>,

    /// The number of props to clear the mission in the `ClearNumProps` game type.
    /// offset: 0x58
    pub num_props_to_clear: u16,
}

impl MissionConfig {
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
}

/// Initialize the `MissionConfig` table `configs`.
fn read_from_data(configs: &mut [MissionConfig; NUM_MISSIONS]) {
    read_mission_config_0x60_table(configs);
    read_vol_penalty_ctrl_pts(configs);
}

/// Read the binary "mission config 0x60" table from the simulation into
/// the `MissionConfig` table `configs`.
fn read_mission_config_0x60_table(configs: &mut [MissionConfig; NUM_MISSIONS]) {
    let table = MC_0X60_TABLE;
    let WIDTH: usize = 0x60;

    for (mission_idx, config) in configs.iter_mut().enumerate() {
        let base = mission_idx * WIDTH;

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
            config.vol_penalty_ctrl_pts = Some(VolPenaltyCtrlPt::from_floats(raw_data));
        }
    }
}

lazy_static! {
    static ref MISSION_CONFIGS: [MissionConfig; NUM_MISSIONS] = unsafe {
        let mut configs: [MissionConfig; NUM_MISSIONS] = std::mem::zeroed();
        read_from_data(&mut configs);
        configs
    };
}
