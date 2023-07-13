use serde::{Serialize, Deserialize};

use crate::macros::panic_log;

/// Encodes nonstandard collision properties of surfaces. For example,
/// the `WallClimbDisabled` hit attribute on a surface stops the katamari
/// from climbing on it, and `SpeedCheckOff` is the hit attribute that the
/// big hill in the Town stage has to make the katamari move extra fast.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum HitAttribute {
    None,
    WaterSurface,
    BottomOfSea,
    NoReactionNoSlope,
    WallClimbFree,
    WallClimbDisabled,
    Turntable,
    Attr0x7,
    SmallLedgeClimb0x8,
    Attr0x9,
    Attr0xA,
    Attr0xB,
    Attr0xC,
    Attr0xD,
    Attr0xE,
    Attr0xF,
    SpeedCheckOff,
    MapLoop,
    MapLoopFog,
    Attr0x13,
    Attr0x14,
    Attr0x15,
    Attr0x16,
    MapSemiTranslucent,
    SpecialCamera,
    KingWarp,
    CameraHit,
    Jump,
}

impl Default for HitAttribute {
    fn default() -> Self {
        Self::None
    }
}

impl From<HitAttribute> for i32 {
    fn from(value: HitAttribute) -> Self {
        match value {
            HitAttribute::None => 0,
            HitAttribute::WaterSurface => 1,
            HitAttribute::BottomOfSea => 2,
            HitAttribute::NoReactionNoSlope => 3,
            HitAttribute::WallClimbFree => 4,
            HitAttribute::WallClimbDisabled => 5,
            HitAttribute::Turntable => 6,
            HitAttribute::Attr0x7 => 7,
            HitAttribute::SmallLedgeClimb0x8 => 8,
            HitAttribute::Attr0x9 => 9,
            HitAttribute::Attr0xA => 10,
            HitAttribute::Attr0xB => 11,
            HitAttribute::Attr0xC => 12,
            HitAttribute::Attr0xD => 13,
            HitAttribute::Attr0xE => 14,
            HitAttribute::Attr0xF => 15,
            HitAttribute::SpeedCheckOff => 16,
            HitAttribute::MapLoop => 17,
            HitAttribute::MapLoopFog => 18,
            HitAttribute::Attr0x13 => 19,
            HitAttribute::Attr0x14 => 20,
            HitAttribute::Attr0x15 => 21,
            HitAttribute::Attr0x16 => 22,
            HitAttribute::MapSemiTranslucent => 23,
            HitAttribute::SpecialCamera => 24,
            HitAttribute::KingWarp => 25,
            HitAttribute::CameraHit => 26,
            HitAttribute::Jump => 27,
        }
    }
}

impl From<i32> for HitAttribute {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::WaterSurface,
            2 => Self::BottomOfSea,
            3 => Self::NoReactionNoSlope,
            4 => Self::WallClimbFree,
            5 => Self::WallClimbDisabled,
            6 => Self::Turntable,
            7 => Self::Attr0x7,
            8 => Self::SmallLedgeClimb0x8,
            9 => Self::Attr0x9,
            10 => Self::Attr0xA,
            11 => Self::Attr0xB,
            12 => Self::Attr0xC,
            13 => Self::Attr0xD,
            14 => Self::Attr0xE,
            15 => Self::Attr0xF,
            16 => Self::SpeedCheckOff,
            17 => Self::MapLoop,
            18 => Self::MapLoopFog,
            19 => Self::Attr0x13,
            20 => Self::Attr0x14,
            21 => Self::Attr0x15,
            22 => Self::Attr0x16,
            23 => Self::MapSemiTranslucent,
            24 => Self::SpecialCamera,
            25 => Self::KingWarp,
            26 => Self::CameraHit,
            27 => Self::Jump,
            _ => {
                panic_log!("unknown hit attribute found: {}", value);
            }
        }
    }
}
