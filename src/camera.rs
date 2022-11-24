use gl_matrix::common::{Mat4, Vec4};

use crate::{katamari::Katamari, macros::log};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CameraMode {
    Normal,
    JumpR1,
    LookL1,
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
            0 => CameraMode::Normal,
            1 => CameraMode::JumpR1,
            2 => CameraMode::LookL1,
            3 => CameraMode::HitByProp,
            4 => CameraMode::Clear,
            5 => CameraMode::Shoot,
            6 => CameraMode::ShootRet,
            7 => CameraMode::Ending1,
            8 => CameraMode::Ending2,
            9 => CameraMode::Ending3,
            10 => CameraMode::AreaChange,
            11 => CameraMode::ClearGoalProp,
            12 => CameraMode::VsResult,
            _ => {
                log!("encountered unknown `CameraMode` value: {}", value);
                CameraMode::Unknown(value)
            }
        }
    }
}

#[derive(Debug)]
pub struct CameraParams {
    /// (??) A duration when the camera is zooming out.
    /// offset: 0x7a0b4
    pub scale_up_duration_long: i32,

    /// (??) A duration when the camera is zooming out.
    /// offset: 0x7a0b8
    pub scale_up_duration_short: i32,

    /// (??) The initial timer when changing to Shoot mode.
    pub shoot_timer_init: u16,

    /// (??) The initial timer when changing to Shootret mode.
    pub shoot_ret_timer_init: u16,
}

impl Default for CameraParams {
    fn default() -> Self {
        Self {
            scale_up_duration_long: 100,
            scale_up_duration_short: 60,
            shoot_timer_init: 0x3c,
            shoot_ret_timer_init: 0x14,
        }
    }
}

/// A control point that determines how the camera should be positioned at a specific
/// katamari size. The actual position is determined by lerping the values of the
/// two control points on either side of the katamari's actual size.
#[derive(Debug, Default)]
pub struct CameraControlPt {
    /// The minimum katamari diameter at which this control point takes effect.
    pub diam_cm: f32,

    /// The control point's camera position (relative to katamari center).
    pub pos: Vec4,

    /// The control point's camera target (relative to katamari center).
    pub target: Vec4,

    /// The max height that the prince reaches after an R1 jump.
    pub jump_r1_height: f32,
}

/// General camera state.
/// offset: 0x192ee0
/// width: 0x980
#[derive(Debug, Default)]
pub struct CameraState {
    /// The camera position's offset from the katamari center position.
    /// offset: 0x0
    kat_to_pos: Vec4,

    /// The camera target's offset from the katamari center position.
    /// offset: 0x10
    kat_to_target: Vec4,

    /// The camera position's velocity (i.e. how much it moves each tick).
    /// offset: 0x40
    pos_velocity: Vec4,

    /// The camera target's velocity (i.e. how much it moves each tick).
    /// offset: 0x50
    target_velocity: Vec4,

    /// The current mission's camera control points.
    /// offset: 0x60
    control_points: Vec<CameraControlPt>,

    /// (??) A timer counting down to when the camera will finish scaling up.
    /// offset: 0x68
    scale_up_end_timer: f32,

    /// (??) The ratio of progress (from 0 to 1) made towards the current scale up.
    /// offset: 0x6c
    scale_up_progress: f32,

    /// (??)
    /// offset: 0x70
    scale_up_duration: f32,

    /// (??)
    /// offset: 0x76
    scale_up_ticks: u16,

    /// The player to which this camera belongs
    /// offset: 0x78
    player: u8,

    /// The current area.
    /// offset: 0x7c
    area: u8,

    /// (??) True if the camera is currently scaling up.
    /// offset: 0x7d
    scale_up_in_progress: bool,

    /// The current camera mode.
    /// offset: 0x7e
    mode: CameraMode,

    /// The camera position in world space on the previous tick.
    /// offset: 0x80
    last_pos: Vec4,

    /// The camera position in world space.
    /// offset: 0x90
    pos: Vec4,

    /// The camera target in world space on the previous tick.
    /// offset: 0xa0
    last_target: Vec4,

    /// The camera target in world space.
    /// offset: 0xb0
    target: Vec4,

