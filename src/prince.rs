use gl_matrix::{
    common::{Mat4, Vec4},
    mat4, vec4,
};

use crate::{
    constants::VEC4_ZERO,
    input::{AnalogPushDirs, GachaDir, StickInput},
    katamari::Katamari,
    math::normalize_bounded_angle,
};

#[derive(Debug, Default, Clone)]
pub struct OujiState {
    pub dash_start: u8,
    pub wheel_spin_start: u8,
    pub dash: u8,
    pub wheel_spin: u8,
    pub jump_180: u8,
    pub sw_speed_disp: u8,
    pub climb_wall: u8,
    pub dash_tired: u8,
    pub camera_mode: u8,
    pub dash_effect: u8,
    pub hit_water: u8,
    pub submerge: u8,
    pub camera_state: u8,
    pub jump_180_leap: u8,
    pub brake: u8,
    pub tutorial_flag_1: u8,
    pub tutorial_flag_2: u8,
    pub tutorial_trigger_1: u8,
    pub tutorial_trigger_2: u8,
    pub power_charge: u8,
    pub fire_to_enemy: u8,
    pub search: u8,
    pub attack_1: u8,
    pub attack_2: u8,
    pub tarai: u8,
    pub attack_wait: u8,
    pub vs_attack: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Normal = 0,
    R1Jump = 1,
    L1Look = 2,
}

impl Default for ViewMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// The directions that the prince can push the katamari.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrincePushDir {
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

/// Classifies the ways the prince can move around the katamari.
/// Note that all katamari-pushing movement belongs to `Push`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrinceTurnType {
    None,
    WalkLeftUp,
    WalkRightUp,
    WalkLeftDown,
    WalkRightDown,
    Flip,
    Push,
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
    /// The player index controlling this prince.
    /// offset: 0x0
    player: u8,

    /// (??) Random bit vector.
    /// offset: 0x4
    flags: u16,

    /// The current position of the prince.
    /// offset: 0xc
    pos: Vec4,

    /// The position of the prince on the previous tick.
    /// offset: 0x1c
    last_pos: Vec4,

    /// The prince's offset from the katamari at the start of a flip.
    /// offset: 0x5c
    flip_init_kat_offset: Vec4,

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
    remaining_huff_ratio: f32,

    /// The prince's view mode, which distinguishes being in the R1 and L1 states.
    /// offset: 0x9c
    view_mode: ViewMode,

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
    quick_shifting: bool,

    /// Various 1-byte fields that are shared with the Unity code.
    /// offset: 0xa2
    oujistate: OujiState,

    /// The previous frame's values of various 1-byte fields that are shared with the Unity code.
    /// offset: 0xbd
    last_oujistate: OujiState,

    /// If >0, ignores player input, and decrements by 1 each frame.
    /// If <0, ignores player input forever until changed.
    /// If 0, player input is read as usual.
    /// offset: 0xf4
    ignore_input_ticks: i16,

    /// The transform matrix of the prince.
    /// Note that this is only a rotation matrix; it doesn't include translation.
    /// offset: 0x138
    transform_rot: Mat4,

    /// The default offset of the prince from the katamari center in local katamari space.
    /// Since the prince is always facing the z+ axis, this vector is therefore always
    /// a multiple of the z- axis (since the prince is behind the katamari center)
    /// offset: 0x24c
    kat_offset_vec: Vec4,

    /// (??)
    /// offset: 0x288
    min_push_angle_y: f32,

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
    min_angle_btwn_sticks_for_fastest_turn: f32,

    /// (??)
    /// offset: 0x2bc
    low_stick_angle_threshold: f32,

    /// (??)
    /// offset: 0x2c0
    high_stick_angle_threshold: f32,

    /// A forward push of this value or higher is scaled to 1, with lower values rescaled between [0,1].
    /// offset: 0x2c4
    forward_push_cap: f32,

    /// The minimum push with both sticks to start moving.
    /// offset: 0x2c8
    min_push_to_move: f32,

    /// The number of ticks it takes to complete the flip animation.
    /// offset: 0x2d0
    flip_duration_ticks: u32,

    /// The
    /// offset: 0x2d4
    max_analog_allowing_flip: f32,

    /// The number of ticks allowed between gachas before the gacha count resets.
    /// offset: 0x2dc
    gacha_window_ticks: u16,

    /// The number of gachas needed to initiate a spin.
    /// offset: 0x2e0
    gachas_for_spin: u16,

    /// The maximum boost energy.
    /// offset: 0x2d4
    max_boost_energy: u16,

    /// The amount of boost energy gained per recharge.
    /// offset: 0x2ec
    boost_recharge_amount: u16,

    /// The number of ticks between boost recharges (assuming the player doesn't spin and
    /// reset the timer)
    /// offset: 0x2f0
    boost_recharge_frequency: u16,

    /// The duration of a huff, in ticks.
    /// offset: 0x2f4
    huff_duration_ticks: u16,

    /// The initial multiplier on katamari speed during a huff (this penalty decays as the
    /// huff gets closer to ending)
    /// offset: 0x2f8
    huff_init_speed_penalty: f32,

    /// (??)
    /// offset: 0x300
    init_uphill_strength: f32,

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

    /// The average of `input_ls` and `input_rs`
    /// offset: 0x398
    input_avg: StickInput,

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
    angle_between_sticks: f32,

    /// (??) too lazy
    /// offset: 0x3c0
    input_scaled_avg_len: f32,

    /// 0 if moving sideways, [0,1] if forwards/backwards depending on y angle of net input
    /// offset: 0x3c4
    scaled_push: f32,

    /// (??)
    /// offset: 0x3cc
    yaw_rotation_something: Mat4,

    /// (??)
    /// offset: 0x40c
    yaw_rotation_something_else: Mat4,

    /// The number of ticks remaining in the current flip animation.
    /// offset: 0x44c
    flip_remaining_ticks: u16,

    /// Input push directions which just changed this tick.
    /// offset: 0x470
    push_dirs_down: AnalogPushDirs,

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
    gacha_window_timer_ticks: u16,

    /// The number of gachas counted without the gacha timer expiring.
    /// offset: 0x47c
    gacha_count: u16,

    /// The amount of remaining boost energy.
    /// offset: 0x47e
    boost_energy: u16,

    /// The number of ticks since the katamari was spun. Used for boost recharging.
    /// offset: 0x484
    no_spin_ticks: u16,

    /// The remaining ticks in the current huff.
    /// offset: 0x486
    huff_remain_ticks: u16,

    /// The strength with which the katamari can be pushed uphill. Decreases while
    /// pushing uphill. (Seems to start at 100 and decrease from there, so maybe it's a percentage)
    /// offset: 0x488
    uphill_strength: f32,

    /// The direction the prince is pushing the katamari, if any.
    /// offset: 0x48c
    push_dir: Option<PrincePushDir>,

    /// The direction the prince is pushing the katamari sideways, if any.
    /// offset: 0x48d
    push_sideways_dir: Option<PrinceSidewaysDir>,

    /// The type of turning around the katamari the prince is doing.
    /// offset: 0x48e
    turn_type: PrinceTurnType,
}

