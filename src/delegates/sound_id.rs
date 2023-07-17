use crate::macros::panic_log;

/// Ids of sounds played from the simulation.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SoundId {
    /// One of three sounds that are chosen randomly when an object is collected while
    /// the katamari is small.
    VerySmallCollect0,

    /// One of three sounds that are chosen randomly when an object is collected while
    /// the katamari is small.
    VerySmallCollect1,

    /// One of three sounds that are chosen randomly when an object is collected while
    /// the katamari is small.
    VerySmallCollect2,

    /// One of three sounds that are chosen randomly when an object is collected while
    /// the katamari is small.
    SmallCollect0,

    /// One of three sounds that are chosen randomly when an object is collected while
    /// the katamari is small.
    SmallCollect1,

    /// One of three sounds that are chosen randomly when an object is collected while
    /// the katamari is small.
    SmallCollect2,

    /// One of three sounds that are chosen randomly when an object is collected while
    /// the katamari is medium-sized.
    MediumCollect0,

    /// One of three sounds that are chosen randomly when an object is collected while
    /// the katamari is medium-sized.
    MediumCollect1,

    /// One of three sounds that are chosen randomly when an object is collected while
    /// the katamari is medium-sized.
    MediumCollect2,

    /// One of three sounds that are chosen randomly when an object is collected while
    /// the katamari is large.
    LargeCollect0,

    /// One of three sounds that are chosen randomly when an object is collected while
    /// the katamari is large.
    LargeCollect1,

    /// One of three sounds that are chosen randomly when an object is collected while
    /// the katamari is large.
    LargeCollect2,

    /// One of five sounds played when the katamari loses a prop from a collision.
    /// The sound chosen depends on the katamari size and the current stage.
    LoseProp0,

    /// One of five sounds played when the katamari loses a prop from a collision.
    /// The sound chosen depends on the katamari size and the current stage.
    LoseProp1,

    /// One of five sounds played when the katamari loses a prop from a collision.
    /// The sound chosen depends on the katamari size and the current stage.
    LoseProp2,

    /// One of five sounds played when the katamari loses a prop from a collision.
    /// The sound chosen depends on the katamari size and the current stage.
    LoseProp3,

    /// One of five sounds played when the katamari loses a prop from a collision.
    /// The sound chosen depends on the katamari size and the current stage.
    LoseProp4,

    /// One of five sounds played when the katamari bonks a wall without losing a prop.
    /// The sound chosen depends on the katamari size and the current stage.
    WallBonk0,

    /// One of five sounds played when the katamari bonks a wall without losing a prop.
    /// The sound chosen depends on the katamari size and the current stage.
    WallBonk1,

    /// One of five sounds played when the katamari bonks a wall without losing a prop.
    /// The sound chosen depends on the katamari size and the current stage.
    WallBonk2,

    /// One of five sounds played when the katamari bonks a wall without losing a prop.
    /// The sound chosen depends on the katamari size and the current stage.
    WallBonk3,

    /// One of five sounds played when the katamari bonks a wall without losing a prop.
    /// The sound chosen depends on the katamari size and the current stage.
    WallBonk4,

    /// The sound made when the katamari begins a spin.
    Spin,

    /// The sound made when the katamari begins a boost.
    Boost,

    /// The sound made when the katamari begins a brake.
    Brake,

    /// The sound made when the katamari hits a wall.
    HitWall,

    /// The sound made when the katamari hits the ground after falling at least a moderate distance.
    HitGround,

    /// The sound made when the katamari vaults.
    Vault,

    /// The sound made when the prince flips around the katamari.
    Flip,

    /// The sound made when an R1 jump begins.
    R1JumpStart,

    /// The sound made when an R1 jump ends.
    R1JumpEnd,

    /// The sound made when a prop is knocked airborne.
    HitPropAirborne,

    /// The sound made after a prop gets up from being knocked airborne and begins to flee.
    PropFlee,

    /// The sound made when the "something's coming" alarm goes off.
    SomethingComing,

    /// The sound made repeatedly while the katamari is underwater.
    Underwater,

    /// The sound made when a policeman prop fires its gun.
    Gunshot,

    /// The sound made once when the katamari enters water.
    EnterWater,

    /// Unknown vs mode sound.
    VsMode0x34,

    /// Unknown vs mode sound.
    VsMode0x3b,

    /// Unknown vs mode sound.
    VsMode0x61,

    /// The sounds made by some objects when they are bonked.
    PropBonk(u8),

    /// The sounds made by some objects when they are collected.
    PropCollect(u8),
}

