use crate::macros::panic_log;

/// Encodes nonstandard collision properties of surfaces. For example,
/// the `WallClimbDisabled` hit attribute on a surface stops the katamari
/// from climbing on it, and `SpeedCheckOff` is the hit attribute that the
/// big hill in the Town stage has to make the katamari move extra fast.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HitAttribute {
    None,
    WaterSurface,
    BottomOfSea,
    NoReactionNoSlope,
    WallClimbFree,
    WallClimbDisabled,
    Turntable,
    Attr0x7,
    Attr0x8,
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
            8 => Self::Attr0x8,
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
