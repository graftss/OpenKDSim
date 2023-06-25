use gl_matrix::{
    common::{Mat4, Vec3},
    mat4, quat, vec3, vec4,
};

use crate::{
    collision::raycast_state::RaycastState,
    constants::{UNITY_TO_SIM_SCALE, VEC3_Y_POS, VEC3_ZERO, VEC3_Z_POS},
    macros::{log, max, min, set_translation, set_y, temp_debug_log, vec3_from},
    math::{
        change_bounded_angle, mat4_compute_yaw_rot, mat4_look_at, vec3_inplace_normalize,
        vec3_inplace_scale, vec3_inplace_subtract_vec,
    },
    mission::{
        config::{CamScaledCtrlPt, MissionConfig},
        state::MissionState,
        GameMode,
    },
};

use self::{mode::CameraMode, params::CameraParams, preclear::PreclearState};

use super::{katamari::Katamari, prince::Prince};

pub mod mode;
pub mod params;
pub mod preclear;

/// TODO
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CamOverrideType {
    PrinceLocked,
}

/// Different camera states when transitioning into and out of an R1 jump.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CamR1JumpState {
    /// At the start of a jump when the prince is gaining height.
    Rising,

    /// During a jump when the prince is at max height.
    AtPeak,

    /// At the end of a jump when the prince loses height.
    Falling,
}

impl Into<u8> for CamR1JumpState {
    fn into(self) -> u8 {
        match self {
            CamR1JumpState::Rising => 0,
            CamR1JumpState::AtPeak => 1,
            CamR1JumpState::Falling => 2,
        }
    }
}

/// General camera state.
/// offset: 0x192ee0
/// width: 0x980
#[derive(Debug, Default)]
pub struct CameraState {
    // START extra fields not in the original simulation
    /// In the original simulation, this was a global variable used to lock the
    /// camera to the prince after some prop collisions.
    /// offset: 0x10ead8
    override_type: Option<CamOverrideType>,

    raycast_state: RaycastState,

    // END extra fields not in the original simulation
    /// The camera position's offset from the katamari center position.
    /// This vector is usually constant, but changes during "swirl" size-up effects
    /// and when the camera moves to avoid looking through a wall.
    /// offset: 0x0
    kat_to_pos: Vec3,

    /// The camera target's offset from the katamari center position.
    /// This vector is usually constant, but changes during "swirl" size-up effects
    /// and when the camera moves to avoid looking through a wall.
    /// offset: 0x10
    kat_to_target: Vec3,

    /// The camera position's offset from the katamari center position, ignoring
    /// camera movements to avoid looking through walls.
    /// offset: 0x20
    kat_to_pos_noclip: Vec3,

    /// The camera target's offset from the katamari center position, ignoring
    /// camera movements to avoid looking through walls.
    /// offset: 0x30
    kat_to_target_noclip: Vec3,

    /// The camera position's velocity (i.e. how much it moves each tick).
    /// offset: 0x40
    pos_velocity: Vec3,

    /// The camera target's velocity (i.e. how much it moves each tick).
    /// offset: 0x50
    target_velocity: Vec3,

    /// The current mission's camera control points.
    /// offset: 0x60
    pub kat_offset_ctrl_pts: Vec<CamScaledCtrlPt>,

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

    /// The index of the current `CamScaledCtrlPt` being used.
    /// offset: 0x7c
    pub kat_offset_ctrl_pt_idx: u8,

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

    /// The initial camera position at the start of the r1 jump.
    /// offset: 0x844
    r1_jump_init_pos: Vec3,

    /// offset: 0x854
    r1_jump_target: Vec3,

    /// offset: 0x864
    r1_jump_extra_height: Vec3,

    /// offset: 0x874
    r1_jump_last_extra_height: Vec3,

    /// offset: 0x884
    r1_jump_timer: u16,

    /// offset: 0x888
    r1_jump_duration: u16,

    /// offset: 0x88c
    r1_jump_peak_height: f32,

    /// offset: 0x890
    r1_jump_height_ratio: f32,

