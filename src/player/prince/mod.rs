use std::f32::consts::{FRAC_PI_2, PI};

use gl_matrix::{
    common::{Mat4, Vec3},
    mat4::{self},
    vec3,
};

use crate::{
    constants::{UNITY_TO_SIM_SCALE, VEC3_ZERO},
    macros::{inv_lerp, inv_lerp_clamp, lerp, max, min, panic_log, set_y},
    math::{
        acos_f32, change_bounded_angle, normalize_bounded_angle, vec3_inplace_add_vec,
        vec3_inplace_normalize,
    },
    mission::{state::MissionState, tutorial::TutorialMove, GameMode},
    player::{
        camera::{mode::CameraMode, Camera},
        input::{AnalogPushDirs, GachaDir, Input, StickInput, StickPushDir},
        katamari::Katamari,
        Player,
    },
};

use self::params::PrinceParams;

mod params;

#[derive(Debug, Default, Clone, Copy)]
pub struct OujiState {
    pub dash_start: bool,
    pub wheel_spin_start: bool,
    pub dash: bool,
    pub wheel_spin: bool,
    pub jump_180: bool,
    pub sw_speed_disp: bool,
    pub climb_wall: bool,
    pub huff: bool,
    pub camera_mode: u8,
    pub dash_effect: bool,
    pub hit_water: bool,
    pub submerge: bool,

    /// A copy of the camera's R1 jump state.
    pub camera_state: u8,

    pub jump_180_leap: u8,
    pub brake: bool,
    pub tutorial_flag_1: u8,
    pub tutorial_flag_2: u8,
    pub tutorial_trigger_1: u8,
    pub tutorial_trigger_2: u8,
    pub power_charge: u8,  // apparently unused
    pub fire_to_enemy: u8, // apparently unused
    pub search: u8,
    pub attack_1: u8,
    pub attack_2: u8,
    pub tarai: u8,
    pub attack_wait: u8,
    pub vs_attack: bool,
}

impl OujiState {
    pub fn end_boost(&mut self) {
        self.dash_start = false;
        self.dash = false;
        self.wheel_spin_start = false;
        self.wheel_spin = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrinceViewMode {
    Normal = 0,
    R1Jump = 1,
    L1Look = 2,
}

impl Default for PrinceViewMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// The directions that the prince can push the katamari.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushDir {
    Forwards,
    Backwards,
    Sideways,
}

/// The directions that the prince can push the katamari sideways.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrinceSidewaysDir {
    Left,
    Right,
}

/// Classifies the ways the prince can be turning around the katamari.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrinceTurnType {
    None,

    /// Left stick up, right stick neutral.
    LeftStickUp,

    /// Right stick up, left stick neutral.
    RightStickUp,

    // Left stick down, right stick neutral.
    LeftStickDown,

    /// Right stick down, left stick neutral.
    RightStickDown,

    /// R1 flip in progress.
    Flip,

    /// Both sticks are non-neutral.
    BothSticks,
}

impl Default for PrinceTurnType {
    fn default() -> Self {
        Self::None
    }
}

/// State pertaining to the prince. It's a little arbitrary as far as what's in `Prince` versus
/// what's in `Katamari`, but this struct notably includes player input.
/// offset: 0xd33210
/// width: 0x518
#[derive(Debug, Default)]
pub struct Prince {
    params: PrinceParams,

    /// The player index controlling this prince.
    /// offset: 0x0
    player: u8,

    /// (??) Random bit vector.
    /// offset: 0x4
    flags: u32,

    /// The current position of the prince.
    /// offset: 0xc
    pos: Vec3,

    /// The position of the prince on the previous tick.
    /// offset: 0x1c
    last_pos: Vec3,

    /// The offset of the prince from the bottom of the katamari, in prince local space.
    /// When not flipping this is a copy of `base_kat_offset`, but flipping results in
    /// this offset deviating from the base offset as the prince jumps over the katamari
    /// in a semicircle.
    /// offset: 0x3c
    kat_offset: Vec3,

    /// The prince's unit lateral offset from the katamari at the start of a flip.
    /// offset: 0x5c
    flip_lateral_kat_offset_unit: Vec3,

    /// The prince's angle around the katamari.
    /// offset: 0x6c
    angle: f32,

    /// Value added to `angle` each tick.
    /// offset: 0x70
    extra_flat_angle_speed: f32,

    /// (??)
    /// offset: 0x74
    auto_rotate_right_speed: f32,

    /// The speed at which the prince is rotating around the katamari.
    /// offset: 0x78
    angle_speed: f32,

    /// The remaining ratio of the duration of the current huff.
    /// (Starts at 1 at the beginning of the huff, huff ends at 0).
    /// offset: 0x90
    huff_timer_ratio: f32,

    /// The prince's view mode, which distinguishes being in the R1 and L1 states.
    /// offset: 0x9c
    view_mode: PrinceViewMode,

    /// (??) Seems to be a vs-mode flag related to huffing? who cares
    /// offset: 0x9d
    vs_mode_huff_related_flag: bool,

    /// True if the prince is huffing
    /// offset: 0x9e
    is_huffing: bool,

    /// Bit flags: &1=left stick pushed, &2=right stick pushed.
    /// offset: 0x9f
    sticks_pushed: u8,

    /// If true, on the second half of the flip where the prince is falling.
    /// offset: 0xa0
    falling_from_flip: bool,

    /// If true, the prince is rotating quickly around the katamari
    /// (e.g. left stick up AND right stick down)
    /// offset: 0xa1
    is_quick_shifting: bool,

    /// Various 1-byte fields that are shared with the Unity code.
    /// offset: 0xa2
    pub oujistate: OujiState,

    /// The previous frame's values of various 1-byte fields that are shared with the Unity code.
    /// offset: 0xbd
    last_oujistate: OujiState,

    /// If >0, ignores player input, and decrements by 1 each frame.
    /// If <0, ignores player input forever until changed.
    /// If 0, player input is read as usual.
    /// offset: 0xf4
    ignore_input_timer: i16,

    /// The transform matrix of the prince.
    /// Note that this is only a rotation matrix; it doesn't include translation.
    /// offset: 0x138
    transform_rot: Mat4,

    /// The default offset of the prince from the katamari center in local katamari space.
    /// The prince-katamari distance scales with the katamari size.
    /// Since the prince is always facing the z+ axis, this vector is therefore usually
    /// a multiple of the z- axis (since the prince is behind the katamari center).
    /// offset: 0x24c
    base_kat_offset: Vec3,

    /// The threshold on the angle between the two analog sticks that differentiates
    /// "rolling fowards/backwards" and "rolling sideways". Whatever that means.
    /// offset: 0x288
    pub push_sideways_angle_threshold: f32,

