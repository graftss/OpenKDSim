use std::fmt::Display;

use gl_matrix::{common::Vec2, vec2};
use serde::{Serialize, Deserialize};

use crate::math::acos_f32;

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
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct StickInput {
    pub axes: Vec2,
}

impl StickInput {
    /// Read the x axis of analog input.
    pub fn x(&self) -> f32 {
        self.axes[0]
    }

    /// Read the y axis of analog input.
    pub fn y(&self) -> f32 {
        self.axes[1]
    }

    /// Compute the angle (in [-pi, pi]) the stick is pointing towards, where:
    /// y+ axis = 0; x+ axis = pi/2; y- axis = pi; x- axis = -pi/2
    pub fn angle(&self) -> f32 {
        let mut ls_angle = acos_f32(self.axes[1]);
        if self.axes[0] < 0.0 {
            ls_angle *= -1.0;
        }

        ls_angle
    }

    /// Compute the angle between this stick input and the stick input `other`.
    pub fn angle_with_other(&self, other: &StickInput) -> f32 {
        let dot = self.axes[0] * other.axes[0] + self.axes[1] * other.axes[1];
        acos_f32(dot)
    }

    /// Clear this input.
    pub fn clear(&mut self) {
        self.axes = [0.0, 0.0];
    }

    /// Write the absolute value of this input to `out`.
    pub fn absolute(&self, out: &mut StickInput) {
        out.axes[0] = self.axes[0].abs();
        out.axes[1] = self.axes[1].abs();
    }

    /// Write this input normalized to a unit vector to `out`.
    pub fn normalize(&self, out: &mut StickInput) {
        vec2::normalize(&mut out.axes, &self.axes);
    }

    pub fn len(&self) -> f32 {
        vec2::len(&self.axes)
    }

    /// Normalize the sum of the `left` and `right` inputs, writing the result to `out`.
    pub fn normalize_sum(out: &mut StickInput, left: &StickInput, right: &StickInput) {
        let mut axes_sum = [0.0, 0.0];
        vec2::add(&mut axes_sum, &left.axes, &right.axes);
        vec2::normalize(&mut out.axes, &axes_sum);
    }
}

/// The possible directions a single stick can push.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StickPushDir {
    Up,
    Down,
}

/// The possible directions both sticks can push.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalogPushDirs {
    pub left: Option<StickPushDir>,
    pub right: Option<StickPushDir>,
}

impl Default for AnalogPushDirs {
    fn default() -> Self {
        Self {
            left: None,
            right: None,
        }
    }
}

impl AnalogPushDirs {
    /// Clears this struct to have no push directions.
    pub fn clear(&mut self) {
        self.left = None;
        self.right = None;
    }
    /// Computes the push directions of `left` and `right`, using
    /// `min_push_len` as the minimum y axis input to be considered pushing.
    pub fn update_from_input(&mut self, ls_y: f32, rs_y: f32, min_push_len: f32) {
        self.left = if ls_y > min_push_len {
            Some(StickPushDir::Up)
        } else if ls_y < -min_push_len {
            Some(StickPushDir::Down)
        } else {
            None
        };

        self.right = if rs_y > min_push_len {
            Some(StickPushDir::Up)
        } else if rs_y < -min_push_len {
            Some(StickPushDir::Down)
        } else {
            None
        };
    }

    /// Computes the "changed" input directions from the `last` directions to the `current` ones.
    /// If `last` and `current` are the same direction the "changed" dir is `None`.
    /// If they're different directions, the "changed" dir is the `current` dir.
    pub fn compute_changed(&mut self, last: &AnalogPushDirs, current: &AnalogPushDirs) {
        self.left = if last.left != current.left {
            current.left
        } else {
            None
        };

        self.right = if last.right != current.right {
            current.right
        } else {
            None
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GachaDir {
    Left,
    Right,
}

impl Input {
    /// Dequantize this input's analog axes from i8 to f32.
    pub fn dequantize(&self, ls: &mut StickInput, rs: &mut StickInput) {
        ls.axes[0] = dequantize!(self.ls_x);
        ls.axes[1] = dequantize!(-self.ls_y);
        rs.axes[0] = dequantize!(self.rs_x);
        rs.axes[1] = dequantize!(-self.rs_y);
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
        // this is where the input is quantized from [-1, 1] to [-128, 127]
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
        self.l1_held = l1_held;
        self.l2_down = l2_down;
        self.l2_held = l2_held;
        self.r1_down = r1_down;
        self.r1_held = r1_held;
        self.r2_down = r2_down;
        self.r2_held = r2_held;
        self.cross_click = cross_click;
    }

    pub fn any_trigger_down(&self) -> bool {
        self.l1_down
            || self.l1_held
            || self.l2_down
            || self.l2_held
            || self.r1_down
            || self.r1_held
            || self.r2_down
            || self.r2_held
    }
}

impl Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Input(L1d={}, L1h={}, L2d={}, L2d={}, L2h={}, R1d={}, R1h={}, R2d={}, R2h={}",
            self.l1_down,
            self.l1_held,
            self.l2_held,
            self.l2_down,
            self.l2_held,
            self.r1_down,
            self.r1_held,
            self.r2_down,
            self.r2_held
        )
    }
}
