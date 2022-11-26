use gl_matrix::{
    common::{Mat4, Vec3},
    mat4, vec3,
};

use crate::{
    constants::{VEC3_Y_POS, VEC3_ZERO, VEC3_Z_POS},
    katamari::Katamari,
    macros::{log, max, min},
    math::{change_bounded_angle, vec3_inplace_normalize, vec3_inplace_scale},
    prince::Prince,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
            0 => CameraMode::Normal,
            1 => CameraMode::R1Jump,
            2 => CameraMode::L1Look,
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

    /// (??)
    /// offset: 0xd345e8
    pub param_0xd345e8: f32,

    /// (??)
    /// offset: 0xd345ec
    pub param_0xd345ec: f32,

    /// A factor controlling how much the "special camera" move closer
    /// *laterally* to the katamari.
    pub special_camera_tighten: f32,

    /// The vertical speed of the L1 look camera.
    /// offset: 0x7a058
    pub l1_look_speed_y: f32,

    /// The horizontal speed of the L1 look camera.
    /// offset: 0x7a05c
    pub l1_look_speed_x: f32,

    /// The max y angle of the L1 look camera.
    /// offset: 0x7a050
    pub l1_look_max_y: f32,

    /// The min y angle of the L1 look camera.
    /// offset: 0x7a054
    pub l1_look_min_y: f32,
}

