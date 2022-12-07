use crate::{
    delegates::DelegatesRef,
    global::rng::RngState,
    player::{
        camera::{mode::CameraMode, CamR1JumpState, Camera},
        katamari::Katamari,
    },
};

use super::{Prince, PrinceTurnType, PrinceViewMode};

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

pub struct AnimationParams {
    /// The highest the katamari's max speed ratio can be while still performing
    /// an idle animation.
    /// default: 0.05
    /// offset: 0x71578 (used at 0x57c74)
    pub max_speed_ratio_to_idle: f32,
}

impl Default for AnimationParams {
    fn default() -> Self {
        Self {
            max_speed_ratio_to_idle: f32::from_bits(0x3d4ccccd), // 0.05
        }
    }
}

/// A struct in the original simulation that seems to mostly hold animation behavior.
/// Since a lot of animation logic in Reroll is handled by Unity, this struct was
/// mostly unused, and mainly just lets Unity know which animations to play.
pub struct AnimationState {
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

impl AnimationState {
    pub fn update(
        &mut self,
        prince: &Prince,
        katamari: &Katamari,
        camera: &Camera,
        rng: &RngState,
    ) {
        // TODO_VS: `prince_update_animation:26-141` (it seems like most or all of this is for vs mode/unused)
        let (animation_id, force_play) = if camera.get_mode() == CameraMode::R1Jump {
            // case 1: play r1 jump animation
            let height_ratio = camera.get_r1_jump_height_ratio();
            match camera.get_r1_jump_state() {
                Some(CamR1JumpState::Rising) if height_ratio < 0.5 => (AnimationId::Flip, false),
                _ => (AnimationId::R1Jump, height_ratio <= 0.0),
            }
        } else if !katamari.physics_flags.airborne {
            if prince.get_view_mode() == PrinceViewMode::L1Look {
                (AnimationId::L1Look, false)
            } else {
                // match prince.turn_type {
                //     PrinceTurnType::None => {

                //     },
                //     PrinceTurnType::LeftStickUp => todo!(),
                //     PrinceTurnType::RightStickUp => todo!(),
                //     PrinceTurnType::LeftStickDown => todo!(),
                //     PrinceTurnType::RightStickDown => todo!(),
                //     PrinceTurnType::Flip => todo!(),
                //     PrinceTurnType::BothSticks => todo!(),
                // }
                todo!()
            }
        } else {
            (AnimationId::Idle1, true)
        };
    }

    /// offset: 0x57cde
    fn try_start_idle_animation(
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

            let idle_id = if prince.input_avg_push_len > 0.0 || prince.input_avg_len <= 0.0 {
                if prince.is_huffing {
                    self.next_idle_animation
                } else {
                    AnimationId::IdleHuff
                }
            } else {
                AnimationId::Pinch
            };

            self.play(idle_id, false, 1.0, rng);
        }

        true
    }

    /// Tells Unity to play the animation with id `id` at the speed `speed`.
    /// offset: 0x57b20
    fn play(&mut self, id: AnimationId, repeat: bool, speed: f32, rng: &mut RngState) {
        if !id.is_idle() {
            self.advance_idle_animation(rng);
        }

        // TODO: what is this doing
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
            }
        }
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
