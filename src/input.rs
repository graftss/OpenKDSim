macro_rules! dequantize {
    ($expr: expr) => {
        if $expr > 0 {
            ($expr as f32) / 91.0
        } else {
            ($expr as f32) / 90.0
        }
    };
}

/// Holds all controller input that can occur in a single tick.
#[derive(Debug, Default)]
pub struct Input {
    // Analog input
    // Each analog axis is quantized to a signed byte.
    /// Left stick x axis input.
    pub ls_x: i8,
    /// Left stick y axis input.
    pub ls_y: i8,
    /// Right stick x axis input.
    pub rs_x: i8,
    /// Right stick y axis input.
    pub rs_y: i8,

    // Button input
    // `down` means this is the first frame the button was pressed.
    // `held` means the button is currently being held down
    // (and may have been held the last frame too)
    pub l1_down: bool,
    pub l1_held: bool,
    pub l2_down: bool,
    pub l2_held: bool,
    pub l3_down: bool,
    pub l3_held: bool,
    pub r1_down: bool,
    pub r1_held: bool,
    pub r2_down: bool,
    pub r2_held: bool,
    pub r3_down: bool,
    pub r3_held: bool,

    // (??)
    pub cross_click: bool,
}

/// A single analog stick's non-quantized input.
#[derive(Debug, Default, Clone, Copy)]
pub struct StickInput {
    pub x: f32,
    pub y: f32,
}

impl StickInput {
    pub fn clear(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
    }

    pub fn absolute(&self, out: &mut StickInput) {
        out.x = self.x.abs();
        out.y = self.y.abs();
    }
}

/// The possible directions a single stick can push.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StickPushDir {
    None,
    Up,
    Down,
}

/// The possible directions both sticks can push.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalogPushDirs {
    pub left: StickPushDir,
    pub right: StickPushDir,
}

impl Default for AnalogPushDirs {
    fn default() -> Self {
        Self {
            left: StickPushDir::None,
            right: StickPushDir::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GachaDir {
    Left,
    Right,
}

impl Input {
    /// Dequantize this input's analog axes from i8 to f32.
    pub fn dequantize(&self, ls: &mut StickInput, rs: &mut StickInput) {
        ls.x = dequantize!(self.ls_x);
        ls.y = dequantize!(self.ls_y);
        rs.x = dequantize!(self.rs_x);
        rs.y = dequantize!(self.rs_y);
    }

    pub fn set_stick_state(
        &mut self,
        ls_x: f32,
        ls_y: f32,
        rs_x: f32,
        rs_y: f32,
        l3_down: bool,
        r3_down: bool,
        l3_held: bool,
        r3_held: bool,
    ) {
        // this is where the input is quantized from [-1, 1] to [-128, 127] (or whatever the endpoints are)
        self.ls_x = (ls_x * 127.0) as i8;
        self.ls_y = (ls_y * 127.0) as i8;
        self.rs_x = (rs_x * 127.0) as i8;
        self.rs_y = (rs_y * 127.0) as i8;

        self.l3_down = l3_down;
        self.r3_down = r3_down;
        self.l3_held = l3_held;
        self.r3_held = r3_held;
    }

    pub fn set_trigger_state(
        &mut self,
        l1_down: bool,
        l1_held: bool,
        l2_down: bool,
        l2_held: bool,
        r1_down: bool,
        r1_held: bool,
        r2_down: bool,
        r2_held: bool,
        cross_click: bool,
    ) {
        self.l1_down = l1_down;
        self.l1_down = l1_held;
        self.l2_down = l2_down;
        self.l2_down = l2_held;
        self.r1_down = r1_down;
        self.r1_down = r1_held;
        self.r2_down = r2_down;
        self.r2_down = r2_held;
        self.cross_click = cross_click;
    }
}
