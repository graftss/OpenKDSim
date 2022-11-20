use gl_matrix::common::{Mat4, Vec4};

#[derive(Debug, Default)]
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

#[derive(Debug, Clone, Copy)]
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

/// State pertaining to the prince. It's a little arbitrary as far as what's in `Prince` versus
/// what's in `Katamari`.
/// offset: 0xd33210
/// width: 0x518
#[derive(Debug, Default)]
pub struct Prince {
    /// The player index controlling this prince.
    /// offset: 0x0
    player: u8,

    /// The prince's view mode, which distinguishes being in the R1 and L1 states.
    /// offset: 0x9c
    view_mode: ViewMode,

    /// Various 1-byte fields that are shared with the Unity code.
    /// offset: 0xa2
    oujistate: OujiState,

    /// The previous frame's values of various 1-byte fields that are shared with the Unity code.
    /// offset: 0xbd
    last_oujistate: OujiState,

    /// (??) The position of the prince. (in world space?? local space??)
    /// offset: 0xc
    pos: Vec4,

    /// (??) The last position of the prince.
    /// offset: 0x1c
    last_pos: Vec4,

    /// If >0, ignores player input, and decrements by 1 each frame.
    /// If <0, ignores player input forever until changed.
    /// If 0, player input is read as usual.
    /// offset: 0xf4
    ignore_input_timer: i16,

    /// The transform matrix of the prince.
    /// offset: 0x138
    transform: Mat4,
}

impl Prince {
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
        *xx = self.transform[0];
        *xy = self.transform[1];
        *xz = self.transform[2];
        *yx = self.transform[4];
        *yy = self.transform[5];
        *yz = self.transform[6];
        *zx = self.transform[8];
        *zy = self.transform[9];
        *zz = self.transform[10];
        *tx = self.pos[0];
        *ty = self.pos[1];
        *tz = self.pos[2];
    }

    pub fn set_ignore_input_timer(&mut self, value: i16) {
        self.ignore_input_timer = value;
    }
}
