/// Ids of sounds played from the simulation.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SoundId {
    /// One of three sounds that are chosen randomly when an object is collected.
    ObjPickup0,

    /// One of three sounds that are chosen randomly when an object is collected.
    ObjPickup1,

    /// One of three sounds that are chosen randomly when an object is collected.
    ObjPickup2,

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
}

impl From<SoundId> for i32 {
    fn from(value: SoundId) -> Self {
        match value {
            SoundId::ObjPickup0 => 4,
            SoundId::ObjPickup1 => 5,
            SoundId::ObjPickup2 => 6,
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
        }
    }
}