    /// The prince's turn speed while not moving backwards.
    /// offset: 0x28c
    non_backwards_turn_speed: f32,

    /// The prince's turn speed while moving backwards.
    /// offset: 0x290
    backwards_turn_speed: f32,

    /// The prince's turn speed while turning exactly one stick up.
    /// offset: 0x29c
    one_stick_up_turn_speed: f32,

    /// The prince's turn speed while turning exactly one stick down.
    /// offset: 0x2a8
    one_stick_down_turn_speed: f32,

    /// The prince's turn speed while turning with both sticks.
    /// offset: 0x2ac
    quick_shift_turn_speed: f32,

    /// The minimum angle between the two sticks necessary to cap turn speed.
    /// offset: 0x2b0
    angle_btwn_sticks_for_fastest_turn: f32,

    /// (??)
    /// offset: 0x2bc
    forward_push_angle_cutoff: f32,

    /// (??)
    /// offset: 0x2c0
    backward_push_angle_cutoff: f32,

    /// A forward push of this value or higher is scaled to 1, with lower values rescaled between [0,1].
    /// offset: 0x2c4
    forward_push_cap: f32,

    /// The minimum push with both sticks to start moving.
    /// offset: 0x2c8
    min_push_to_move: f32,

    /// The number of ticks it takes to complete the flip animation.
    /// offset: 0x2d0
    flip_duration: u16,

    /// The
    /// offset: 0x2d4
    max_analog_allowing_flip: f32,

    /// The number of ticks allowed between gachas before the gacha count resets.
    /// offset: 0x2dc
    gacha_window_duration: u16,

    /// The number of gachas needed to initiate a spin.
    /// offset: 0x2e0
    gachas_for_spin: u16,

    /// The maximum boost energy.
    /// offset: 0x2d4
    boost_max_energy: u16,

    /// The amount of boost energy gained per recharge.
    /// offset: 0x2ec
    boost_recharge: u16,

    /// The number of ticks between boost recharges (assuming the player doesn't spin and
    /// reset the timer)
    /// offset: 0x2f0
    boost_recharge_frequency: u16,

    /// The duration of a huff, in ticks.
    /// offset: 0x2f4
    huff_duration: u16,

    /// The initial multiplier on katamari speed during a huff (this penalty decays as the
    /// huff gets closer to ending)
    /// offset: 0x2f8
    huff_init_speed_penalty: f32,

    /// (??)
    /// offset: 0x300
    max_push_uphill_strength: f32,

    /// (??)
    /// offset: 0x304
    uphill_strength_loss: f32,

    /// Exact left stick analog input.
    /// offset: 0x318
    input_ls: StickInput,

    /// Exact right stick analog input.
    /// offset: 0x328
    input_rs: StickInput,

    /// Exact unit left stick analog input.
    /// offset: 0x338
    input_ls_unit: StickInput,

    /// Exact unit right stick analog input.
    /// offset: 0x348
    input_rs_unit: StickInput,

    /// The normalized sum `input_ls + input_rs`.
    /// offset: 0x358
    input_sum_unit: StickInput,

    /// The absolute value of `input_ls` (on both axes, independently).
    /// offset: 0x368
    input_ls_abs: StickInput,

    /// The absolute value of `input_rs` (on both axes, independently).
    /// offset: 0x378
    input_rs_abs: StickInput,

    /// The average of `input_ls` and `input_rs` (unused, apparently)
    /// offset: 0x398
    // input_avg: StickInput,

    /// The magnitude of the `input_ls` input.
    /// offset: 0x3a8
    input_ls_len: f32,

    /// The magnitude of the `input_rs` input.
    /// offset: 0x3ac
    input_rs_len: f32,

    /// The average of `input_rs_len` and `input_ls_len`.
    /// offset: 0x3b0
    input_avg_len: f32,

    /// The angle of the left stick relative to the y+ axis.
    /// offset: 0x3b4
    input_ls_angle: f32,

    /// The angle of the right stick relative to the y+ axis.
    /// offset: 0x3b8
    input_rs_angle: f32,

    /// The difference between the angles of the two sticks.
    /// offset: 0x3bc
    input_angle_btwn_sticks: f32,

    /// (??) too lazy
    /// offset: 0x3c0
    pub input_avg_push_len: f32,

    /// 0 if moving sideways, [0,1] if forwards/backwards depending on y angle of net input
    /// offset: 0x3c4
    push_strength: f32,

    /// The matrix encoding a y-axis rotation by the input push angle.
    /// offset: 0x3cc
    nonboost_push_yaw_rot: Mat4,

    /// (??) The value of the `0x3cc` matrix when not boosting, otherwise it's the identity matrix.
    /// offset: 0x40c
    boost_push_yaw_rot: Mat4,

    /// The number of ticks remaining in the current flip animation.
    /// offset: 0x44c
    flip_timer: u16,

    /// Input push directions which just changed this tick.
    /// offset: 0x470
    push_dirs_changed: AnalogPushDirs,

    /// Current input push directions.
    /// offset: 0x472
    push_dirs: AnalogPushDirs,

    /// Input push directions on the previous tick.
    /// offset: 0x474
    last_push_dirs: AnalogPushDirs,

    /// The previous gacha direction input.
    /// offset: 0x476
    last_gacha_direction: Option<GachaDir>,

    /// The remaining ticks before the gacha count resets.
    /// offset: 0x478
    gacha_window_timer: u16,

    /// True if a gacha has been performed since the last spin.
    /// (Reset to false after every spin)
    /// offset: 0x47a
    did_gacha_since_last_spin: bool,

    /// The number of gachas counted without the gacha timer expiring.
    /// offset: 0x47c
    gacha_count: u8,

    /// The amount of remaining boost energy.
    /// offset: 0x47e
    boost_energy: u16,

    /// The number of ticks since the katamari was spun. Used for boost recharging.
    /// offset: 0x484
    no_dash_ticks: u16,

    /// The remaining ticks in the current huff.
    /// offset: 0x486
    huff_timer: u16,

    /// The strength with which the katamari can be pushed uphill. Decreases while
    /// pushing uphill. (Seems to start at 100 and decrease from there, so maybe it's a percentage)
    /// offset: 0x488
    push_uphill_strength: f32,

    /// The direction the prince is pushing the katamari, if any.
    /// offset: 0x48c
    push_dir: Option<PushDir>,

    /// The direction the prince is pushing the katamari sideways, if any.
    /// offset: 0x48d
    push_sideways_dir: Option<PrinceSidewaysDir>,

    /// The type of turning around the katamari the prince is doing.
    /// offset: 0x48e
    turn_type: PrinceTurnType,