impl Prince {
    /// Initialize the prince at the start of a mission.
    pub fn init(&mut self, player: u8, init_angle: f32, kat: &Katamari) {
        self.player = player;
        self.no_spin_ticks = 0;
        self.huff_remain_ticks = 0;
        vec4::copy(&mut self.pos, &VEC4_ZERO);
        self.auto_rotate_right_speed = 0.0;
        self.angle = init_angle;
        self.kat_offset_vec[2] = kat.get_prince_offset();
        self.push_dir = None;
        self.angle_speed = 0.0;
        mat4::identity(&mut self.yaw_rotation_something);
        mat4::identity(&mut self.transform_rot);
        self.huff_init_speed_penalty = 0.4;
        self.huff_duration_ticks = 240; // TODO: some weird potential off-by-one issue here.
        self.init_uphill_strength = 100.0;
        self.uphill_strength_loss = 0.7649993;
        self.low_stick_angle_threshold = 0.8733223;
        self.high_stick_angle_threshold = 2.270252;
        self.forward_push_cap = 0.5;
        self.one_stick_up_turn_speed = 0.035;
        self.one_stick_down_turn_speed = 0.025;
        self.quick_shift_turn_speed = 0.055;
        self.backwards_turn_speed = 0.03;
        self.non_backwards_turn_speed = 0.06;
        self.max_analog_allowing_flip = 0.3;
        self.gacha_window_ticks = 14;
        self.max_boost_energy = 0xf0;
        self.gachas_for_spin = 3;
        self.boost_recharge_amount = 18;
        self.boost_recharge_frequency = 100;
        self.min_angle_btwn_sticks_for_fastest_turn = 0.75;
        self.min_push_angle_y = 0.363474;

        self.update_transform(kat);

        self.boost_energy = self.max_boost_energy;
        self.uphill_strength = self.init_uphill_strength;
        self.view_mode = ViewMode::Normal;
        self.ignore_input_ticks = 0;

        // TODO: `prince_init:100-123` (vs mode crap)
    }

    /// The main function to update the prince's transform matrix each tick.
    fn update_transform(&mut self, kat: &Katamari) {
        let kat_offset = kat.get_prince_offset();
        let kat_center = kat.get_center();
        self.last_pos = self.pos;
        self.kat_offset_vec[2] = kat_offset;

        // update transform differently depending on if the prince is flipping
        if self.oujistate.jump_180 != 0 {
            self.update_flip_transform(kat_offset);
        } else {
            self.update_nonflip_transform(kat_offset, &kat_center);
        }

        self.flags |= 0x100;
    }

    /// TODO
    /// offset: 0x55480
    fn update_flip_transform(&mut self, _kat_offset: f32) {}

    /// Update the prince's transform matrix while not flipping.
    /// offset: 0x53650
    fn update_nonflip_transform(&mut self, kat_offset: f32, kat_center: &Vec4) {
        self.angle = normalize_bounded_angle(self.angle);
        self.angle += self.extra_flat_angle_speed;
        self.angle = normalize_bounded_angle(self.angle);

        let id = mat4::create();
        let mut local_pos = vec4::create();
        let mut rotation_mat = [0.0; 16];

        mat4::rotate_y(
            &mut rotation_mat,
            &id,
            self.angle + self.auto_rotate_right_speed,
        );
        vec4::transform_mat4(&mut local_pos, &[0.0, 0.0, kat_offset, 1.0], &rotation_mat);

        // TODO: `prince_update_nonflip_transform:141-243` (vs mode crap)

        vec4::add(&mut self.pos, &local_pos, &kat_center);

        // TODO: `prince_update_nonclip_transform:251-268` (handle r1 jump)

        mat4::copy(&mut self.transform_rot, &rotation_mat);
    }

    pub fn copy_oujistate_ptr(&mut self, oujistate: &mut *mut OujiState, data_size: &mut i32) {
        *data_size = 0x1b;
        *oujistate = &mut self.oujistate as *mut OujiState;
    }

    pub fn get_view_mode(&self) -> ViewMode {
        self.view_mode
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
        *tx = self.pos[0];
        *ty = self.pos[1];
        *tz = self.pos[2];
    }

    pub fn set_ignore_input_timer(&mut self, value: i16) {
        self.ignore_input_ticks = value;
    }
}
