use crate::macros::panic_log;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfxId {
    Boost,

    Treasure,

    DustCloud,

    Kasuri,

    SubmergedCircle,

    UnderwaterSplash,

    EnterWaterSplash,

    PropScared,

    Climb,

    BrakeForward,

    BrakeBackward,

    BrakeSideways,

    Impact0,

    Impact1,

    Impact2,
}

impl From<u16> for VfxId {
    fn from(value: u16) -> Self {
        match value {
            0x7 => VfxId::Boost,
            0x9 => VfxId::Treasure,
            0xb => VfxId::DustCloud,
            0xe => VfxId::Kasuri,
            0x11 => VfxId::SubmergedCircle,
            0x17 => VfxId::UnderwaterSplash,
            0x18 => VfxId::EnterWaterSplash,
            0x19 => VfxId::PropScared,
            0x1b => VfxId::Climb,
            0x1c => VfxId::BrakeForward,
            0x1d => VfxId::BrakeBackward,
            0x1e => VfxId::BrakeSideways,
            0x28 => VfxId::Impact0,
            0x29 => VfxId::Impact1,
            0x2a => VfxId::Impact2,
            _ => {
                panic_log!("unexpected vfx id: {value}");
            }
        }
    }
}

impl From<VfxId> for u16 {
    fn from(value: VfxId) -> Self {
        match value {
            VfxId::Boost => 0x7,
            VfxId::Treasure => 0x9,
            VfxId::DustCloud => 0xb,
            VfxId::Kasuri => 0xe,
            VfxId::SubmergedCircle => 0x11,
            VfxId::UnderwaterSplash => 0x17,
            VfxId::EnterWaterSplash => 0x18,
            VfxId::PropScared => 0x19,
            VfxId::Climb => 0x1b,
            VfxId::BrakeForward => 0x1c,
            VfxId::BrakeBackward => 0x1d,
            VfxId::BrakeSideways => 0x1e,
            VfxId::Impact0 => 0x28,
            VfxId::Impact1 => 0x29,
            VfxId::Impact2 => 0x2a,
        }
    }
}