    /// If true, the lock caused by large collision impacts is active.
    /// offset: 0x490
    impact_lock_active: bool,

    /// (??) the impact lock velocity at which the impact lock ends
    /// offset: 0x494
    impact_lock_final_vel: Vec3,

    /// (??) the initial impact lock velocity, imparted by the colliding prop
    /// offset: 0x4a4
    impact_lock_init_vel: Vec3,
}

impl Prince {
    pub fn set_global_turn_speed(&mut self, value: f32) {
        self.params.global_turn_speed_mult = value;
    }

    pub fn get_pos(&self) -> &Vec3 {
        &self.pos
    }

    pub fn get_angle(&self) -> f32 {
        self.angle
    }

    pub fn get_oujistate(&self) -> OujiState {
        self.oujistate.clone()
    }

    pub fn get_push_dir(&self) -> Option<PushDir> {
        self.push_dir
    }

    pub fn get_flags(&self) -> u32 {
        self.flags
    }

    pub fn get_angle_speed(&self) -> f32 {
        self.angle_speed
    }

    pub fn get_push_strength(&self) -> f32 {
        self.push_strength
    }

    pub fn get_input_avg_len(&self) -> f32 {
        self.input_avg_len
    }

    pub fn get_is_huffing(&self) -> bool {
        self.is_huffing
    }

    pub fn get_boost_push_yaw_rot(&self) -> &Mat4 {
        &self.boost_push_yaw_rot
    }

    pub fn get_nonboost_push_yaw_rot(&self) -> &Mat4 {
        &self.nonboost_push_yaw_rot
    }

    pub fn get_params(&self) -> &PrinceParams {
        &self.params
    }

    pub fn get_turn_type(&self) -> PrinceTurnType {
        self.turn_type
    }

    pub fn get_push_sideways_dir(&self) -> Option<PrinceSidewaysDir> {
        self.push_sideways_dir
    }

    pub fn get_is_quick_shifting(&self) -> bool {
        self.is_quick_shifting
    }

    pub fn get_input_sum_unit(&self) -> &StickInput {
        &self.input_sum_unit
    }

    pub fn get_flip_lateral_kat_offset_unit(&self) -> &Vec3 {
        &self.flip_lateral_kat_offset_unit
    }

    pub fn get_sticks_pushed(&self) -> u8 {
        self.sticks_pushed
    }

    /// Returns `true` if the current input state admits a wallclimb.
    /// (independently of all other things that could influence whether a wallclimb is allowed,
    /// up to and including whether the katamari even contacts a wall in the first place)
    pub fn has_wallclimb_input(&self) -> bool {
        self.input_avg_push_len >= self.params.wallclimb_min_avg_push_len
            && self.input_sum_unit.angle().abs() < self.params.wallclimb_input_sum_angle_threshold
    }

    /// Returns the max speed reduction while huffing. This penalty eases off
    /// as the huff progresses, smoothly leading to into no penalty when the huff ends.
    /// offset: 0x226be (in `kat_update_velocity`)
    pub fn get_huff_speed_penalty(&self) -> f32 {
        if self.is_huffing {
            // TODO: this might not be right, double check
            lerp!(self.huff_timer_ratio, 1.0, self.huff_init_speed_penalty)
        } else {
            // if not huffing, there's no speed penalty
            1.0
        }
    }

    /// (??) Returns the accel (?) penalty while moving uphill.
    /// offset: 0x226fc (in `kat_update_velocity`)
    pub fn get_uphill_accel_penalty(&self) -> f32 {
        self.push_uphill_strength / self.max_push_uphill_strength
    }

    pub fn reset_push_uphill_strength(&mut self) {
        self.push_uphill_strength = self.max_push_uphill_strength;
    }

    pub fn decrease_push_uphill_strength(&mut self, t: f32) {
        self.push_uphill_strength -= t * self.uphill_strength_loss;
        if self.push_uphill_strength < 0.0 {
            self.push_uphill_strength = 0.0;
        }
    }

    /// If false, the input is not considered "pushing" the katamari for the purposes
    /// of `Katamari::compute_brake_state`.
    /// offset: 0x21962 (in `kat_compute_brake_state`)
    pub fn is_pushing_for_brake(&self) -> bool {
        self.input_angle_btwn_sticks <= self.params.max_angle_btwn_sticks_for_push
            && self.input_avg_push_len > 0.0
    }
}

impl Prince {
    /// Initialize the prince at the start of a mission.
    pub fn init(&mut self, player: u8, init_angle: f32, kat: &Katamari, camera: &Camera) {
        self.player = player;
        self.no_dash_ticks = 0;
        self.huff_timer = 0;
        vec3::copy(&mut self.pos, &VEC3_ZERO);
        self.auto_rotate_right_speed = 0.0;
        self.angle = init_angle;
        self.base_kat_offset[2] = kat.get_prince_offset();
        self.push_dir = None;
        self.angle_speed = 0.0;
        mat4::identity(&mut self.nonboost_push_yaw_rot);
        mat4::identity(&mut self.boost_push_yaw_rot);
        mat4::identity(&mut self.transform_rot);

        // TODO: make this a `PrinceParams` struct or something
        self.huff_init_speed_penalty = 0.4;
        self.huff_duration = 240; // TODO: some weird potential off-by-one issue here.
        self.max_push_uphill_strength = 100.0;
        self.uphill_strength_loss = 0.7649993;
        self.forward_push_angle_cutoff = 0.8733223;
        self.backward_push_angle_cutoff = 2.270252;
        self.forward_push_cap = 0.5;
        self.one_stick_up_turn_speed = 0.035;
        self.one_stick_down_turn_speed = 0.025;
        self.quick_shift_turn_speed = 0.055;
        self.backwards_turn_speed = 0.03;
        self.non_backwards_turn_speed = 0.06;
        self.max_analog_allowing_flip = 0.3;
        self.gacha_window_duration = 14;
        self.boost_max_energy = 0xf0;
        self.gachas_for_spin = 3;
        self.boost_recharge = 18;
        self.boost_recharge_frequency = 100;
        self.angle_btwn_sticks_for_fastest_turn = 0.75;
        self.push_sideways_angle_threshold = 0.363474;

        self.update_transform(kat, camera);

        self.boost_energy = self.boost_max_energy;
        self.push_uphill_strength = self.max_push_uphill_strength;
        self.view_mode = PrinceViewMode::Normal;
        self.ignore_input_timer = 0;

        // TODO: `prince_init:100-123` (vs mode crap)
    }

    /// Copy the prince's `OujiState` to the `oujistate` pointer (which is passed from unity).
    pub fn copy_oujistate_ptr(&mut self, oujistate: &mut *mut OujiState, data_size: &mut i32) {
        *data_size = 0x1b;
        *oujistate = &mut self.oujistate as *mut OujiState;
    }

