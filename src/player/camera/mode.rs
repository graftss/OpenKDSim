use serde::{Deserialize, Serialize};

use crate::macros::log;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum CameraMode {
    Normal,
    R1Jump,
    L1Look,
    HitByProp,
    Clear,
    Shoot,
    ShootRet,
    Ending1,
    Ending2,
    Ending3,
    AreaChange,
    ClearGoalProp,
    VsResult,
    Unknown(i32),
}

impl Default for CameraMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<i32> for CameraMode {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::R1Jump,
            2 => Self::L1Look,
            3 => Self::HitByProp,
            4 => Self::Clear,
            5 => Self::Shoot,
            6 => Self::ShootRet,
            7 => Self::Ending1,
            8 => Self::Ending2,
            9 => Self::Ending3,
            10 => Self::AreaChange,
            11 => Self::ClearGoalProp,
            12 => Self::VsResult,
            _ => {
                log!("encountered unknown `CameraMode` value: {}", value);
                Self::Unknown(value)
            }
        }
    }
}

impl Into<u8> for CameraMode {
    fn into(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::R1Jump => 1,
            Self::L1Look => 2,
            Self::HitByProp => 3,
            Self::Clear => 4,
            Self::Shoot => 5,
            Self::ShootRet => 6,
            Self::Ending1 => 7,
            Self::Ending2 => 8,
            Self::Ending3 => 9,
            Self::AreaChange => 10,
            Self::ClearGoalProp => 11,
            Self::VsResult => 12,
            Self::Unknown(value) => value as u8,
        }
    }
}