    /// offset: 0x894
    r1_jump_state: Option<CamR1JumpState>,

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

impl CameraState {
    /// Main function to update the camera state. Computes the next camera position and target,
    /// and writes that data to the transform.
    /// TODO_REFACTOR: those two steps should be separated once everything is working
    /// offset: 0xb7d0
    pub fn update(
        &mut self,
        prince: &Prince,
        katamari: &mut Katamari,
        mission_state: &MissionState,
        cam_transform: &mut CameraTransform,
    ) {
        self.last_pos = self.pos;
        self.last_target = self.target;

        match self.mode {
            CameraMode::Normal => {
                self.update_main(prince, katamari, true, mission_state, cam_transform);
                // TODO: self.update_clip_pos(prince, katamari);
            }
            CameraMode::R1Jump => {
                self.update_r1_jump(prince, katamari);
            }
            CameraMode::L1Look => {
                self.update_main(prince, katamari, false, mission_state, cam_transform);
            }
            CameraMode::HitByProp => {
                // TODO: `camera_update_state:67-115`
            }
            CameraMode::Clear => {
                // TODO: `camera_update_state:116-151`
            }
            CameraMode::Shoot => {
                // TODO_VS: `camera_update_state:152-178`
            }
            CameraMode::ShootRet => {
                // TODO_VS: `camera_update_state:179-188`
            }
            CameraMode::Ending1 | CameraMode::Ending2 | CameraMode::Ending3 => {
                // TODO: call `self.state.update_ending_callback`,
                // but presumably it would be easier to just call a concrete
                // function here...
            }
            CameraMode::AreaChange => {
                // TODO: `camera_update_state:196-237`
            }
            CameraMode::ClearGoalProp => {
                self.update_clear_goal_prop();
            }
            CameraMode::VsResult => {
                // TODO_VS: `camera_update_vs_result()`
            }
            CameraMode::Unknown(_) => (),
        }
    }

    /// The default camera update function that applies to `Normal` mode and several other
    /// special modes. The camera position and target points are computed, then written to
    /// the camera transform.
    /// offset: 0xc500
    fn update_main(
        &mut self,
        prince: &Prince,
        katamari: &Katamari,
        is_normal_mode: bool,
        mission_state: &MissionState,
        cam_transform: &mut CameraTransform,
    ) {
        self.update_pos_and_target_main(prince, katamari, is_normal_mode, mission_state);
        cam_transform.pos = self.pos;
        cam_transform.target = self.target;
    }

    /// Update this state's camera position and target points.
    fn update_pos_and_target_main(
        &mut self,
        prince: &Prince,
        katamari: &Katamari,
        is_normal_mode: bool,
        mission_state: &MissionState,
    ) {
        if !self.scale_up_in_progress || !is_normal_mode {
            self.update_area_params(&mission_state.mission_config, katamari.get_diam_cm());
        } else {
            // TODO: `camera_update_main:53-140` (scaling up camera after swirl, presumably)
        }

        let mut pos = [0.0; 3];
        let mut target = [0.0; 3];

        if prince.oujistate.jump_180 {
            // if flipping, defer to the flip subroutine and bail
            self.compute_flip_pos_and_target(&mut pos, &mut target);
            self.pos = pos;
            self.target = target;
            return;
        }

        match self.override_type {
            None => {
                // if there's no camera override:
                if mission_state.gamemode == GameMode::Ending || self.mode == CameraMode::Normal {
                    // in the ending mission or normal mode:
                    self.compute_normal_pos_and_target(&mut pos, &mut target, katamari, prince);
                } else {
                    self.compute_abnormal_pos_and_target(&mut pos, &mut target)
                }

                // TODO: `camera_update_main:174-236` (easing camera)

                // this line is temporary until the easing is added
                self.pos = pos;
                self.target = target;
            }
            Some(CamOverrideType::PrinceLocked) => {
                self.compute_normal_pos_and_target(&mut pos, &mut target, katamari, prince);
                self.pos = pos;
                self.pos = target;
            }
        }
    }