    /// Get the prince's view mode.
    pub fn get_view_mode(&self) -> PrinceViewMode {
        self.view_mode
    }

    /// Set the prince's view mode.
    pub fn set_view_mode(&mut self, view_mode: PrinceViewMode) {
        self.view_mode = view_mode;
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
    ) -> () {
        *xx = self.transform_rot[0];
        *xy = self.transform_rot[1];
        *xz = self.transform_rot[2];
        *yx = self.transform_rot[4];
        *yy = self.transform_rot[5];
        *yz = self.transform_rot[6];
        *zx = self.transform_rot[8];
        *zy = self.transform_rot[9];
        *zz = self.transform_rot[10];
        *tx = self.pos[0] / UNITY_TO_SIM_SCALE;
        *ty = self.pos[1] / UNITY_TO_SIM_SCALE;
        *tz = self.pos[2] / UNITY_TO_SIM_SCALE;
    }

    pub fn set_ignore_input_timer(&mut self, value: i16) {
        self.ignore_input_timer = value;
    }
}

impl Prince {
    /// Read player input from the `GameState`.
    /// offset: 0x54160
    pub fn read_input(
        &mut self,
        input: &Input,
        mission_state: &mut MissionState,
        katamari: &Katamari,
        camera: &mut Camera,
    ) {
        // update whether the sticks are being pushed at all
        self.sticks_pushed = 0;
        if input.ls_x != 0 || input.ls_y != 0 {
            self.sticks_pushed |= 1;
        }
        if input.rs_x != 0 || input.rs_y != 0 {
            self.sticks_pushed |= 2;
        }

        // update analog input
        let ignore_input_timer = self.ignore_input_timer;
        if ignore_input_timer == 0 {
            input.dequantize(&mut self.input_ls, &mut self.input_rs);
        } else {
            self.input_ls.clear();
            self.input_rs.clear();
        }

        if self.should_init_flip(input) {
            // TODO_FX: play flip sound

            if mission_state.is_tutorial() {
                if let Some(tutorial_state) = &mut mission_state.tutorial {
                    tutorial_state.set_move_held(TutorialMove::Flip);
                }
            }

            self.oujistate.jump_180 = true;
            self.view_mode = PrinceViewMode::Normal;
            self.ignore_input_timer = 0;
            camera.set_mode_normal();
            self.flip_timer = self.flip_duration;

            let kat_center = katamari.get_center();
            vec3::subtract(
                &mut self.flip_lateral_kat_offset_unit,
                kat_center,
                &self.pos,
            );
            set_y!(self.flip_lateral_kat_offset_unit, 0.0);
            vec3_inplace_normalize(&mut self.flip_lateral_kat_offset_unit);
        }

        // update absolute analog input
        self.input_ls.absolute(&mut self.input_ls_abs);
        self.input_rs.absolute(&mut self.input_rs_abs);
    }

    /// Returns `true` if the prince should begin a flip by checking if both
    /// analog sticks are pressed.
    /// offset: 0x544b0
    fn should_init_flip(&mut self, input: &Input) -> bool {
        // TODO: more early returns
        // if katamari.physics_flags.vs_mode_state == 2 { return false; }
        // if katamari.hysics_flags.airborne { return false; }
        // if gamemode == 3 || gamemode == 4 { return false; }
        // if gamemode == Tutorial && tutorial_flags.page == 0 { return false; }

        if self.view_mode == PrinceViewMode::R1Jump || self.oujistate.jump_180 {
            return false;
        }
        // this is apparently an airborne flag
        if self.flags & 0x100 == 0 {
            return false;
        }

        // both sticks are considered down if one stick was already held
        let both_sticks_down = (input.l3_held && input.r3_down) || (input.r3_held && input.l3_down);
        if !both_sticks_down {
            return false;
        }

        // once both sticks are down, we check the analog magnitudes to make sure they're within
        // the maximum magnitude allowing a flip
        let max = self.max_analog_allowing_flip;
        let ls_ok = self.input_ls.axes[0].abs() <= max && self.input_ls.axes[1].abs() <= max;
        let rs_ok = self.input_rs.axes[0].abs() <= max && self.input_rs.axes[1].abs() <= max;

        ls_ok && rs_ok
    }

    /// Update the prince's huff state.
    /// offset: 0x547f0 (first half of the function)
    fn update_huff(&mut self) {
        self.huff_timer_ratio = if self.huff_timer == 0 {
            0.0
        } else {
            self.huff_timer -= 1;
            self.huff_timer as f32 / self.huff_duration as f32
        };

        self.is_huffing = self.huff_timer != 0;
        self.oujistate.huff = self.huff_timer != 0;
    }

    /// Decide if the current non-normal view mode should be ended.
    /// If so, end it.
    /// offset: 0x547f0 (second half of function)
    fn try_end_view_mode(&mut self, camera: &mut Camera) {
        let should_end = match self.view_mode {
            PrinceViewMode::R1Jump => {
                if camera.preclear.get_enabled() {
                    // if in preclear mode, end the jump immediately
                    return self.end_view_mode(None);
                } else {
                    // if not in preclear mode, end the jump if the camera mode isn't R1 anymore
                    camera.get_mode() == CameraMode::Normal
                }
            }
            PrinceViewMode::L1Look => {
                if camera.preclear.get_enabled() {
                    // if in preclear mode, end the look immediately.
                    return self.end_view_mode(Some(camera));
                } else {
                    // if not in preclear mode, end the look if both sticks are pushed.
                    self.sticks_pushed & 3 == 3
                }
            }
            PrinceViewMode::Normal => {
                return;
            }
        };

        if should_end {
            self.end_view_mode(Some(camera));
        }
    }

    /// Reset the current view mode back to `Normal`.
    /// If a camera reference is passed, sets the camera mode back to normal as well.
    fn end_view_mode(&mut self, camera: Option<&mut Camera>) {
        self.view_mode = PrinceViewMode::Normal;
        self.ignore_input_timer = 0;
        camera.map(|camera| camera.set_mode_normal());
    }

    fn update_boost_recharge(&mut self) {
        if !self.oujistate.dash {
            // if we aren't dashing, increment `no_dash_ticks`:
            self.no_dash_ticks += 1;
            if self.no_dash_ticks >= self.boost_recharge_frequency {
                // if we haven't spun for long enough to recharge, do the recharge:
                self.no_dash_ticks = 0;
                self.boost_energy = max!(
                    self.boost_energy + self.boost_recharge,
                    self.boost_max_energy
                );
            }
        } else {
            // if we are dashing, reset `no_dash_ticks` to 0:
            self.no_dash_ticks = 0;
        }
    }