impl Default for CameraParams {
    fn default() -> Self {
        Self {
            scale_up_duration_long: 100,
            scale_up_duration_short: 60,
            shoot_timer_init: 0x3c,
            shoot_ret_timer_init: 0x14,
            param_0xd345e8: f32::from_bits(0xff027d4b),
            param_0xd345ec: 0.0,
            special_camera_tighten: 0.75,
            l1_look_speed_x: f32::from_bits(0x3d0ef998),
            l1_look_speed_y: f32::from_bits(0x3cc82be9),
            l1_look_max_y: f32::from_bits(0x3f860a7c),
            l1_look_min_y: f32::from_bits(0x3e71465f),
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
    pub pos: Vec3,

    /// The control point's camera target (relative to katamari center).
    pub target: Vec3,

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
    kat_to_pos: Vec3,

    /// The camera target's offset from the katamari center position.
    /// offset: 0x10
    kat_to_target: Vec3,

    /// The camera position's velocity (i.e. how much it moves each tick).
    /// offset: 0x40
    pos_velocity: Vec3,

    /// The camera target's velocity (i.e. how much it moves each tick).
    /// offset: 0x50
    target_velocity: Vec3,

    /// The current mission's camera control points.
    /// offset: 0x60
    control_points: Vec<CameraControlPt>,

    /// (??) A timer counting down to when the camera will finish scaling up.
    /// offset: 0x68
    scale_up_end_timer: f32,

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
    last_pos: Vec3,

    /// The camera position in world space.
    /// offset: 0x90
    pos: Vec3,

    /// The camera target in world space on the previous tick.
    /// offset: 0xa0
    last_target: Vec3,

    /// The camera target in world space.
    /// offset: 0xb0
    target: Vec3,

    /// (??)
    /// offset: 0x8a8
    l1_look_init_pos_to_target: Vec3,

    /// The current y angle of the L1 look camera.
    /// offset: 0x8b8
    l1_look_y_angle: f32,

    /// (??) Some kind of timer for vs mode shooting.
    /// offset: 0x918
    shoot_timer: u16,

    /// (??) Some kind of position for vs mode shooting.
    /// offset: 0x91c
    shoot_pos: Vec3,

    /// If true, eases the camera towards its intended position.
    /// If false, the camera instantly teleports the behind the prince every tick.
    /// offset: 0x969
    apply_easing: bool,

    /// If true, applies the `clear_rot` rotation about the y axis to
    /// the final camera transform.
    /// offset: 0x96a
    clear_is_rotating: bool,

    /// (??)
    /// offset: 0x96b
    cam_eff_1P_related: bool,

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

    /// If true, uses the "zoomed in" camera position (e.g. under living room table in MAS1/MAS2)
    /// offset: katamari+0x10b
    pub use_special_camera: bool,

    /// If true, the "special camera" state is currently transitioning to the current value
    /// of `use_special_camera`.
    /// offset: katamari+0x10c
    pub changing_special_camera: bool,
}

/// Transform matrices for the camera.
/// offset: 0xd34180
/// width: 0x188
#[derive(Debug, Default)]
pub struct CameraTransform {
    /// The transformation matrix of the camera looking at its target.
    /// offset: 0x0
    transform: Mat4,

    /// The rotation component of `transform`
    /// offset: 0xc0
    transform_rot: Mat4,

    /// The camera's "up" vector (which should always be the y+ axis unit vector)
    /// offset: 0x140
    up: Vec3,

    /// (??) The camera's rotation expressed as Euler angles
    /// offset: 0x150
    euler_angles: Vec3,

    /// The target of the camera.
    /// offset: 0x160
    target: Vec3,

    /// The position of the camera.
    /// offset: 0x170
    pos: Vec3,

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
    pub fn get_mode(&self) -> CameraMode {
        self.state.mode
    }

    pub fn is_scaling_up(&self) -> bool {
        self.state.scale_up_in_progress
    }
}

impl Camera {
    pub fn init(&mut self, kat: &Katamari, prince: &Prince) {
        self.init_state(kat, prince);
        self.set_mode(CameraMode::Normal);
        self.init_transform();
        self.reset_state(kat, prince);
    }

    /// Initialize the `CameraState` struct.
    /// offset: 0xb410
    pub fn init_state(&mut self, kat: &Katamari, prince: &Prince) {
        let mut pos = vec3::create();
        let mut target = vec3::create();

        self.compute_normal_pos_and_target(kat, prince, &mut pos, &mut target);
        self.state.pos = pos;
        self.state.last_pos = pos;
        self.state.target = target;
        self.state.last_target = target;
    }

    /// Reset the camera state. This is performed at the start of every mission
    /// and after a royal warp.
    /// offset: 0xaba0
    pub fn reset_state(&mut self, kat: &Katamari, prince: &Prince) {
        self.state.update_ending_callback = None;
        self.state.apply_easing = true;

        let mut pos = vec3::create();
        let mut target = vec3::create();
        self.compute_normal_pos_and_target(kat, prince, &mut pos, &mut target);

        self.state.pos = pos;
        self.state.last_pos = pos;
        self.state.target = target;
        self.state.last_target = target;
        self.state.cam_eff_1P = false;
        self.state.cam_eff_1P_related = false;
    }

    /// Update the camera state during an L1 look with the left stick input `(ls_x, ls_y)`.
    /// Since the camera's x angle is just the prince's angle, a mutable reference to that
    /// field on the `Prince` object is needed.
    /// offset: 0x54c90 (the second half)
    pub fn update_l1_look(&mut self, ls_x: f32, ls_y: f32, prince_angle: &mut f32) {
        let speed_x = self.params.l1_look_speed_x;
        let speed_y = self.params.l1_look_speed_y;
        let min_y = self.params.l1_look_min_y;
        let max_y = self.params.l1_look_max_y;

        // update y angle
        if ls_y > 0.0 {
            self.state.l1_look_y_angle = max!(self.state.l1_look_y_angle - speed_y * ls_y, max_y);
        } else if ls_y < 0.0 {
            self.state.l1_look_y_angle = min!(self.state.l1_look_y_angle - speed_y * ls_y, min_y);
        }

        // update x angle, which is written directly to the prince
        change_bounded_angle(prince_angle, ls_x * speed_x);
    }

    /// Writes the current desired camera position and target to
    /// `pos` and `target`, respectively.
    /// offset: 0xd4a0
    pub fn compute_normal_pos_and_target(
        &mut self,
        kat: &Katamari,
        prince: &Prince,
        pos: &mut Vec3,
        target: &mut Vec3,
    ) {
        let mat4_id = mat4::create();
        let mut mat2 = mat4::create();
        let mut vec1 = vec3::create();

        let kat_center = kat.get_center();

        // compute the unit vector in the direction from the prince to the katamari
        let mut prince_to_kat = vec3::create();
        vec3::subtract(&mut prince_to_kat, &kat.get_center(), &prince.get_pos());
        prince_to_kat[1] = 0.0;
        vec3_inplace_normalize(&mut prince_to_kat);

        if kat.physics_flags.under_water {
            // TODO: `camera_compute_normal_pos_and_target:63-150`
        } else {
            if self.state.clear_is_rotating {
                // if doing the mission clear rotation, apply the angle from that
                // rotation to the `prince_to_kat` vector.
                mat4::rotate_y(&mut mat2, &mat4_id, self.state.clear_rot);
                vec3::copy(&mut vec1, &prince_to_kat);
                vec3::transform_mat4(&mut prince_to_kat, &vec1, &mat2);
            }

            // compute camera target
            let target_offset = self.state.kat_to_target[2];
            target[0] = kat_center[0] + target_offset * prince_to_kat[0];
            target[2] = kat_center[2] + target_offset * prince_to_kat[2];

            let pos_offset = self.state.kat_to_pos[2];
            let mut kat_to_cam_pos = vec3::create();
            vec3::scale(&mut kat_to_cam_pos, &prince_to_kat, pos_offset);

            // update special camera state before computing camera position
            self.state.use_special_camera =
                if !kat.hit_flags.special_camera && !kat.last_hit_flags[0].special_camera {
                    // if special camera is off
                    if self.state.use_special_camera {
                        // detect when special camera is changing from on to off
                        self.state.changing_special_camera = true;
                    }
                    false
                } else {
                    // if special camera is on, scooch in the camera position
                    vec3_inplace_scale(&mut kat_to_cam_pos, self.params.special_camera_tighten);

                    if !self.state.use_special_camera {
                        // detect when special camera is changing from off to on
                        self.state.changing_special_camera = true;
                    }
                    true
                };

            // compute camera position
            pos[0] = kat_center[0] + kat_to_cam_pos[0];
            pos[2] = kat_center[2] + kat_to_cam_pos[2];

            // TODO: `camera_compute_normal_pos_and_target:217-221` (extra weird check)
        }
    }

    /// Initialize the `CameraTransform` struct
    pub fn init_transform(&mut self) {
        self.transform.pos = VEC3_ZERO;
        self.transform.euler_angles = VEC3_ZERO;
        self.transform.target = VEC3_Z_POS;
        self.transform.up = VEC3_Y_POS;
        self.transform.mas4_preclear_offset = 0.0;
    }

    pub fn set_mode(&mut self, mode: CameraMode) {
        // if self.state.mode == CameraMode::LookL1 {
        //     kat.set_look_l1(true);
        // }

        self.state.mode = mode;
        self.state.update_ending_callback = None;

        match mode {
            CameraMode::R1Jump => {
                // TODO: `camera_set_mode:40-97`
            }
            CameraMode::L1Look => {
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