    /// (??) reads the next swirl params from the mission config, but it seems like
    /// other stuff too
    /// offset: 0xd0b0
    fn update_area_params(&mut self, mission_config: &MissionConfig, diam_cm: f32) {
        // TODO
        mission_config.get_camera_ctrl_point(self, diam_cm);
    }

    /// Writes the camera position and target points during normal camera movement
    /// to the vectors `pos` and `target`.
    /// offset: 0xd4a0
    fn compute_normal_pos_and_target(
        &mut self,
        mut pos: &mut Vec3,
        mut target: &mut Vec3,
        katamari: &Katamari,
        prince: &Prince,
    ) {
        // compute the lateral unit vector from the prince to the katamari
        let kat_center = katamari.get_center();
        let prince_pos = prince.get_pos();
        let mut pri_to_kat_unit = vec3_from!(-, kat_center, prince_pos);
        set_y!(pri_to_kat_unit, 0.0);
        vec3_inplace_normalize(&mut pri_to_kat_unit);

        // TODO: `camera_compute_normal_pos_and_target:65-187` (a bunch of unusual cases)

        let pos_offset = [
            pri_to_kat_unit[0] * self.kat_to_pos[2],
            self.kat_to_pos[1],
            pri_to_kat_unit[2] * self.kat_to_pos[2],
        ];

        vec3::add(&mut pos, &kat_center, &pos_offset);

        // TODO: `camera_compute_normal_pos_and_target:198-213` (handle `SpecialCamera` flag)
        let target_offset = [
            pri_to_kat_unit[0] * self.kat_to_target[2],
            self.kat_to_target[1],
            pri_to_kat_unit[2] * self.kat_to_target[2],
        ];
        vec3::add(&mut target, &kat_center, &target_offset);

        // TODO: `camera_compute_normal_pos_and_target:217-221` (special case: in water on world)
    }

    /// Writes the camera position and target points during abnormal camera movement
    /// to the vectors `pos` and `target`.
    /// offset: 0xdc80
    fn compute_abnormal_pos_and_target(&mut self, _pos: &mut Vec3, _target: &mut Vec3) {
        // TODO
    }

    /// Writes the camera position and target points during a flip ("jump 180")
    /// to the vectors `pos` and `target`.
    /// offset: 0xf5c0
    fn compute_flip_pos_and_target(&mut self, _pos: &mut Vec3, _target: &mut Vec3) {
        // TODO
    }

    /// Check if the camera would look through any walls, and adjust its position if it would.
    /// offset: 0xe5b0
    fn update_clip_pos(&mut self, prince: &Prince, katamari: &mut Katamari) {
        // save the current pos and target offsets, then compute the next
        // pos and target based on the noclip offsets.
        let _kat_to_pos_init = self.kat_to_pos.clone();
        let _kat_to_target_init = self.kat_to_target.clone();
        self.kat_to_pos = self.kat_to_pos_noclip;
        self.kat_to_target = self.kat_to_target_noclip;

        let mut noclip_pos = [0.0; 3];
        let mut noclip_target = [0.0; 3];

        if !prince.oujistate.jump_180 {
            // temporarily set `under_water` and `in_water` flags to false for
            // the purposes of computing the camera position and target.
            // TODO: is this necessary?
            let under_water = katamari.physics_flags.under_water;
            let in_water = katamari.physics_flags.in_water;
            katamari.physics_flags.under_water = false;
            katamari.physics_flags.in_water = false;

            self.compute_normal_pos_and_target(
                &mut noclip_pos,
                &mut noclip_target,
                katamari,
                prince,
            );

            katamari.physics_flags.under_water = under_water;
            katamari.physics_flags.in_water = in_water;
        } else {
            self.compute_flip_pos_and_target(&mut noclip_pos, &mut noclip_target);
        }

        // TODO: `camera_update_normal:103-173` (check if noclip camera clipped)
        // self.raycast_state.load_ray(katamari.get_center(), &noclip_pos);
        // if self.raycast_state.find_nearest_unity_hit(RaycastCallType::Stage, true) {

        // }

        // TODO: temporary line until camera clipping is added
        self.kat_to_pos = self.kat_to_pos_noclip;
    }

