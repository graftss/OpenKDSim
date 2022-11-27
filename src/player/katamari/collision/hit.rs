use gl_matrix::common::Vec3;

use crate::{macros::panic_log, props::prop::WeakPropRef};

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

impl From<u8> for HitAttribute {
    fn from(value: u8) -> Self {
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

/// Describes a collision between a katamari collision ray and another
/// surface (which could be on either a prop or the map)
#[derive(Debug, Default, Clone)]
pub struct KatamariHit {
    /// (??)
    /// offset: 0x0
    pub clip_normal_len: f32,

    /// The length of the katamari collision ray making the contact.
    /// offset: 0x4
    pub ray_len: f32,

    /// The index of the katamari collision ray making the contact.
    /// offset: 0x8
    pub ray_idx: u16,

    /// The position along the ray where the hit occured, rescaled to be
    /// in the interval [0, 1], where 0 means the ray's initial point
    /// and 1 means its endpoint.
    /// offset: 0xc
    pub ray_hit_scaled_pos: f32,

    /// The vector from the ray initial point to its endpoint.
    /// offset: 0x10
    pub ray_vec: Vec3,

    /// The contacted surface's unit normal vector.
    /// offset: 0x20
    pub surface_normal_unit: Vec3,

    /// (??)
    /// offset: 0x30
    pub clip_normal: Vec3,

    /// The point on the surface that was contacted.
    /// offset: 0x40
    pub contact_point: Vec3,

    /// The type of surface that was contacted.
    /// offset: 0x50
    pub hit_attr: HitAttribute,

    /// If the contact surface belongs to a prop collision mesh, this
    /// is a pointer to that prop.
    /// offset: 0x58
    pub prop: Option<WeakPropRef>,
}
