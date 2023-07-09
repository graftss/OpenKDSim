pub mod has_delegates;

use std::{cell::RefCell, rc::Rc};

use gl_matrix::common::Vec3;

use crate::{
    collision::raycast_state::RaycastCallType,
    debug::draw::{DebugDrawBus, DebugDrawType},
};

pub type MonoGenerateDelegate = extern "C" fn(ctrl_idx: i32, name_idx: i32) -> ();
pub type MotionEndDelegate = extern "C" fn(player: i32) -> ();
pub type MessageRequestDelegate = extern "C" fn(ctrl_idx: i32) -> ();
pub type DoHitDelegate = extern "C" fn(
    p0x: f32,
    p0y: f32,
    p0z: f32,
    p1x: f32,
    p1y: f32,
    p1z: f32,
    include_objs: i32,
    draw_hits: i32,
    call_type: i32,
) -> i32;
pub type GetHitCountDelegate = extern "C" fn() -> i32;
pub type GetImpactPointDelegate =
    extern "C" fn(hit_index: i32, x: &mut f32, y: &mut f32, z: &mut f32) -> ();
pub type GetImpactNormalDelegate =
    extern "C" fn(hit_index: i32, x: &mut f32, y: &mut f32, z: &mut f32) -> ();
pub type GetHitAttributeDelegate = extern "C" fn(hit_index: i32, hit_attr: &mut i32) -> ();
pub type PlaySoundFxDelegate = extern "C" fn(sound_id: i32, volume: f32, pan: i32) -> ();
pub type PlayVisualFxDelegate = extern "C" fn(
    vfx_id: i32,
    x: f32,
    y: f32,
    z: f32,
    dir_x: f32,
    dir_y: f32,
    dir_z: f32,
    scale: f32,
    attach_id: i32,
    player: i32,
) -> ();
pub type VibrationDelegate = extern "C" fn(player: i32, ratio: f32, time: f32, no: i32) -> ();
pub type PlayAnimationDelegate =
    extern "C" fn(player: i32, animation_id: i32, speed: f32, repeat: i32) -> ();
pub type LogPropCollectedDelegate = extern "C" fn(ctrl_idx: i32) -> ();
pub type SetCameraDelegate = extern "C" fn(
    xx: f32,
    xy: f32,
    xz: f32,
    yx: f32,
    yy: f32,
    yz: f32,
    zx: f32,
    zy: f32,
    zz: f32,
    tx: f32,
    ty: f32,
    tz: f32,
) -> ();
pub type VsVolumeDiffDelegate =
    extern "C" fn(f1: i32, f2: i32, f3: i32, ratio: f32, time: f32, recover: i32) -> ();

pub type DebugDrawDelegate = extern "C" fn(draw_type: DebugDrawType) -> ();

#[derive(Default)]
pub struct Delegates {
    pub mono_generate: Option<MonoGenerateDelegate>,
    pub motion_end: Option<MotionEndDelegate>,
    pub message_request: Option<MessageRequestDelegate>,
    pub do_hit: Option<DoHitDelegate>,
    pub get_hit_count: Option<GetHitCountDelegate>,
    pub get_impact_point: Option<GetImpactPointDelegate>,
    pub get_impact_normal: Option<GetImpactNormalDelegate>,
    pub get_hit_attribute: Option<GetHitAttributeDelegate>,
    pub play_sound_fx: Option<PlaySoundFxDelegate>,
    pub play_visual_fx: Option<PlayVisualFxDelegate>,
    pub vibration: Option<VibrationDelegate>,
    pub play_animation: Option<PlayAnimationDelegate>,
    pub log_prop_collected: Option<LogPropCollectedDelegate>,
    pub set_camera: Option<SetCameraDelegate>,
    pub vs_volume_diff: Option<VsVolumeDiffDelegate>,

    // delegates unique to the open simulation
    pub debug_draw: DebugDrawBus,
}

pub type DelegatesRef = Rc<RefCell<Delegates>>;

impl core::fmt::Debug for Delegates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // don't bother writing any delegates here for now
        f.debug_struct("Delegates").finish()
    }
}

impl Delegates {
    /// Convenience wrapper around the `do_hit` delegate that allows vector inputs.
    /// offset: 0x6070
    pub fn call_do_hit(
        &self,
        point0: &Vec3,
        point1: &Vec3,
        call_type: RaycastCallType,
        include_objects: bool,
    ) -> i32 {
        if let Some(do_hit) = self.do_hit {
            do_hit(
                point0[0],
                point0[1],
                point0[2],
                point1[0],
                point1[1],
                point1[2],
                include_objects.into(),
                0,
                call_type.into(),
            )
        } else {
            0
        }
    }
}