    /// The camera update function for `R1Jump` mode.
    /// offset: 0xbe60
    fn update_r1_jump(&mut self, _prince: &Prince, _katamari: &Katamari) {}

    /// TODO: `camera_update_clear_goal_prop`
    /// offset: 0xebf0
    fn update_clear_goal_prop(&mut self) {}

    /// Set the camera's offsets from the katamari to those described by the control point `pt`.
    pub fn set_kat_offsets(&mut self, pt: &CamScaledCtrlPt) {
        vec3::copy(&mut self.kat_to_pos, &pt.kat_to_pos);
        vec3::copy(&mut self.kat_to_target, &pt.kat_to_target);
        self.kat_to_pos[1] *= -1.0;
        self.kat_to_target[1] *= -1.0;
    }
}

/// Transform matrices for the camera.
/// offset: 0xd34180
/// width: 0x188
#[derive(Debug, Default)]
pub struct CameraTransform {
    /// The transformation matrix of the camera looking at its target.
    /// offset: 0x0
    pub lookat: Mat4,

    /// The yaw rotation component of the `lookat` matrix.
    /// offset: 0x40
    pub lookat_yaw_rot: Mat4,

    /// (??)
    /// offset: 0x80
    mat_0x80: Mat4,

    /// The inverse of the rotation component of the `lookat` matrix.
    /// offset: 0xc0
    lookat_rot_inv: Mat4,

    /// The yaw rotation component of `lookat_rot_inv`.
    /// offset: 0x100
    pub lookat_yaw_rot_inv: Mat4,

    /// The camera's "up" vector (which should always be the y+ axis unit vector)
    /// offset: 0x140
    up: Vec3,

    /// (??) The camera's rotation expressed as Euler angles
    /// offset: 0x150
    euler_angles: Vec3,

    /// The target of the camera.
    /// offset: 0x160
    pub target: Vec3,

    /// The position of the camera.
    /// offset: 0x170
    pub pos: Vec3,

    /// The extra zoom out distance as the timer expires at the end of MAS4.
    /// offset: 0x180
    mas4_preclear_offset: f32,
}

impl CameraTransform {
    /// Update the camera transform using the current values of `pos` and `target`,
    /// which should have been already updated.
    /// offset: 0x57fd0
    pub fn update(&mut self) {
        // compute the lookat matrix of the camera
        mat4_look_at(&mut self.lookat, &self.pos, &self.target, &VEC3_Y_POS);

        // compute the yaw component of the lookat matrix
        mat4_compute_yaw_rot(&mut self.lookat_yaw_rot, &self.lookat);

        // compute the inverse rotation of the lookat matrix.
        mat4::transpose(&mut self.lookat_rot_inv, &self.lookat);
        self.lookat_rot_inv[3] = 0.0;
        self.lookat_rot_inv[7] = 0.0;
        self.lookat_rot_inv[11] = 0.0;

        mat4_compute_yaw_rot(&mut self.lookat_yaw_rot_inv, &self.lookat_rot_inv);

        // TODO: `camera_update_extra_matrices()` (offset 0x58e40)
        //       this is a separate function in the simulation, but it's always called immediately
        //       after the 0x57fd0 function, so they should probably just be combined
    }
}

#[derive(Debug, Default)]
pub struct Camera {
    pub state: CameraState,
    pub transform: CameraTransform,
    pub params: CameraParams,
    pub preclear: PreclearState,
}

impl Camera {
    pub fn get_mode(&self) -> CameraMode {
        self.state.mode
    }

    pub fn get_transform(&self) -> &CameraTransform {
        &self.transform
    }

    pub fn is_scaling_up(&self) -> bool {
        self.state.scale_up_in_progress
    }

    pub fn get_r1_jump_state(&self) -> Option<CamR1JumpState> {
        self.state.r1_jump_state
    }

    pub fn get_r1_jump_height_ratio(&self) -> f32 {
        self.state.r1_jump_height_ratio
    }

