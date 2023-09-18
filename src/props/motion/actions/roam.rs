use gl_matrix::{common::Vec3, mat4, vec3};
use serde::{Deserialize, Serialize};

use crate::{
    collision::raycast_state::RaycastRef,
    constants::{FRAC_PI_2, VEC3_ZERO, VEC3_Z_NEG, VEC3_Z_POS},
    global::GlobalState,
    macros::{panic_log, set_translation, vec3_from},
    math::acos_f32,
    props::{
        config::NamePropConfig,
        motion::actions::common::is_not_facing_target,
        prop::{Prop, PropAnimationType, PropFlags2},
    },
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
enum RoamState {
    Init = 0,
    Roam = 1,
    InitTurnInPlace = 2,
    TurnInPlace = 3,
}

impl Default for RoamState {
    fn default() -> Self {
        Self::Init
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
enum RoamTurnState {
    NotTurning,
    Turning,
    DoneTurning,
}

impl Default for RoamTurnState {
    fn default() -> Self {
        Self::NotTurning
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
enum RoamTurnDirection {
    Right = 0,
    Left = 1,
}

impl From<u32> for RoamTurnDirection {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Right,
            1 => Self::Left,
            _ => {
                panic_log!("invalid roam turn direction");
            }
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct WaitFlags: u8 {
        const CanWait = 0x1;
        const IsWaiting = 0x2;
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Roam {
    // TODO_REFACTOR: once more motion actions are implemented, it should be possible to factor
    // this (and potentially other fields common to many motion actions) into a sub-struct
    // that's common to multiple motion actions
    /// `prop.motion_state` offset 0x32
    do_alt_action: bool,

    /// offset: 0x0
    state: RoamState,

    /// offset: 0x1
    turn_state: RoamTurnState,

    /// offset: 0x2
    turn_timer: u16,

    /// offset: 0x4
    moving_duration: u16,

    /// offset: 0x6
    just_started_moving: bool,

    /// offset: 0x7
    last_turn_direction: Option<RoamTurnDirection>,

    /// offset: 0x8
    zone: Option<u8>,

    /// When the prop is waiting, counts down to when the prop will start moving again.
    /// When the prop is moving, counts down to when the prop will start waiting.
    /// offset: 0xa
    wait_timer: u16,

    /// offset: 0xc
    wait_flags: WaitFlags,

    /// offset: 0x20
    forward_unit: Vec3,

    /// offset: 0x30
    forward_unit_copy: Vec3,

    /// offset: 0x40
    right: Vec3,

    /// offset: 0x50
    forward_before_turn_unit: Vec3,

    /// offset: 0x70
    forward_speed: f32,

    /// offset: 0x74
    yaw_turned: f32,

    /// offset: 0x78
    yaw_target: f32,

    /// offset: 0x7c
    yaw_speed: f32,

    /// offset: 0x80
    scary_kat_vol_m3: f32,
}

/// Compute the random length of the timer counting down before a roaming NPC turns.
/// This was probably a macro in the original simulation; see e.g. offset 0x3b117.
pub fn compute_rand_turn_timer(rng1: u32, ctrl_idx: u16) -> u16 {
    let mut result = rng1;
    result += ctrl_idx as u32;
    result %= 0x100;
    result *= 0x23a;
    result /= 0xff;
    result += 0x78;
    result as u16
}

/// Compute the random length of a roaming NPC's stationary wait, in frames.
/// offset: 0x3b2a
pub fn compute_rand_wait_duration(rng1: u32) -> u16 {
    let mut result = rng1;
    result &= 0xff;
    result *= 0x1e;
    result /= 0xff;
    result += 0x1e;
    result as u16
}

/// Compute the random time a roaming NPC moves until it waits, in frames.
/// offset: 0x3b23a
pub fn compute_rand_move_duration(rng1: u32) -> u16 {
    let mut result = rng1;
    result &= 0xff;
    result *= 0x3c;
    result /= 0xff;
    result += 0x78;
    result as u16
}

/// Compute the random turn direction of a roaming NPC.
/// offset: 0x3c06a
fn compute_rand_turn_dir(rng1: u32, ctrl_idx: u16) -> RoamTurnDirection {
    ((rng1 + ctrl_idx as u32) % 2).into()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compute_rand_turn_timer() {
        assert_eq!(compute_rand_turn_timer(2355903381, 0x12c), 0x227);
        assert_eq!(compute_rand_turn_timer(74768733, 0x12c), 0x1aa);
        assert_eq!(compute_rand_turn_timer(3452927929, 0x176), 0xe1);
    }

    #[test]
    fn test_compute_rand_wait_duration() {
        assert_eq!(compute_rand_wait_duration(952573193), 0x1f);
        assert_eq!(compute_rand_wait_duration(1378020957), 0x28);
        assert_eq!(compute_rand_wait_duration(964385473), 0x34);
    }

    #[test]
    fn test_compute_rand_move_duration() {
        assert_eq!(compute_rand_move_duration(1317998337), 0x78);
        assert_eq!(compute_rand_move_duration(2506520697), 0x94);
        assert_eq!(compute_rand_move_duration(419735713), 0x9d);
    }

    #[test]
    fn test_compute_rand_turn_dir() {
        assert_eq!(
            compute_rand_turn_dir(3724529093, 0x165),
            RoamTurnDirection::Right
        );
        assert_eq!(
            compute_rand_turn_dir(2387665901, 0x112),
            RoamTurnDirection::Left
        );
        assert_eq!(
            compute_rand_turn_dir(3710434989, 0x165),
            RoamTurnDirection::Right
        );
    }
}

impl Roam {
    /// offset: 0x3ac60
    pub fn update(
        &mut self,
        prop: &mut Prop,
        global_state: &mut GlobalState,
        raycasts: RaycastRef,
    ) {
        if prop.move_type.is_some() && !prop.get_flags2().contains(PropFlags2::Wobble) {
            // core motion update depending on the motion's state
            match self.state {
                RoamState::Init => self.update_state_init(prop, global_state, raycasts),
                RoamState::Roam => todo!(),
                RoamState::InitTurnInPlace => todo!(),
                RoamState::TurnInPlace => todo!(),
            }

            prop.update_somethings_coming();

            if prop.get_behavior() != Some(0x1f) {
                self.check_alt_action_trigger(prop);
            }
        }
    }

    /// offset: 0x3c0b0
    fn check_alt_action_trigger(&mut self, prop: &mut Prop) {
        if prop.alt_motion_action.is_some() {
            // TODO_ALT
        }
    }

    /// offset: 0x3ad20
    fn update_state_init(
        &mut self,
        prop: &mut Prop,
        global_state: &mut GlobalState,
        raycasts: RaycastRef,
    ) {
        if prop.get_behavior() != Some(0x1f) {
            self.do_alt_action = false;
        }

        mat4::identity(&mut prop.rotation_mat);
        mat4::identity(&mut prop.init_rotation_mat);
        self.forward_unit = VEC3_Z_NEG;
        self.forward_unit_copy = VEC3_Z_NEG;

        let mut prop_rot = prop.get_unattached_transform().clone();
        set_translation!(prop_rot, VEC3_ZERO);

        let mut prop_forward = [0.0; 3];
        vec3::transform_mat4(&mut prop_forward, &VEC3_Z_POS, &prop_rot);

        let mut facing_angle = acos_f32(prop_forward[2]);
        if is_not_facing_target(facing_angle, &prop_forward, &VEC3_Z_POS) {
            facing_angle *= -1.0;
        }

        let mut facing_angle_yawrot = [0.0; 16];
        mat4::from_y_rotation(&mut facing_angle_yawrot, facing_angle);

        let temp = self.forward_unit_copy;
        vec3::transform_mat4(&mut self.forward_unit_copy, &temp, &facing_angle_yawrot);

        self.right = VEC3_Z_POS;

        self.forward_speed = NamePropConfig::get(prop.get_name_idx()).get_innate_move_speed();

        prop.animation_type = PropAnimationType::MovingForward;
        self.turn_state = RoamTurnState::NotTurning;
        self.last_turn_direction = None;
        self.turn_timer = compute_rand_turn_timer(global_state.rng.get_rng1(), prop.get_ctrl_idx());

        self.zone = raycasts.borrow_mut().find_zone_below_point(
            &prop.pos,
            prop.get_radius(),
            &prop.get_unattached_transform(),
        );

        if self.zone.is_none() {
            // TODO_LOW: can we actually return from here? the original simulation doesn't, but
            // it seems to have the same effect (either way the prop ends its motion);
            return prop.end_motion();
        }

        // TODO_PARAM: the 0.1 is KatamariParams::prop_attach_vol_ratio
        self.scary_kat_vol_m3 = prop.get_compare_vol_m3() / 0.1;

        if prop.get_motion_action() == Some(0xf) {
            self.wait_flags = WaitFlags::CanWait;
            self.wait_timer = 0x1e;
        } else {
            self.wait_flags = WaitFlags::empty();
            self.wait_timer = 0;
        }

        self.state = RoamState::Roam;
        self.just_started_moving = false;
        self.moving_duration = 0;
        prop.has_motion = true;
    }

    fn update_state_roam(&mut self, prop: &mut Prop, global_state: &mut GlobalState) {
        let mut stationary = false;

        if self.wait_flags.contains(WaitFlags::CanWait) {
            if self.wait_flags.contains(WaitFlags::IsWaiting) {
                // If the prop is waiting:
                if self.wait_timer == 0 {
                    // If the wait timer is 0, stop waiting.
                    self.wait_flags.remove(WaitFlags::IsWaiting);
                    self.wait_timer = compute_rand_move_duration(global_state.rng.get_rng1());
                    prop.animation_type = PropAnimationType::MovingForward;
                } else {
                    // If the wait timer is nonzero, decrement it and keep waiting.
                    self.wait_timer -= 1;
                    stationary = true;
                }
            } else {
                // If the prop is moving:
                if self.wait_timer == 0 {
                    // If the wait timer is 0, start waiting.
                    self.wait_flags.insert(WaitFlags::IsWaiting);
                    self.wait_timer = compute_rand_wait_duration(global_state.rng.get_rng1());
                    prop.animation_type = PropAnimationType::Waiting;
                    stationary = true;
                } else {
                    // If the wait timer is nonzero, decrement it and keep moving.
                    self.wait_timer -= 1;
                }
            }
        }

        prop.stationary = stationary;

        if !stationary {
            prop.animation_type = PropAnimationType::MovingForward;
            self.update_forward_yaw(prop, global_state);
            /* TODO:
                if pmot_roam_update_forward_pos(prop, motion, false) {
                    self.state = RoamState::InitTurnInPlace;
                }
            */
        }
    }

    /// offset: 0x3bfb0
    fn update_forward_yaw(&mut self, prop: &mut Prop, global_state: &mut GlobalState) {
        match self.turn_state {
            RoamTurnState::NotTurning => {
                if self.turn_timer > 0 {
                    self.turn_timer -= 1;
                    return;
                }

                self.yaw_turned = 0.0;
                self.forward_before_turn_unit = self.forward_unit_copy;

                let turn_dir =
                    compute_rand_turn_dir(global_state.rng.get_rng1(), prop.get_ctrl_idx());
                self.last_turn_direction = Some(turn_dir);
                self.turn_state = RoamTurnState::Turning;

                // TODO_PARAM
                const YAW_SPEED: f32 = 0.05;
                const YAW_TARGET: f32 = FRAC_PI_2;

                match turn_dir {
                    RoamTurnDirection::Right => {
                        self.yaw_target = YAW_TARGET;
                        self.yaw_speed = YAW_SPEED;
                    }
                    RoamTurnDirection::Left => {
                        self.yaw_target = -YAW_TARGET;
                        self.yaw_speed = -YAW_SPEED;
                    }
                }
            }
            RoamTurnState::Turning => {
                // TODO: pmot_roam_update_forward_turn()
                if self.yaw_turned == self.yaw_target {
                    self.state = RoamState::InitTurnInPlace
                }
            }
            RoamTurnState::DoneTurning => {
                self.turn_state = RoamTurnState::NotTurning;
                self.turn_timer =
                    compute_rand_turn_timer(global_state.rng.get_rng1(), prop.get_ctrl_idx());
            }
        }
    }

    /// offset: 0x31a30
    fn update_forward_pos(&mut self, prop: &mut Prop, raycasts: RaycastRef) {
        // TODO: pmot_roam_align_height_with_zone()
        prop.last_pos = prop.pos;
        vec3::scale(
            &mut prop.trajectory_velocity,
            &self.forward_unit,
            self.forward_speed,
        );

        // cast a ray downwards from the prop's next position.
        let next_pos = vec3_from!(+, prop.trajectory_velocity, prop.pos);
        let mut below_next_pos = next_pos;
        below_next_pos[1] -= prop.get_radius() * 5.0;

        prop.pos = next_pos;

        raycasts.borrow_mut().load_ray(&next_pos, &below_next_pos);
        let next_pos_zone = raycasts
            .borrow_mut()
            .ray_hits_zone(&prop.get_unattached_transform());

        if next_pos_zone == 0 {}
    }
}
