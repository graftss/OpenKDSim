use std::f32::consts::PI;

use gl_matrix::common::Vec3;
use lazy_static::lazy_static;

use crate::{
    constants::NUM_MISSIONS,
    macros::{read_bool, read_f32, read_u16, read_u8},
    math::vec3_inplace_scale,
    mission::GameType,
    player::constants::MAX_PLAYERS,
    util::vec3_from_le_bytes,
};

use super::Mission;

static MC_0X60_TABLE: &'static [u8] = include_bytes!("bin/mission_config_0x60_table.bin");

/// Constant features of each mission.
/// offset: 0x5f7a0
/// size: 0x60
#[derive(Debug)]
pub struct MissionConfig {
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
    pub fn get(mission: Mission) -> &'static MissionConfig {
        &MISSION_CONFIGS[mission as usize]
    }

    pub fn read_from_data(configs: &mut [MissionConfig; NUM_MISSIONS]) {
        Self::read_mission_config_0x60_table(configs);
    }

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
}

lazy_static! {
    static ref MISSION_CONFIGS: [MissionConfig; NUM_MISSIONS] = unsafe {
        let mut configs: [MissionConfig; NUM_MISSIONS] = std::mem::zeroed();
        MissionConfig::read_from_data(&mut configs);
        configs
    };
}