    pub fn set_delay(&mut self, x: f32, y: f32, z: f32) {
        self.params.delay_x = x;
        self.params.delay_y = y;
        self.params.delay_z = z;
    }
}

impl Camera {
    pub fn init(&mut self, katamari: &Katamari, prince: &Prince, mission_config: &MissionConfig) {
        self.init_state(katamari, prince);
        self.set_mode(CameraMode::Normal);
        self.init_transform();
        self.reset_state(katamari, prince);

        mission_config.get_camera_ctrl_point(&mut self.state, katamari.get_diam_cm());
    }

    /// Initialize the `CameraState` struct.
    /// offset: 0xb410
    pub fn init_state(&mut self, katamari: &Katamari, prince: &Prince) {
        let mut pos = vec3::create();
        let mut target = vec3::create();

        self.compute_normal_pos_and_target(katamari, prince, &mut pos, &mut target);
        self.state.pos = pos;
        self.state.last_pos = pos;
        self.state.target = target;
        self.state.last_target = target;
    }

    /// Reset the camera state. This is performed at the start of every mission
    /// and after a royal warp.
    /// offset: 0xaba0
    pub fn reset_state(&mut self, katamari: &Katamari, prince: &Prince) {
        self.state.update_ending_callback = None;
        self.state.apply_easing = true;

        let mut pos = vec3::create();
        let mut target = vec3::create();
        self.compute_normal_pos_and_target(katamari, prince, &mut pos, &mut target);

        self.state.pos = pos;
        self.state.last_pos = pos;
        self.state.target = target;
        self.state.last_target = target;
        self.state.cam_eff_1P = false;
        self.state.cam_eff_1P_related = false;
    }

    /// Update the camera.
    pub fn update(
        &mut self,
        prince: &Prince,
        katamari: &mut Katamari,
        mission_state: &MissionState,
    ) {
        // TODO_REFACTOR: is it really necessary to propagate the pos and target twice?
        self.transform.update();
        self.state
            .update(prince, katamari, mission_state, &mut self.transform);
        self.transform.update();
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
        katamari: &Katamari,
        prince: &Prince,
        mut pos: &mut Vec3,
        mut target: &mut Vec3,
    ) {
        let mat4_id = mat4::create();
        let mut mat2 = mat4::create();
        let mut vec1 = vec3::create();

        let kat_center = katamari.get_center();

        // compute the unit vector in the direction from the prince to the katamari
        let mut prince_to_kat = vec3::create();
        vec3::subtract(
            &mut prince_to_kat,
            &katamari.get_center(),
            &prince.get_pos(),
        );
        prince_to_kat[1] = 0.0;
        vec3_inplace_normalize(&mut prince_to_kat);

        if katamari.physics_flags.under_water {
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
            vec3::scale_and_add(&mut target, &kat_center, &prince_to_kat, target_offset);

            let pos_offset = self.state.kat_to_pos[2];
            let mut kat_to_cam_pos = vec3::create();
            vec3::scale(&mut kat_to_cam_pos, &prince_to_kat, pos_offset);

            // update special camera state before computing camera position
            self.state.use_special_camera = if !katamari.hit_flags.special_camera
                && !katamari.last_hit_flags[0].special_camera
            {
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
            vec3::add(&mut pos, &kat_center, &kat_to_cam_pos);

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
        self.transform.update();
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
        *xx = self.transform.lookat_rot_inv[0];
        *xy = self.transform.lookat_rot_inv[1];
        *xz = self.transform.lookat_rot_inv[2];
        *yx = self.transform.lookat_rot_inv[4];
        *yy = self.transform.lookat_rot_inv[5];
        *yz = self.transform.lookat_rot_inv[6];
        *zx = self.transform.lookat_rot_inv[8];
        *zy = self.transform.lookat_rot_inv[9];
        *zz = self.transform.lookat_rot_inv[10];

        *tx = self.transform.pos[0] / UNITY_TO_SIM_SCALE;
        *ty = self.transform.pos[1] / UNITY_TO_SIM_SCALE;
        *tz = self.transform.pos[2] / UNITY_TO_SIM_SCALE;

        *offset = self.transform.mas4_preclear_offset;
    }
}
