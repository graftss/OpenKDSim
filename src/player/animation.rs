use crate::{
    delegates::DelegatesRef,
    global::rng::RngState,
    macros::{log},
    mission::{state::MissionState, tutorial::TutorialMove},
    player::{
        camera::{mode::CameraMode, CamR1JumpState, Camera},
        katamari::Katamari,
    },
};

use super::{
    katamari::CamRelativeDir,
    prince::{Prince, PrinceSidewaysDir, PrinceTurnType, PrinceViewMode, PushDir},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationId {
    Idle1,
    IdleHuff,
    WalkUp,
    JogUp,
    WalkDown,
    WalkRight,
    WalkLeft,
    Spin,
    Boost,
    WalkUpHuff,
    WalkUpNoPush,
    WalkRightNoPush,
    WalkLeftNoPush,
    WalkAroundLeft,
    WalkAroundRight,
    BrakeUp,
    BrakeDown,
    BrakeLeft,
    BrakeRight,
    VsMode0x16,
    L1Look,
    R1Jump,
    Flip,
    L1LookUnused0x1a,
    Climb,
    Idle2,
    VsMode0x22,
    Pinch,
    WalkDownNoPush,
}

impl Into<u8> for AnimationId {
    fn into(self) -> u8 {
        match self {
            AnimationId::Idle1 => 0,
            AnimationId::IdleHuff => 1,
            AnimationId::WalkUp => 2,
            AnimationId::JogUp => 3,
            AnimationId::WalkDown => 4,
            AnimationId::WalkRight => 5,
            AnimationId::WalkLeft => 6,
            AnimationId::Spin => 7,
            AnimationId::Boost => 8,
            AnimationId::WalkUpHuff => 9,
            AnimationId::WalkUpNoPush => 0xd,
            AnimationId::WalkRightNoPush => 0xe,
            AnimationId::WalkLeftNoPush => 0xf,
            AnimationId::WalkAroundLeft => 0x10,
            AnimationId::WalkAroundRight => 0x11,
            AnimationId::BrakeUp => 0x12,
            AnimationId::BrakeDown => 0x13,
            AnimationId::BrakeLeft => 0x14,
            AnimationId::BrakeRight => 0x15,
            AnimationId::VsMode0x16 => 0x16,
            AnimationId::L1Look => 0x17,
            AnimationId::R1Jump => 0x18,
            AnimationId::Flip => 0x19,
            AnimationId::L1LookUnused0x1a => 0x1a,
            AnimationId::Climb => 0x1b,
            AnimationId::Idle2 => 0x1c,
            AnimationId::VsMode0x22 => 0x22,
            AnimationId::Pinch => 0x26,
            AnimationId::WalkDownNoPush => 0x28,
        }
    }
}

impl Default for AnimationId {
    fn default() -> Self {
        Self::Idle1
    }
}

impl AnimationId {
    /// Returns `true` if this animation id is a non-huff idle animation.
    pub fn is_idle(self) -> bool {
        self == AnimationId::Idle1 || self == AnimationId::Idle2
    }
}

#[derive(Debug)]
pub struct AnimationParams {
    /// The highest the katamari's max speed ratio can be while still performing
    /// an idle animation.
    /// default: 0.05
    /// offset: 0x71578 (used at 0x57c74)
    pub max_speed_ratio_to_idle: f32,

    /// The lowest the katamari's max speed ratio must be to enter the `JogUp` animation.
    /// default: 0.3
    /// offset: 0x715a8 (used at 0x5797d and 0x57a94)
    pub max_speed_ratio_to_jog: f32,
}

impl Default for AnimationParams {
    fn default() -> Self {
        Self {
            max_speed_ratio_to_idle: f32::from_bits(0x3d4ccccd), // 0.05
            max_speed_ratio_to_jog: f32::from_bits(0x3e99999a),  // 0.3
        }
    }
}

/// A struct in the original simulation that seems to mostly hold animation behavior.
/// Since a lot of animation logic in Reroll is handled by Unity, this struct was
/// mostly unused, and mainly just lets Unity know which animations to play.
#[derive(Debug, Default)]
pub struct Animation {
    // BEGIN not part of original simulation struct
    /// A reference to the Unity delegates for the purposes of starting animations in Unity.
    delegates: Option<DelegatesRef>,

    /// Collected magic constants used in animation code.
    params: AnimationParams,

    /// The next idle animation that will play when the player is idle.
    /// This value changes randomly between the `Idle1` and `Idle2` animations.
    /// For some reason, it was a global variable in the original simulation.
    /// offset: 0xd35558
    next_idle_animation: AnimationId,

    /// The player index associated to this animation state.
    player_idx: u8,

    // END not part of original simulation struct
    /// The ID of the active animation.
    /// offset: 0xd
    pub id: AnimationId,

    /// (??)
    /// offset: 0xf    
    pub unknown_0xf: u8,

    /// (??)
    /// offset: 0x10
    pub alt_id: AnimationId,

    /// The playback speed of the active animation.
    /// offset: 0x1c8
    pub speed: f32,
}

impl Animation {
    pub fn set_delegates(&mut self, delegates: &DelegatesRef) {
        self.delegates = Some(delegates.clone());
    }

    /// Main animation update function.
    /// offset: 0x57420
    pub fn update(
        &mut self,
        prince: &Prince,
        katamari: &Katamari,
        camera: &Camera,
        mission_state: &mut MissionState,
        rng: &mut RngState,
    ) {
        // TODO_VS: `prince_update_animation:26-141` (it seems like most or all of this is for vs mode/unused)
        let (anim_id, speed, repeat, tut_move) = if camera.get_mode() == CameraMode::R1Jump {
            // case 1: play r1 jump animation
            let height_ratio = camera.get_r1_jump_height_ratio();
            match camera.get_r1_jump_state() {
                Some(CamR1JumpState::Rising) if height_ratio < 0.5 => {
                    (AnimationId::Flip, 1.0, false, None)
                }
                _ => (AnimationId::R1Jump, 1.0, height_ratio <= 0.0, None),
            }
        } else if !katamari.physics_flags.airborne {
            if prince.get_view_mode() == PrinceViewMode::L1Look {
                // case 2: not airborne, L1 look
                (AnimationId::L1Look, 1.0, false, None)
            } else {
                let max_speed_ratio = katamari.get_max_speed_ratio();

                // case 3: not airborne, not L1 look
                let (id, speed, tut_move) = match prince.get_turn_type() {
                    PrinceTurnType::None => {
                        // case 3.1: not turning (no input)
                        if self.try_play_idle_animation(prince, katamari, rng) {
                            return;
                        }

                        let id = match katamari.get_cam_relative_dir() {
                            Some(CamRelativeDir::Forwards) | None => AnimationId::WalkUpNoPush,
                            Some(CamRelativeDir::Backwards) => AnimationId::WalkDownNoPush,
                            Some(CamRelativeDir::Left) => AnimationId::WalkLeftNoPush,
                            Some(CamRelativeDir::Right) => AnimationId::WalkRightNoPush,
                        };

                        (id, 1.0, None)
                    }

                    // case 3.2: left stick up turn
                    PrinceTurnType::LeftStickUp => (
                        AnimationId::WalkAroundRight,
                        1.0,
                        Some(TutorialMove::ShiftLeftOrRight),
                    ),

                    // case 3.3: right stick up turn
                    PrinceTurnType::RightStickUp => (
                        AnimationId::WalkAroundLeft,
                        1.0,
                        Some(TutorialMove::ShiftLeftOrRight),
                    ),

                    // case 3.4: left stick down turn
                    PrinceTurnType::LeftStickDown => (
                        AnimationId::WalkAroundLeft,
                        0.85,
                        Some(TutorialMove::ShiftLeftOrRight),
                    ),

                    // case 3.5 right stick down turn
                    PrinceTurnType::RightStickDown => (
                        AnimationId::WalkAroundRight,
                        0.85,
                        Some(TutorialMove::ShiftLeftOrRight),
                    ),

                    // case 3.6: flipping
                    PrinceTurnType::Flip => (AnimationId::Flip, 1.0, None),

                    // case 3.7: both sticks are pushed
                    PrinceTurnType::BothSticks => {
                        if prince.oujistate.wheel_spin {
                            // case 3.7.1: spinning
                            (AnimationId::Spin, 1.0, None)
                        } else if prince.oujistate.dash {
                            // case 3.7.2: boosting (and not spinning)
                            (AnimationId::Boost, 1.0, None)
                        } else if prince.get_is_quick_shifting() {
                            // case 3.7.3: quick shifting (and not spinning or boosting)
                            let id = if !katamari.physics_flags.immobile
                                && max_speed_ratio >= self.params.max_speed_ratio_to_jog
                            {
                                if katamari.get_cam_relative_dir()
                                    == Some(CamRelativeDir::Backwards)
                                {
                                    AnimationId::WalkDown
                                } else {
                                    AnimationId::JogUp
                                }
                            } else if prince.get_angle_speed() > 0.0 {
                                AnimationId::WalkAroundRight
                            } else {
                                AnimationId::WalkAroundLeft
                            };

                            (id, 1.7, None)
                        } else if self.try_play_idle_animation(prince, katamari, rng) {
                            // case 3.7.4: moving slow enough to play the idle animation
                            return;
                        } else if katamari.physics_flags.climbing_wall {
                            // case 3.7.5: climbing wall
                            (AnimationId::Climb, 1.0, None)
                        } else if !katamari.physics_flags.braking {
                            // case 3.7.6: not braking
                            let id = match katamari.get_cam_relative_dir() {
                                Some(CamRelativeDir::Forwards) => {
                                    if self.try_play_pinch_animation(prince, rng) {
                                        return;
                                    } else if max_speed_ratio < self.params.max_speed_ratio_to_jog {
                                        AnimationId::WalkUp
                                    } else if prince.get_is_huffing() {
                                        AnimationId::WalkUpHuff
                                    } else {
                                        AnimationId::JogUp
                                    }
                                }
                                Some(CamRelativeDir::Backwards) => AnimationId::WalkDown,
                                Some(CamRelativeDir::Left) => AnimationId::WalkLeft,
                                Some(CamRelativeDir::Right) => AnimationId::WalkRight,
                                None => {
                                    if self.try_play_pinch_animation(prince, rng) {
                                        return;
                                    } else {
                                        AnimationId::JogUp
                                    }
                                }
                            };

                            (id, 1.0, None)
                        } else {
                            // case 3.7.7: braking
                            let side_dir = prince.get_push_sideways_dir();

                            let anim_id = match katamari.get_brake_push_dir() {
                                Some(PushDir::Forwards) => AnimationId::BrakeUp,
                                Some(PushDir::Backwards) => AnimationId::BrakeDown,
                                Some(PushDir::Sideways)
                                    if side_dir == Some(PrinceSidewaysDir::Left) =>
                                {
                                    AnimationId::BrakeLeft
                                }
                                Some(PushDir::Sideways)
                                    if side_dir == Some(PrinceSidewaysDir::Right) =>
                                {
                                    AnimationId::BrakeRight
                                }
                                _ => self.next_idle_animation,
                            };

                            (anim_id, 1.0, Some(TutorialMove::RollBackwards))
                        }
                    }
                };

                (id, speed, false, tut_move)
            }
        } else {
            // case 4: airborne (??)
            (AnimationId::JogUp, 1.0, false, None)
        };

        if let Some(tm) = tut_move {
            mission_state.set_tutorial_move_held(tm);
        };

        self.play(anim_id, speed, repeat, rng);
    }

    /// offset: 0x57cde
    fn try_play_idle_animation(
        &mut self,
        prince: &Prince,
        katamari: &Katamari,
        rng: &mut RngState,
    ) -> bool {
        if !katamari.physics_flags.wheel_spin {
            if !katamari.physics_flags.immobile
                && katamari.get_max_speed_ratio() >= self.params.max_speed_ratio_to_idle
            {
                // if moving too fast to enter the idle animation:
                return false;
            }

            let idle_id = if prince.input_avg_push_len > 0.0 || prince.get_input_avg_len() <= 0.0 {
                if prince.get_is_huffing() {
                    AnimationId::IdleHuff
                } else {
                    self.next_idle_animation
                }
            } else {
                AnimationId::Pinch
            };

            self.play(idle_id, 1.0, false, rng);
        }

        true
    }

    /// Checks the conditions for playing the `Pinch` animation, and if they are met, plays it.
    /// Returns `true` if the `Pinch` animation was successfully played.
    /// offset: 0x57d10
    fn try_play_pinch_animation(&mut self, prince: &Prince, rng: &mut RngState) -> bool {
        if prince.get_input_avg_len() > 0.0 && prince.input_avg_push_len == 0.0 {
            self.play(AnimationId::Pinch, 1.0, false, rng);
            true
        } else {
            false
        }
    }

    /// Tells Unity to play the animation with id `id` at the speed `speed`.
    /// offset: 0x57b20
    fn play(&mut self, id: AnimationId, speed: f32, repeat: bool, rng: &mut RngState) {
        if !id.is_idle() {
            self.advance_idle_animation(rng);
        }

        // (??): what is this doing
        if !repeat {
            if self.id == id && self.speed == speed {
                return;
            }

            if self.unknown_0xf == 0 || self.alt_id != id {
                self.unknown_0xf = 1;
                self.alt_id = id;
                return;
            }

            self.unknown_0xf += 1;
            if self.unknown_0xf < 4 {
                return;
            }
        }

        self.speed = speed;
        self.id = id;
        self.alt_id = id;
        self.unknown_0xf = 0;

        if let Some(delegates) = &self.delegates {
            if let Some(play_animation) = delegates.clone().borrow().play_animation {
                let id_u8: u8 = id.into();
                play_animation(self.player_idx as i32, id_u8 as i32, speed, repeat as i32);
                return;
            }
        }

        log!("warning: tried to play animation without `AnimationState::delegates` set.");
    }

    /// A common pattern duplicated in several places in the original simulation.
    /// The idle animation is randomly rerolled, advancing the `rng2` RNG value.
    fn advance_idle_animation(&mut self, rng: &mut RngState) {
        self.next_idle_animation = if rng.get_rng2() & 1 == 1 {
            AnimationId::Idle1
        } else {
            AnimationId::Idle2
        };
    }
}
