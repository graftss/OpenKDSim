use serde::{Serialize, Deserialize};

use crate::macros::panic_log;

pub mod config;
pub mod ending;
pub mod stage;
pub mod state;
pub mod tutorial;
pub mod vsmode;

/// The "game type" of a mission encodes its objective (e.g. reaching a fixed
/// clear size for Make a Star levels).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl Default for GameType {
    fn default() -> Self {
        Self::ClearSize
    }
}

impl From<u8> for GameType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::ClearSize,
            1 => Self::ClearProps,
            2 => Self::NumThemeProps,
            3 => Self::ClearNumProps,
            4 => Self::GameTypeD,
            5 => Self::NorthStar,
            6 => Self::GameTypeF,
            7 => Self::GameTypeS,
            8 => Self::Eternal,
            _ => {
                panic_log!("encountered unknown `GameType` value: {}", value);
            }
        }
    }
}

/// Game missions.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    Pisces = 15,            // pisces
    Virgo = 16,             // virgo
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
    Vs0 = 31,
    Vs1 = 32,
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

impl Mission {
    /// Returns `true` if and only if this mission is a VS mode mission.
    pub fn is_vs_mode(&self) -> bool {
        match self {
            Self::Vs0
            | Self::Vs1
            | Self::Vs2
            | Self::Vs3
            | Self::Vs4
            | Self::Vs5
            | Self::Vs6
            | Self::Vs7 => true,
            _ => false,
        }
    }

    /// Returns the VS mode index of this mission if it has one.
    pub fn vs_mission_idx(&self) -> Option<u8> {
        match self {
            Self::Vs0 => Some(0),
            Self::Vs1 => Some(1),
            Self::Vs2 => Some(2),
            Self::Vs3 => Some(3),
            Self::Vs4 => Some(4),
            Self::Vs5 => Some(5),
            Self::Vs6 => Some(6),
            Self::Vs7 => Some(7),
            _ => None,
        }
    }
}

impl Default for Mission {
    fn default() -> Self {
        Self::None
    }
}

impl From<u8> for Mission {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,               // king talking???
            1 => Self::MAS1,               // MAS1
            2 => Self::MAS2,               // MAS2
            3 => Self::MAS4,               // MAS4
            4 => Self::MAS3,               // MAS3
            5 => Self::MAS5,               // MAS5
            6 => Self::MAS6,               // MAS6
            7 => Self::MAS7,               // MAS7
            8 => Self::MAS8,               // MAS8
            9 => Self::MAS9,               // MAS9
            10 => Self::MTM,               // MTM
            11 => Self::Cancer,            // cancer
            12 => Self::Cygnus,            // cygnus
            13 => Self::Mission13,         // (unused) "50 object" debug level, broken
            14 => Self::Corona,            // corona
            15 => Self::Pisces,            // pisces
            16 => Self::Virgo,             // virgo
            17 => Self::Ursa,              // ursa major
            18 => Self::Gemini,            // gemini
            19 => Self::Taurus,            // taurus
            20 => Self::Mission20,         // (unused) mas7 area with no objects
            21 => Self::NorthStar,         // north star
            22 => Self::Eternal1,          // eternal 1
            23 => Self::Eternal2,          // eternal 2
            24 => Self::Eternal3,          // eternal 3
            25 => Self::Mission25ShopDemo, // (unused) debug l evel with starting size 0
            26 => Self::Mission26, // (unused) debug level with no collision, spawn above pond in mas8
            27 => Self::Mission27, // (unused) mas7 area with no objects
            28 => Self::Tutorial,  // tutorial (opens with PRESS START)
            29 => Self::Ending,    // countries level, gametype N
            30 => Self::Mission30Load, // nothing loads
            31 => Self::Vs0,
            32 => Self::Vs1,
            33 => Self::Vs2,
            34 => Self::Vs3, // vs level with magazine bridge
            35 => Self::Vs4,
            36 => Self::Vs5,
            37 => Self::Vs6,
            38 => Self::Vs7,
            39 => Self::GameShow, // nothing loads
            40 => Self::Test0,    // nothing loads
            41 => Self::Test1,    // nothing loads
            42 => Self::Test2,    // nothing loads
            43 => Self::Test3,    // nothing loads
            44 => Self::Test4,    // nothing loads
            _ => {
                panic_log!("encountered unknown `Mission` value: {}", value);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Normal = 0,
    Tutorial = 1,
    TutorialB = 2,
    Ending = 3,
    Load = 4,
}

impl Default for GameMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<u8> for GameMode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::Tutorial,
            2 => Self::TutorialB,
            3 => Self::Ending,
            4 => Self::Load,
            _ => {
                panic_log!("encountered unknown `GameMode` value: {}", value);
            }
        }
    }
}

impl GameMode {
    pub fn can_update_view_mode(&self) -> bool {
        match self {
            GameMode::Normal | GameMode::Tutorial | GameMode::TutorialB => true,
            GameMode::Ending | GameMode::Load => false,
        }
    }
}