    /// **After** `read_input`, compute various features of analog input (some of which are
    /// also expressed as analog input, e.g. unit input).
    /// offset: 0x53cc0 (first half of `prince_update_boost`)
    fn update_analog_input_features(&mut self) {
        self.input_ls.normalize(&mut self.input_ls_unit);
        self.input_rs.normalize(&mut self.input_rs_unit);
        StickInput::normalize_sum(&mut self.input_sum_unit, &self.input_ls, &self.input_rs);

        self.input_ls_angle = self.input_ls_unit.angle();
        self.input_rs_angle = self.input_rs_unit.angle();
        self.input_angle_btwn_sticks = self.input_ls_unit.angle_with_other(&self.input_rs_unit);

        self.input_ls_len = min!(1.0, self.input_ls.len());
        self.input_rs_len = min!(1.0, self.input_rs.len());
        self.input_avg_len = (self.input_ls_len + self.input_rs_len) / 2.0;

        // this is reset here and computed in `update_angle`, i guess because it depends more complicated stuff
        self.input_avg_push_len = 0.0;

        if self.view_mode == PrinceViewMode::Normal {
            self.last_push_dirs = self.push_dirs;

            // TODO: extract as sim param, also it's different in vs mode or whatever
            let min_push_len = 0.35;

            if !self.vs_mode_huff_related_flag {
                let ls_y = self.input_ls.y();
                let rs_y = self.input_rs.y();

                self.push_dirs.update_from_input(ls_y, rs_y, min_push_len);
                self.push_dirs_changed
                    .compute_changed(&self.last_push_dirs, &self.push_dirs);
            } else {
                self.push_dirs.clear();
                self.push_dirs_changed.clear();
            }
        }
    }

    /// Update the prince's gacha count based on input that should have been processed earlier in the tick.
    /// Also initiates SFX and VFX for boosting and spinning.
    /// offset: 0x566d0
    fn update_gachas(
        &mut self,
        katamari: &mut Katamari,
        camera: &Camera,
        mission_state: &mut MissionState,
    ) {
        // TODO_VS: this whole function only applies to single player. would need to be rewritten for vs mode

        // use a different gacha updating strategy while huffing
        if self.is_huffing {
            return self.update_gachas_while_huffing(katamari, mission_state.is_vs_mode);
        }

        if self.oujistate.wheel_spin == false && katamari.physics_flags.airborne {
            self.end_spin_and_boost(katamari);
        }

        self.oujistate.dash_start = false;
        self.oujistate.wheel_spin_start = false;

        if camera.get_mode() == CameraMode::Shoot {
            return self.oujistate.dash = true;
        }

        // early check to block gachas based on the gamemode
        let block_gachas = match mission_state.gamemode {
            GameMode::Normal => false,
            GameMode::Tutorial => mission_state.tutorial.as_ref().unwrap().get_page() == 0,
            _ => true,
        };
        if block_gachas {
            return;
        }

        let gacha_window = self.gacha_window_duration;

        // decrement boost energy, but not in the tutorial
        if self.oujistate.dash && mission_state.gamemode != GameMode::Tutorial {
            self.boost_energy -= 1;
            if self.boost_energy == 0 {
                self.reset_boost_state(katamari);
                self.huff_timer = self.huff_duration;
                self.is_huffing = true;
                self.vs_mode_huff_related_flag = true;
                return;
            }
        }

        // Compute the new gacha direction, if one exists.
        let change = self.push_dirs_changed;
        let push = self.push_dirs;
        let new_gacha =
            // left stick push dir just changed to the opposite of the right stick push dir
            (change.left != None && push.right != None && change.left != push.right) ||
            // right stick push dir just changed to the opposite of the left stick push dir
            (change.right != None && push.left != None && change.right != push.left);

        let new_gacha_dir = if !new_gacha {
            None
        } else if push.right == Some(StickPushDir::Down) {
            Some(GachaDir::Right)
        } else {
            Some(GachaDir::Left)
        };

        let mut just_did_gacha = false;
        if let Some(gacha_dir) = new_gacha_dir {
            // if a gacha just occurred:
            just_did_gacha = true;
            self.last_gacha_direction = Some(gacha_dir);
            self.gacha_window_timer = gacha_window;
            self.gacha_count += 1;
            // TODO_VS: check `should_reset_gachas_in_vs_mode()`
        }

        // TODO_VS: `prince_update_gachas:154-170`

        if self.gacha_window_timer > 0 {
            // if there are gachas in progress and the gacha timer hasn't expired:
            self.gacha_window_timer -= 1;

            let gachas_for_spin = self.params.prince_gachas_for_spin;
            let gachas_for_boost = self.params.gachas_for_boost(katamari.get_diam_cm());

            // TODO_VS: `prince_update_gachas:177-211`

            if just_did_gacha && self.gacha_count == gachas_for_boost {
                // if initiating a boost:
                // TODO: `prince_update_gachas:234-241` (play boost sfx and vfx)
                return;
            }

            if just_did_gacha && self.gacha_count == gachas_for_spin {
                // if initiating a spin:
                self.oujistate.wheel_spin_start = true;
            }

            if self.gacha_count >= gachas_for_spin && self.gacha_count < gachas_for_boost {
                // if spinning, but not yet enough gachas for a boost:
                // TODO: `prince_update_gachas:249-253` (play spin sfx)
                self.oujistate.dash = true;
                self.oujistate.wheel_spin = true;
            } else if self.gacha_count > gachas_for_boost {
                // if enough gachas for a boost:
                self.oujistate.dash = true;
                self.oujistate.wheel_spin = false;
            }

            if !self.oujistate.wheel_spin
                && self.oujistate.dash
                && mission_state.gamemode == GameMode::Tutorial
            {
                mission_state
                    .tutorial
                    .as_mut()
                    .unwrap()
                    .set_move_held(TutorialMove::Boost);
            }
            return;
        } else {
            // if the gacha timer has expired:

            // TODO_VS: `prince_update_gachas:275-316` (vs mode crap)
            // TODO: `prince_update_gachas:318-329` (update `did_gacha_since_last_spin`, probably a no-op)
            self.gacha_count = 0;
            self.oujistate.dash = false;
            self.oujistate.wheel_spin = false;
        }
    }

    /// Exit spin/boost state and reset gachas.
    /// offset: 0x56600
    pub fn end_spin_and_boost(&mut self, katamari: &mut Katamari) {
        self.oujistate.end_boost();
        self.gacha_count = 0;
        katamari.physics_flags.wheel_spin = false;
    }

