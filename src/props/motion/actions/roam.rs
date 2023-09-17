use gl_matrix::{common::Vec3, mat4, vec3};
use serde::{Deserialize, Serialize};

use crate::{
    collision::raycast_state::RaycastRef,
    constants::{VEC3_ZERO, VEC3_Z_NEG, VEC3_Z_POS},
    global::GlobalState,
    macros::set_translation,
    math::acos_f32,
    props::{
        config::NamePropConfig,
        motion::actions::common::is_not_facing_target,
        prop::{Prop, PropAnimationType, PropFlags2},
    },
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
enum RoamState {
    InitRoam = 0,
    Roam = 1,
    InitTurnInPlace = 2,
    TurnInPlace = 3,
}

impl Default for RoamState {
    fn default() -> Self {
        Self::InitRoam
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

    /// offset: 0xa
    wait_timer: u16,

    /// offset: 0xc
    wait_flags: u8,

    /// offset: 0x20
    forward_unit: Vec3,

    /// offset: 0x30
    forward_unit_copy: Vec3,

    /// offset: 0x40
    right: Vec3,

    /// offset: 0x50
    forward_before_turn_unit: Vec3,

    /// offset: 0x70
    speed: f32,

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compute_rand_turn_timer() {
        assert_eq!(compute_rand_turn_timer(2355903381, 0x12c), 0x227);
        assert_eq!(compute_rand_turn_timer(74768733, 0x12c), 0x1aa);
        assert_eq!(compute_rand_turn_timer(3452927929, 0x176), 0xe1);
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
                RoamState::InitRoam => self.update_state_init(prop, global_state, raycasts),
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

        self.speed = NamePropConfig::get(prop.get_name_idx()).get_innate_move_speed();

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
            self.wait_flags = 1;
            self.wait_timer = 0x1e;
        } else {
            self.wait_flags = 0;
            self.wait_timer = 0;
        }

        self.state = RoamState::Roam;
        self.just_started_moving = false;
        self.moving_duration = 0;
        prop.has_motion = true;
    }
}
