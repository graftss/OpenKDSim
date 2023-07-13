use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TutorialMoves {
    pub roll_forwards: bool,
    pub roll_to_the_right: bool,
    pub roll_to_the_left: bool,
    pub roll_backwards: bool,
    pub brake: bool,
    pub shift_left_or_right: bool,
    pub quick_shift: bool,
    pub roll_sideways: bool,
    pub boost: bool,
    pub look_l1: bool,
    pub flip: bool,
    pub jump_r1: bool,
}

impl TutorialMoves {
    /// Get a reference to the move flag corresponding to `tut_move`.
    pub fn set_move(&mut self, tut_move: TutorialMove, value: bool) {
        match tut_move {
            TutorialMove::RollForwards => self.roll_forwards = value,
            TutorialMove::RollToTheRight => self.roll_to_the_right = value,
            TutorialMove::RollToTheLeft => self.roll_to_the_left = value,
            TutorialMove::RollBackwards => self.roll_backwards = value,
            TutorialMove::Brake => self.brake = value,
            TutorialMove::ShiftLeftOrRight => self.shift_left_or_right = value,
            TutorialMove::QuickShift => self.quick_shift = value,
            TutorialMove::RollSideways => self.roll_sideways = value,
            TutorialMove::Boost => self.boost = value,
            TutorialMove::LookL1 => self.look_l1 = value,
            TutorialMove::Flip => self.flip = value,
            TutorialMove::JumpR1 => self.jump_r1 = value,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum TutorialMove {
    RollForwards,
    RollToTheRight,
    RollToTheLeft,
    RollBackwards,
    Brake,
    ShiftLeftOrRight,
    QuickShift,
    RollSideways,
    Boost,
    LookL1,
    Flip,
    JumpR1,
}

/// Maintains tutorial-specific state.
/// offset: 0xd34680
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TutorialState {
    /// (??) The second argument of `SetTutorialA`.
    /// offset: 0x0
    page_step: u8,

    /// (??)
    /// offset: 0x4
    camera_animation_timer_ticks: u16,

    /// (??) The first argument of `SetTutorialA`.
    /// offset: 0x8
    page: u8,

    /// A move flag is `true` if it is currently being held by the player.
    /// offset: 0x16
    move_held: TutorialMoves,

    /// A move flag is `true` if the player has received credit for performing it.
    /// offset: 0x22
    move_credit: TutorialMoves,

    /// A move flag is `true` if the player has just started performing it this tick.
    /// ofset: 0x2e
    move_down: TutorialMoves,
}

impl TutorialState {
    pub fn get_page(&self) -> u8 {
        self.page
    }

    pub fn set_page(&mut self, page: i32, page_step: i32) {
        self.page = page.try_into().unwrap();
        self.page_step = page_step.try_into().unwrap();
    }

    pub fn set_move_held(&mut self, tut_move: TutorialMove) {
        self.move_held.set_move(tut_move, true);
    }
}