    /// offset: 0x56650
    fn reset_boost_state(&mut self, katamari: &mut Katamari) {
        // TODO: vs mode crap; just look at the function
        self.end_spin_and_boost(katamari);
        self.boost_energy = self.boost_max_energy;
        self.gacha_window_timer = 0;
    }

    /// Update gachas while huffing.
    /// offset: 0x56e60
    fn update_gachas_while_huffing(&mut self, katamari: &mut Katamari, is_vs_mode: bool) {
        if !is_vs_mode {
            self.oujistate.end_boost();
            self.gacha_count = 0;
            katamari.physics_flags.wheel_spin = false;
        }
        // TODO_VS: `prince_update_gachas_while_huffing:13-18` (vs mode crap)
    }

    /// Update the prince's angle around the katamari.
    /// If the katamari is performing a tutorial move, that move is returned.
    /// offset: 0x55b70
    fn update_angle(&mut self, katamari: &Katamari) -> Option<TutorialMove> {
        let min_push = self.min_push_to_move;
        let mut tut_move_held = None;

        // turn off `self.flags & 0x40000`
        self.flags &= 0xfffbffff;

        if self.input_avg_len <= 0.0 || katamari.physics_flags.vs_mode_state == 2 {
            // if no analog input:
            self.angle_speed = 0.0;
            self.is_quick_shifting = false;
            self.turn_type = PrinceTurnType::None;
            self.input_avg_push_len = 0.0;
            return None;
        }

        // if there is at least some input, compute the `turn_type` from stick inputs, with six possible cases:
        if self.input_ls_len == 0.0 {
            // if left stick neutral:
            if self.input_rs_len == 0.0 {
                // case 1 (left stick neutral, right stick neutral)
                // i guess this shouldn't happen because it would have been detected above, but i'm not sure
                self.turn_type = PrinceTurnType::None;
                self.input_avg_len = 0.0;
                self.input_avg_push_len = 0.0;
                panic_log!("weird edge case in `update_angle`");
            } else if self.input_rs.y() > 0.0 {
                // case 2 (left stick neutral, right stick up)
                self.turn_type = PrinceTurnType::RightStickUp;
                self.angle_speed =
                    -self.one_stick_up_turn_speed * inv_lerp!(self.input_rs_abs.y(), min_push, 1.0);
                change_bounded_angle(&mut self.angle, self.angle_speed);
            } else {
                // case 3 (left stick neutral, right stick down)
                self.turn_type = PrinceTurnType::RightStickDown;
                self.angle_speed = self.one_stick_down_turn_speed
                    * 0.7
                    * inv_lerp!(self.input_rs_abs.y(), min_push, 1.0);
                change_bounded_angle(&mut self.angle, self.angle_speed);
            }
        } else if self.input_rs_len == 0.0 {
            if self.input_ls.y() > 0.0 {
                // case 4 (right stick neutral, left stick up)
                self.turn_type = PrinceTurnType::LeftStickUp;
                self.angle_speed =
                    self.one_stick_up_turn_speed * inv_lerp!(self.input_ls_abs.y(), min_push, 1.0);
                change_bounded_angle(&mut self.angle, self.angle_speed);
            } else {
                // case 5 (right stick neutral, left stick down)
                self.turn_type = PrinceTurnType::LeftStickDown;
                self.angle_speed = -self.one_stick_down_turn_speed
                    * 0.7
                    * inv_lerp!(self.input_ls_abs.y(), min_push, 1.0);
                change_bounded_angle(&mut self.angle, self.angle_speed);
            }
        } else {
            // case 6 (neither stick neutral)
            self.turn_type = PrinceTurnType::BothSticks;

            // within this case, there are 6 subcases depending on the angle between the two analog sticks.
            let arms_angle = self.input_angle_btwn_sticks;

            if arms_angle >= FRAC_PI_2 {
                // case 6.1 ("quick shifting": angle between sticks in [pi/2, pi])
                if katamari.get_speed() > 0.0 {
                    self.flags |= 0x40000;
                    self.input_avg_push_len = 0.0;
                }

                // TODO: move this to prince params
                self.update_angle_from_quick_shift(self.params.global_turn_speed_mult);
                return Some(TutorialMove::QuickShift);
            }

            let ls_push_len = inv_lerp_clamp!(self.input_ls_len, min_push, 1.0);
            let rs_push_len = inv_lerp_clamp!(self.input_rs_len, min_push, 1.0);
            self.input_avg_push_len = (ls_push_len + rs_push_len) * 0.5;

            let push_angle_len = acos_f32(self.input_sum_unit.y());
            let mut push_angle = self.input_sum_unit.angle();
            self.push_sideways_dir = None;

            if push_angle_len < self.params.prince_roll_forwards_angle_threshold {
                // case 6.2 ("roll forwards": push angle is below the threshold for rolling forwards)
                tut_move_held = Some(TutorialMove::RollForwards);
            } else if push_angle_len < FRAC_PI_2 - self.push_sideways_angle_threshold {
                if push_angle >= 0.0 {
                    // case 6.3 ("roll to the right")
                    tut_move_held = Some(TutorialMove::RollToTheRight);
                } else {
                    // case 6.4 ("roll to the left")
                    tut_move_held = Some(TutorialMove::RollToTheLeft);
                }
            } else if push_angle_len > FRAC_PI_2 + self.push_sideways_angle_threshold {
                // case 6.5 ("roll backwards": push angle is above the threshold for rolling backwards)
                tut_move_held = Some(TutorialMove::RollBackwards);
            } else {
                // case 6.6 ("roll sideways": push angle in `[pi/2 - t, pi/2 + t]`,
                //           where `t` is the push sideways angle threshold.)
                tut_move_held = Some(TutorialMove::RollSideways);

                // whether we're rolling sideways left or right is determined by the
                // sign of the push-input's x axis (which is identical to the sign
                // of the `push_angle`).
                if push_angle >= 0.0 {
                    push_angle = FRAC_PI_2;
                    self.push_sideways_dir = Some(PrinceSidewaysDir::Right)
                } else {
                    push_angle = -FRAC_PI_2;
                    self.push_sideways_dir = Some(PrinceSidewaysDir::Left);
                }
            }

            let id = mat4::create();
            mat4::rotate_y(&mut self.nonboost_push_yaw_rot, &id, push_angle);
            self.update_angle_from_push(self.params.global_turn_speed_mult, push_angle_len);
        }

        tut_move_held
    }