impl From<SoundId> for u16 {
    fn from(value: SoundId) -> Self {
        match value {
            SoundId::VerySmallCollect0 => 0x1,
            SoundId::VerySmallCollect1 => 0x2,
            SoundId::VerySmallCollect2 => 0x3,
            SoundId::SmallCollect0 => 0x4,
            SoundId::SmallCollect1 => 0x5,
            SoundId::SmallCollect2 => 0x6,
            SoundId::MediumCollect0 => 0x7,
            SoundId::MediumCollect1 => 0x8,
            SoundId::MediumCollect2 => 0x9,
            SoundId::LargeCollect0 => 0xa,
            SoundId::LargeCollect1 => 0xb,
            SoundId::LargeCollect2 => 0xc,
            SoundId::LoseProp0 => 0x10,
            SoundId::LoseProp1 => 0x11,
            SoundId::LoseProp2 => 0x12,
            SoundId::LoseProp3 => 0x13,
            SoundId::LoseProp4 => 0x14,
            SoundId::WallBonk0 => 0x15,
            SoundId::WallBonk1 => 0x16,
            SoundId::WallBonk2 => 0x17,
            SoundId::WallBonk3 => 0x18,
            SoundId::WallBonk4 => 0x19,
            SoundId::Spin => 0x1a,
            SoundId::Boost => 0x1b,
            SoundId::Brake => 0x1c,
            SoundId::HitWall => 0x1d,
            SoundId::HitGround => 0x1e,
            SoundId::Vault => 0x1f,
            SoundId::Flip => 0x20,
            SoundId::R1JumpStart => 0x29,
            SoundId::R1JumpEnd => 0x2a,
            SoundId::HitPropAirborne => 0x2b,
            SoundId::PropFlee => 0x2c,
            SoundId::SomethingComing => 0x2d,
            SoundId::Underwater => 0x30,
            SoundId::Gunshot => 0x31,
            SoundId::EnterWater => 0x33,
            SoundId::VsMode0x34 => 0x34,
            SoundId::VsMode0x3b => 0x3b,
            SoundId::VsMode0x61 => 0x61,
            SoundId::PropCollect(kind) => 0x78 + (kind as u16 - 1) * 2,
            SoundId::PropBonk(kind) => 0x79 + (kind as u16 - 1) * 2,
        }
    }
}

impl From<u16> for SoundId {
    fn from(value: u16) -> Self {
        match value {
            0x1 => SoundId::VerySmallCollect0,
            0x2 => SoundId::VerySmallCollect1,
            0x3 => SoundId::VerySmallCollect2,
            0x4 => SoundId::SmallCollect0,
            0x5 => SoundId::SmallCollect1,
            0x6 => SoundId::SmallCollect2,
            0x7 => SoundId::MediumCollect0,
            0x8 => SoundId::MediumCollect1,
            0x9 => SoundId::MediumCollect2,
            0xa => SoundId::LargeCollect0,
            0xb => SoundId::LargeCollect1,
            0xc => SoundId::LargeCollect2,
            0x10 => SoundId::LoseProp0,
            0x11 => SoundId::LoseProp1,
            0x12 => SoundId::LoseProp2,
            0x13 => SoundId::LoseProp3,
            0x14 => SoundId::LoseProp4,
            0x15 => SoundId::WallBonk0,
            0x16 => SoundId::WallBonk1,
            0x17 => SoundId::WallBonk2,
            0x18 => SoundId::WallBonk3,
            0x19 => SoundId::WallBonk4,
            0x1a => SoundId::Spin,
            0x1b => SoundId::Boost,
            0x1c => SoundId::Brake,
            0x1d => SoundId::HitWall,
            0x1e => SoundId::HitGround,
            0x1f => SoundId::Vault,
            0x20 => SoundId::Flip,
            0x29 => SoundId::R1JumpStart,
            0x2a => SoundId::R1JumpEnd,
            0x2b => SoundId::HitPropAirborne,
            0x2c => SoundId::PropFlee,
            0x2d => SoundId::SomethingComing,
            0x30 => SoundId::Underwater,
            0x31 => SoundId::Gunshot,
            0x33 => SoundId::EnterWater,
            0x34 => SoundId::VsMode0x34,
            0x3b => SoundId::VsMode0x3b,
            0x61 => SoundId::VsMode0x61,
            _ if value % 2 == 0 && value >= 0x78 && value <= 0x203 => {
                let kind = (value - 0x78) / 2 + 1;
                SoundId::PropCollect(kind as u8)
            }
            _ if value % 2 == 1 && value >= 0x78 && value <= 0x203 => {
                let kind = (value - 0x79) / 2 + 1;
                SoundId::PropBonk(kind as u8)
            }
            _ => {
                panic_log!("unexpected sound id: {}", value);
            }
        }
    }
}