    /// (??) Some kind of timer for vs mode shooting.
    /// offset: 0x918
    shoot_timer: u16,

    /// (??) Some kind of position for vs mode shooting.
    /// offset: 0x91c
    shoot_pos: Vec4,

    /// If true, applies the `clear_rot` rotation about the y axis to
    /// the final camera transform.
    /// offset: 0x96a
    clear_is_rotating: bool,

    /// (??) something to do with clearing i think
    /// offset: 0x96c
    pub cam_eff_1P: bool,

    /// The extra rotation about the y axis applied to the camera after
    /// clearing a `ClearProp` mission.
    /// offset: 0x964
    clear_goal_prop_rot: f32,

    /// (??) The extra rotation about the y axis applied to the camera after
    /// clearing certain non-`ClearProp` gamemodes.
    /// offset: 0x978
    clear_rot: f32,

    /// (??) The update callback that will run during the ending gamemode.
    /// offset: 0x970
    update_ending_callback: Option<Box<fn() -> ()>>,
}

/// Transform matrices for the camera.
/// offset: 0xd34180
/// width: 0x188
#[derive(Debug, Default)]
pub struct CameraTransform {
    /// The transformation matrix of the camera looking at its target.
    /// offset: 0x0
    transform: Mat4,

    /// The position of the camera.
    /// offset: 0x170
    pos: Vec4,

    /// The extra zoom out distance as the timer expires at the end of MAS4.
    /// offset: 0x180
    mas4_preclear_offset: f32,
}

#[derive(Debug, Default)]
pub struct Camera {
    state: CameraState,
    transform: CameraTransform,
    params: CameraParams,
}

impl Camera {
    pub fn set_mode(&mut self, kat: &mut Katamari, mode: CameraMode) {
        if self.state.mode == CameraMode::LookL1 {
            kat.set_look_l1(true);
        }

        self.state.mode = mode;
        self.state.update_ending_callback = None;

        match mode {
            CameraMode::JumpR1 => {
                // TODO: `camera_set_mode:40-97`
            }
            CameraMode::LookL1 => {
                // TODO: `camera_set_mode:98-113`
            }
            CameraMode::HitByProp => {
                // TODO: `camera_set_mode: 114-129`
            }
            CameraMode::Clear => {
                // TODO: `camera_set_mode: 129-160`
            }
            CameraMode::Shoot => {
                self.state.shoot_timer = self.params.shoot_timer_init;
                self.state.shoot_pos = self.transform.pos;
            }
            CameraMode::ShootRet => {
                self.state.shoot_timer = self.params.shoot_ret_timer_init;
            }
            CameraMode::AreaChange => {
                // TODO `camera_set_mode:171-188` (but this seems to be unused in reroll??)
            }
            CameraMode::ClearGoalProp => {
                self.state.clear_goal_prop_rot = 0.0;
            }
            CameraMode::VsResult => {
                self.state.clear_rot = 0.0;
            }
            _ => (),
        };
    }

    pub fn check_scale_up(&mut self, _flag: bool) {
        // TODO: reimplement `SetCameraCheckScaleUp`
    }

    pub fn set_cam_eff_1P(&mut self, cam_eff_1P: i32) {
        self.state.cam_eff_1P = cam_eff_1P > 0;
    }

    pub fn get_matrix(
        &self,
        xx: &mut f32,
        xy: &mut f32,
        xz: &mut f32,
        yx: &mut f32,
        yy: &mut f32,
        yz: &mut f32,
        zx: &mut f32,
        zy: &mut f32,
        zz: &mut f32,
        tx: &mut f32,
        ty: &mut f32,
        tz: &mut f32,
        offset: &mut f32,
    ) {
        *xx = self.transform.transform[0];
        *xy = self.transform.transform[1];
        *xz = self.transform.transform[2];
        *yx = self.transform.transform[4];
        *yy = self.transform.transform[5];
        *yz = self.transform.transform[6];
        *zx = self.transform.transform[8];
        *zy = self.transform.transform[9];
        *zz = self.transform.transform[10];

        *tx = self.transform.pos[0];
        *ty = self.transform.pos[1];
        *tz = self.transform.pos[2];

        *offset = self.transform.mas4_preclear_offset;
    }
}
