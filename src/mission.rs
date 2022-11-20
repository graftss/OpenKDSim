use gl_matrix::common::Vec4;
use lazy_static::lazy_static;

use crate::{
    constants::{MAX_PLAYERS, NUM_MISSIONS},
    macros::{read_bool, read_f32, read_u16, read_u8},
    util::vec4_from_le_bytes,
};

static MC_0X60_TABLE: &'static [u8] = include_bytes!("data/mission_config_0x60_table.bin");

#[derive(Debug, Copy, Clone)]
pub enum GameType {
    /// Game type for "Make a Star" missions, where the objective is to reach a fixed diameter.
    ClearSize = 0,

    /// Game type for "Make Taurus/Ursa Major", where the objective is to collect one
    /// of some predetermined list of prop types.
    ClearProps = 1,

    /// Game type for constellations (e.g. "Make Cancer"), where the objective is to collect
    /// as many on-theme objects as possible.
    NumThemeProps = 2,

    /// Game type to collected a fixed *quantity* of objects.
    /// Uncompleted implementation, but there are references to it elsewhere.
    ClearNumProps = 3,

    /// Unused
    GameTypeD = 4,

    /// Game type for "Make the North Star".
    NorthStar = 5,

    /// Unused
    GameTypeF = 6,

    /// Unused
    GameTypeS = 7,

    /// Game type for "Eternal" missions.
    Eternal = 8,
}

impl TryFrom<u8> for GameType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::ClearSize),
            1 => Ok(Self::ClearProps),
            2 => Ok(Self::NumThemeProps),
            3 => Ok(Self::ClearNumProps),
            4 => Ok(Self::GameTypeD),
            5 => Ok(Self::NorthStar),
            6 => Ok(Self::GameTypeF),
            7 => Ok(Self::GameTypeS),
            8 => Ok(Self::Eternal),
            _ => panic!("unrecognized `GameType`"),
        }
    }
}

/// Game missions.
#[derive(Debug, Copy, Clone)]
pub enum Mission {
    None = 0,               // king talking???
    MAS1 = 1,               // MAS1
    MAS2 = 2,               // MAS2
    MAS4 = 3,               // MAS4
    MAS3 = 4,               // MAS3
    MAS5 = 5,               // MAS5
    MAS6 = 6,               // MAS6
    MAS7 = 7,               // MAS7
    MAS8 = 8,               // MAS8
    MAS9 = 9,               // MAS9
    MTM = 10,               // MTM
    Cancer = 11,            // cancer
    Cygnus = 12,            // cygnus
    Mission13 = 13,         // (unused) "50 object" debug level, broken
    Corona = 14,            // corona
    Pisces = 0xF,           // pisces
    Virgo = 0x10,           // virgo
    Ursa = 17,              // ursa major
    Gemini = 18,            // gemini
    Taurus = 19,            // taurus
    Mission20 = 20,         // (unused) mas7 area with no objects
    NorthStar = 21,         // north star
    Eternal1 = 22,          // eternal 1
    Eternal2 = 23,          // eternal 2
    Eternal3 = 24,          // eternal 3
    Mission25ShopDemo = 25, // (unused) debug l evel with starting size 0
    Mission26 = 26,         // (unused) debug level with no collision, spawn above pond in mas8
    Mission27 = 27,         // (unused) mas7 area with no objects
    Tutorial = 28,          // tutorial (opens with PRESS START)
    Ending = 29,            // countries level, gametype N
    Mission30Load = 30,     // nothing loads
    Vs0 = 0x1F,
    Vs1 = 0x20,
    Vs2 = 33,
    Vs3 = 34, // vs level with magazine bridge
    Vs4 = 35,
    Vs5 = 36,
    Vs6 = 37,
    Vs7 = 38,
    GameShow = 39, // nothing loads
    Test0 = 40,    // nothing loads
    Test1 = 41,    // nothing loads
    Test2 = 42,    // nothing loads
    Test3 = 43,    // nothing loads
    Test4 = 44,    // nothing loads
}

#[derive(Debug)]
pub enum GameMode {
    Normal = 0,
    Tutorial = 1,
    TutorialB = 2,
    Ending = 3,
    Load = 4,
}

impl TryFrom<i32> for GameMode {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::Tutorial),
            2 => Ok(Self::TutorialB),
            3 => Ok(Self::Ending),
            4 => Ok(Self::Load),
            _ => panic!("unrecognized gamemode"),
        }
    }
}

/// Constant features of each mission.
/// offset: 0x5f7a0
/// size: 0x60
pub struct MissionConfig {
    /// If true, props can be smaller without being destroyed at alpha 0.
    /// offset: 0x2
    pub keep_distant_props_alive: bool,

    /// The initial position of each katamari.
    /// offset: 0x8
    pub init_kat_pos: [Vec4; MAX_PLAYERS],

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

            config.keep_distant_props_alive = read_bool!(table, base + 0x2);

            for (i, init_pos) in config.init_kat_pos.iter_mut().enumerate() {
                vec4_from_le_bytes(init_pos, &table, base + 0x8 + i * 0x10);
            }

            for (i, angle) in config.init_prince_angle.iter_mut().enumerate() {
                *angle = read_f32!(table, base + 0x28 + i * 4);
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