    /// Update the prince's angle around the katamari when quick shifting.
    /// offset: 0x56510
    fn update_angle_from_quick_shift(&mut self, global_speed_mult: f32) {
        let ls_y = self.input_ls_unit.y();
        let ls_y_sign = ls_y.signum();

        let rs_y = self.input_rs_unit.y();
        let rs_y_sign = rs_y.signum();

        // don't quick shift if the two sticks have the same y axis sign.
        if ls_y_sign == rs_y_sign {
            return;
        }

        let base_turn_speed = ls_y_sign
            * inv_lerp_clamp!(
                self.input_angle_btwn_sticks,
                FRAC_PI_2,
                self.angle_btwn_sticks_for_fastest_turn * PI
            );

        self.is_quick_shifting = true;
        self.angle_speed = base_turn_speed * self.quick_shift_turn_speed;
        change_bounded_angle(&mut self.angle, global_speed_mult * self.angle_speed);
    }

    /// Update the prince's angle around the katamari when pushing (i.e.
    /// whenever neither stick is neutral).
    /// `push_angle` is the absolute value of the angle between the sticks.
    /// offset: 0x56250
    fn update_angle_from_push(&mut self, global_speed_mult: f32, push_angle: f32) {
        let forw_cutoff = self.forward_push_angle_cutoff;
        let forw_max = FRAC_PI_2 - self.push_sideways_angle_threshold;
        let back_min = FRAC_PI_2 + self.push_sideways_angle_threshold;
        let back_cutoff = self.backward_push_angle_cutoff;

        // compute the first speed component from the angle between the analog
        // stick inputs.
        let mut push_angle_speed = if push_angle < forw_cutoff {
            // case 1: angles in `[0, forw_cutoff]` -> speed in `[0, 1]`
            push_angle / forw_cutoff
        } else if push_angle <= forw_max {
            // case 2: angles in `[forw_cutoff, forw_max]` -> speed in `[0, 1]`
            inv_lerp!(push_angle, forw_cutoff, forw_max)
        } else if push_angle <= back_min {
            // case 3: angles in `[forw_max, back_min]` -> speed `0`
            0.0
        } else if push_angle <= back_cutoff {
            // case 4: angles in `[back_min, back_cutoff]` -> speed in `[0, -1]`
            1.0 - inv_lerp!(push_angle, back_min, back_cutoff)
        } else {
            // case 5: angles in `[back_cutoff, pi]` -> speed in `[0, -1]`
            1.0 - inv_lerp!(push_angle, back_cutoff, PI)
        };

        // the sign of the push angle speed is determined by the x axis of the analog input
        if self.input_sum_unit.x() < 0.0 {
            push_angle_speed *= -1.0;
        }

        // compute the second speed component from the difference between the
        // stick input magnitudes.
        let mut stick_mag_speed = self.input_ls_len - self.input_rs_len;
        if push_angle > FRAC_PI_2 {
            stick_mag_speed *= -1.0;
        }

        // compute the base angle speed depending on if the prince is pushing backwards or not
        let base_angle_speed = if push_angle >= back_min {
            self.backwards_turn_speed
        } else {
            self.non_backwards_turn_speed
        };

        // compute angle speed from the above two speed components, then update the angle
        self.angle_speed = 0.5 * base_angle_speed * (push_angle_speed + stick_mag_speed);
        change_bounded_angle(&mut self.angle, global_speed_mult * self.angle_speed);

        // compute the push direction and push strength
        if push_angle <= forw_max {
            // case 1: angle in `[0, forw_max]` (pushing forwards):
            self.push_dir = Some(PushDir::Forwards);

            // TODO: simplify this, it should be a single clamped inverse lerp
            let scaled_str = (forw_max - push_angle) / forw_max;
            self.push_strength = if scaled_str < self.forward_push_cap {
                scaled_str / self.forward_push_cap
            } else {
                1.0
            };
        } else if push_angle < back_min {
            // case 2: angle in `[forw_max, back_min]` (pushing sideways):
            // when pushing sideways, we have zero push strength
            self.push_dir = Some(PushDir::Sideways);
            self.push_strength = 0.0;
        } else {
            // case 3: angle in `[back_min, pi]` (pushing backwards):
            self.push_dir = Some(PushDir::Backwards);
            self.push_strength = inv_lerp!(push_angle, back_min, PI);
        };
    }

    /// The bottom chunk of `prince_update_input_features_and_gachas` in ghidra,
    /// after `prince_update_angle` is called.
    pub fn update_boost_push_rotation_mat(&mut self, is_vs_mode: bool) {
        mat4::copy(&mut self.boost_push_yaw_rot, &self.nonboost_push_yaw_rot);
        if self.oujistate.dash {
            self.extra_flat_angle_speed = 0.0;
            if !is_vs_mode {
                mat4::identity(&mut self.nonboost_push_yaw_rot);
            }
        }
    }

    pub fn update_view_mode(
        &mut self,
        camera: &mut Camera,
        katamari: &mut Katamari,
        input: &Input,
        mission_state: &mut MissionState,
    ) -> Option<TutorialMove> {
        let mut held_tut_move: Option<TutorialMove> = None;

        match self.view_mode {
            PrinceViewMode::Normal => {
                if !camera.is_scaling_up() {
                    // TODO: `prince_update_trigger_actions:24-44` (vs mode crap)
                    if !camera.preclear.get_enabled() {
                        // check if the katamari is moving slow enough to change view mode
                        let under_speed_threshold = if mission_state.is_vs_mode {
                            false
                        } else if katamari.physics_flags.immobile {
                            true
                        } else if !self.oujistate.dash && katamari.physics_flags.contacts_floor {
                            let speed_ratio = katamari
                                .get_speed_ratio(self.push_dir.unwrap_or(PushDir::Forwards))
                                .clamp(0.0, 1.0);
                            speed_ratio <= self.params.max_speed_for_view_mode
                        } else {
                            false
                        };

                        let tutorial = &mission_state.tutorial;
                        let can_view_mode =
                            tutorial.as_ref().map_or(true, |tut| tut.get_page() > 0)
                                && under_speed_threshold
                                && !self.oujistate.jump_180;

                        if can_view_mode {
                            if input.l1_down && !input.r1_down {
                                // initiate an L1 look
                                held_tut_move = Some(TutorialMove::LookL1);
                                self.end_spin_and_boost(katamari);
                                katamari.set_immobile(mission_state);
                                self.view_mode = PrinceViewMode::L1Look;
                                camera.set_mode(CameraMode::L1Look, Some(katamari), Some(&self));
                            } else if input.r1_down && !input.l1_down {
                                // initiate an R1 jump
                                held_tut_move = Some(TutorialMove::JumpR1);
                                self.end_spin_and_boost(katamari);
                                // TODO: play R1_JUMP sfx
                                katamari.set_immobile(mission_state);
                                self.view_mode = PrinceViewMode::R1Jump;
                                camera.set_mode(CameraMode::R1Jump, Some(katamari), Some(&self));
                            }
                        }
                    }
                }
            }
            PrinceViewMode::L1Look => {
                // `prince_update_look_l1`
                // offset: 0x54c90
                // if the camera starts scaling up, return to the normal view mode.
                if camera.is_scaling_up() {
                    self.view_mode = PrinceViewMode::Normal;
                    self.ignore_input_timer = 0;
                    camera.set_mode_normal();
                    return None;
                }

                // update the camera with the current input
                let ls_x = (input.ls_x as f32) / 91.0;
                let ls_y = (input.ls_y as f32) / 91.0;
                camera.update_l1_look(ls_x, ls_y, &mut self.angle);
            }
            _ => (),
        }

        held_tut_move
    }

    /// Update the impact lock state.
    /// offset: 0x54e90
    fn update_impact_lock(&mut self) {}

    /// Update the prince after a royal warp.
    pub fn update_royal_warp(&mut self, katamari: &Katamari, camera: &Camera, angle: f32) {
        self.angle = angle;
        self.update_transform(katamari, camera);
    }

    /// The main function to update the prince's transform matrix each tick.
    pub fn update_transform(&mut self, katamari: &Katamari, camera: &Camera) {
        let kat_offset = -katamari.get_prince_offset();
        self.last_pos = self.pos;
        self.base_kat_offset[2] = kat_offset;

        // update transform differently depending on if the prince is flipping
        if self.oujistate.jump_180 {
            self.update_flip_transform(katamari);
        } else {
            let mut base_kat_offset = self.base_kat_offset;
            self.update_nonflip_transform(camera, &mut base_kat_offset, katamari.get_bottom());
            self.base_kat_offset = base_kat_offset;
        }

        self.flags |= 0x100;
    }

    /// Compute the ratio of the current flip that has been completed.
    /// The value is 0 when the flip starts, and 1 when the flip ends.
    pub fn get_flip_progress(&self) -> f32 {
        (self.flip_duration - self.flip_timer) as f32 / self.flip_duration as f32
    }

    /// Update the prince's transform while flipping.
    /// offset: 0x55480
    fn update_flip_transform(&mut self, katamari: &Katamari) {
        self.flip_timer -= 1;
        self.turn_type = PrinceTurnType::Flip;

        let flip_progress = self.get_flip_progress();
        self.falling_from_flip = flip_progress <= 0.5;
        self.oujistate.jump_180_leap =
            ((self.flip_timer as f32 / self.flip_duration as f32) * 128.0) as u8;

        let base_offset_z = self.base_kat_offset[2];
        let unit_flip_height = (flip_progress * PI).sin();

        self.kat_offset[0] = 0.0;
        self.kat_offset[1] = katamari.get_radius() * unit_flip_height;
        self.kat_offset[2] = base_offset_z + 2.0 * flip_progress * base_offset_z.abs();

        // compute the katamari offset in world space using the vector in prince space
        let id = mat4::create();
        let mut prince_angle_rot = mat4::create();
        mat4::rotate_y(&mut prince_angle_rot, &id, self.angle);

        let mut world_kat_offset = vec3::create();
        vec3::transform_mat4(&mut world_kat_offset, &self.kat_offset, &prince_angle_rot);
        vec3::add(&mut self.pos, &world_kat_offset, katamari.get_bottom());

        mat4::rotate_y(
            &mut self.transform_rot,
            &prince_angle_rot,
            flip_progress * PI,
        );

        if self.flip_timer == 0 {
            // end the current flip once the timer reaches 0

            self.oujistate.jump_180 = false;
            self.ignore_input_timer = 0;

            // add pi to the prince angle, signifying that it has rotated 180 degrees around the
            // katamari at the end of the flip
            change_bounded_angle(&mut self.angle, PI);
        }
    }

    /// Update the prince's transform matrix while not flipping.
    /// offset: 0x53650
    fn update_nonflip_transform(
        &mut self,
        camera: &Camera,
        mut kat_offset: &mut Vec3,
        kat_bottom: &Vec3,
    ) {
        self.angle = normalize_bounded_angle(self.angle);
        self.angle += self.extra_flat_angle_speed;
        self.angle = normalize_bounded_angle(self.angle);

        let id = mat4::create();
        let mut local_pos = vec3::create();
        let mut rotation_mat = [0.0; 16];

        mat4::rotate_y(
            &mut rotation_mat,
            &id,
            self.angle + self.auto_rotate_right_speed,
        );
        vec3::transform_mat4(&mut local_pos, &[0.0, 0.0, kat_offset[2]], &rotation_mat);

        // TODO_VS: `prince_update_nonflip_transform:141-243`

        vec3::add(&mut self.pos, &local_pos, &kat_bottom);

        if self.view_mode == PrinceViewMode::R1Jump {
            // TODO_PARAM
            let R1_JUMP_TRANSLATION_CAM_MULT = 0.6;
            let translation = camera.get_r1_jump_translation().clone();
            vec3::scale(&mut kat_offset, &translation, R1_JUMP_TRANSLATION_CAM_MULT);
            vec3_inplace_add_vec(&mut self.pos, &kat_offset);
        }

        mat4::copy(&mut self.transform_rot, &rotation_mat);
    }
}

impl Player {
    /// The main function to update the prince each tick.
    pub fn update_prince(&mut self, mission_state: &mut MissionState) {
        let prince = &mut self.prince;
        let katamari = &mut self.katamari;
        let input = &mut self.input;
        let camera = &mut self.camera;
        let stage_config = &mission_state.stage_config;

        prince.last_oujistate = prince.oujistate;

        prince.flip_duration = stage_config.get_flip_duration(katamari.get_diam_cm()) as u16;

        prince.read_input(input, mission_state, katamari, camera);
        prince.update_huff();
        prince.try_end_view_mode(camera);
        prince.update_boost_recharge();
        prince.update_analog_input_features();

        if prince.view_mode == PrinceViewMode::Normal {
            prince.update_gachas(katamari, camera, mission_state);
            let angle_tut_move = prince.update_angle(katamari);

            if let Some(tut_move) = angle_tut_move {
                mission_state
                    .tutorial
                    .as_mut()
                    .map(|tut| tut.set_move_held(tut_move));
            }
        }

        prince.update_boost_push_rotation_mat(mission_state.is_vs_mode);

        if mission_state.gamemode.can_update_view_mode()
            && katamari.physics_flags.vs_mode_state != 2
        {
            prince.update_impact_lock();
            let view_mode_tut_move =
                prince.update_view_mode(camera, katamari, input, mission_state);

            if let Some(tut_move) = view_mode_tut_move {
                mission_state
                    .tutorial
                    .as_mut()
                    .map(|tut| tut.set_move_held(tut_move));
            }
        }
    }
}
