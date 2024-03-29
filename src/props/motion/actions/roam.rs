use gl_matrix::{
    common::{Mat4, Vec3},
    mat4, vec3,
};
use serde::{Deserialize, Serialize};

use crate::{
    collision::raycast_state::RaycastRef,
    constants::{FRAC_PI_180, FRAC_PI_2, PI, VEC3_Y_POS, VEC3_ZERO, VEC3_Z_NEG, VEC3_Z_POS},
    global::GlobalState,
    macros::{panic_log, set_translation, vec3_from, vec3_unit_xz},
    math::{
        acos_f32, mat4_from_rotation_sim, normalize_bounded_angle, vec3_inplace_add_vec,
        vec3_inplace_normalize, vec3_inplace_scale, vec3_inplace_zero_small,
    },
    props::{
        config::NamePropConfig,
        motion::actions::common::is_not_facing_target,
        prop::{Prop, PropAnimationType, PropFlags2},
    },
};

use super::MotionAction;

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
    UNK_lateral_forward_unit: Vec3,

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

impl MotionAction for Roam {
    fn should_do_alt_action(&self) -> bool {
        self.do_alt_action
    }

    fn get_zone(&self) -> Option<u8> {
        self.zone
    }
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
        raycast_ref: RaycastRef,
    ) {
        if prop.move_type.is_some() && !prop.get_flags2().contains(PropFlags2::Wobble) {
            // core motion update depending on the motion's state
            match self.state {
                RoamState::Init => self.update_state_init(prop, global_state, raycast_ref),
                RoamState::Roam => self.update_state_roam(prop, global_state, raycast_ref),
                RoamState::InitTurnInPlace => {
                    self.update_state_init_turn_in_place(prop, raycast_ref)
                }
                RoamState::TurnInPlace => self.update_state_turn_in_place(prop),
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
        raycast_ref: RaycastRef,
    ) {
        if prop.get_behavior() != Some(0x1f) {
            self.do_alt_action = false;
        }

        mat4::identity(&mut prop.rotation_mat);
        mat4::identity(&mut prop.init_rotation_mat);
        self.forward_unit = VEC3_Z_NEG;
        self.UNK_lateral_forward_unit = VEC3_Z_NEG;

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

        let temp = self.UNK_lateral_forward_unit;
        vec3::transform_mat4(
            &mut self.UNK_lateral_forward_unit,
            &temp,
            &facing_angle_yawrot,
        );

        self.right = VEC3_Z_POS;

        self.forward_speed = NamePropConfig::get(prop.get_name_idx()).get_innate_move_speed();

        prop.animation_type = PropAnimationType::MovingForward;
        self.turn_state = RoamTurnState::NotTurning;
        self.last_turn_direction = None;
        self.turn_timer = compute_rand_turn_timer(global_state.rng.get_rng1(), prop.get_ctrl_idx());

        self.zone = raycast_ref.borrow_mut().find_zone_below_point(
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

    /// offset: 0x3b1e0
    fn update_state_roam(
        &mut self,
        prop: &mut Prop,
        global_state: &mut GlobalState,
        raycast_ref: RaycastRef,
    ) {
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
            if self.update_forward_pos(prop, false, raycast_ref) {
                self.state = RoamState::InitTurnInPlace;
            }
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
                self.forward_before_turn_unit = self.UNK_lateral_forward_unit;

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
                self.update_forward_turn();
                if self.yaw_turned == self.yaw_target {
                    self.turn_state = RoamTurnState::DoneTurning;
                }
            }
            RoamTurnState::DoneTurning => {
                self.turn_state = RoamTurnState::NotTurning;
                self.turn_timer =
                    compute_rand_turn_timer(global_state.rng.get_rng1(), prop.get_ctrl_idx());
            }
        }
    }

    /// offset: 0x31f90
    fn update_forward_turn(&mut self) {
        let next_turn_yaw = self.yaw_turned + self.yaw_speed;
        let yaw_target = self.yaw_target;

        let done_turning = if yaw_target >= 0.0 {
            yaw_target < next_turn_yaw
        } else {
            next_turn_yaw < yaw_target
        };

        let next_yaw = match done_turning {
            true => yaw_target,
            false => next_turn_yaw,
        };
        self.yaw_turned = normalize_bounded_angle(next_yaw);

        let mut yaw_rot = [0.0; 16];
        mat4::from_y_rotation(&mut yaw_rot, self.yaw_turned);
        vec3::transform_mat4(
            &mut self.UNK_lateral_forward_unit,
            &self.forward_before_turn_unit,
            &yaw_rot,
        );
    }

    /// Moving the roaming prop forwards while staying within its zone.
    /// If this function returns `true`, then the prop should stop moving forward
    /// and enter the "turn in place" state instead.
    /// offset: 0x31a30
    fn update_forward_pos(
        &mut self,
        prop: &mut Prop,
        unk_flag: bool,
        raycast_ref: RaycastRef,
    ) -> bool {
        let bbox_max = prop.get_aabb_max();
        let bbox_max_y = bbox_max[1];

        self.unk_align_height_with_zone(prop, raycast_ref.clone());
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

        let mut raycast = raycast_ref.borrow_mut();
        raycast.load_ray(&next_pos, &below_next_pos);

        if raycast.ray_hits_zone(prop.get_unattached_transform()) == 0 {
            // TODO_DOC: what is the y coordinate bump doing?
            let mut moved_unit = vec3_from!(-, prop.pos, prop.last_pos);
            moved_unit[1] += bbox_max_y + bbox_max_y;
            vec3_inplace_normalize(&mut moved_unit);

            let mut ray_start = prop.last_pos;
            ray_start[1] -= bbox_max_y;

            let mut pos = prop.pos;
            pos[1] -= bbox_max_y;
            let mut ray_end = [0.0; 3];
            vec3::scale_and_add(&mut ray_end, &pos, &moved_unit, bbox_max[2]);

            raycast.load_ray(&ray_start, &ray_end);

            let next_pos_in_zone = raycast.ray_hits_zone(prop.get_unattached_transform());
            if next_pos_in_zone != 0 {
                prop.pos = raycast.get_closest_hit().unwrap().impact_point;
                prop.pos[1] += bbox_max_y.abs();
                self.forward_unit = self.UNK_lateral_forward_unit;
            } else {
                prop.pos = prop.last_pos;
                return true;
            }
        } else {
            let current_zone = raycast.find_zone_below_point(
                &prop.pos,
                prop.get_radius() * 5.0,
                prop.get_unattached_transform(),
            );

            if self.zone != current_zone {
                prop.pos = prop.last_pos;
                return true;
            }

            prop.pos = raycast.get_closest_hit().unwrap().impact_point;
            prop.pos[1] += bbox_max_y.abs();
        }

        // either `next_pos_in_zone != 0` from the if branch above,
        // or `self.zone == current_zone` from the else branch above.
        self.moving_duration += 1;
        // TODO_PARAM
        if self.moving_duration > 0x3c {
            self.just_started_moving = false;
        }

        if !unk_flag {
            let similarity = vec3::dot(&self.right, &self.forward_unit);

            let mut angle = acos_f32(similarity);
            if self.forward_unit[0] < 0.0 {
                angle = -angle;
            }
            prop.rotation_vec[1] = angle;
        }

        false
    }

    // offset: 0x30ec0
    fn unk_align_height_with_zone(&mut self, prop: &mut Prop, raycast_ref: RaycastRef) {
        let radius = prop.get_radius();
        let center = prop.pos;
        let mut pitch = [0.0; 16];

        const MAX_ITER: i32 = 8;

        for i in 1..=MAX_ITER {
            let mut top = [0.0; 3];

            let mut prop_rot = prop.get_unattached_transform().clone();
            set_translation!(prop_rot, VEC3_ZERO);

            vec3::transform_mat4(&mut top, &VEC3_Y_POS, &prop_rot);
            vec3_inplace_scale(&mut top, (-i as f32) * radius);
            vec3_inplace_add_vec(&mut top, &center);

            let res =
                self.unk_compute_pitch_rotation(prop, &center, &top, &mut pitch, &raycast_ref);
            if res {
                break;
            }

            if i == 8 {
                return;
            }
        }

        if !NamePropConfig::get(prop.get_name_idx()).lock_pitch {
            prop.rotation_mat = pitch;
        }

        vec3::transform_mat4(
            &mut self.forward_unit,
            &self.UNK_lateral_forward_unit,
            &pitch,
        );

        let raycast = raycast_ref.borrow();
        let hit = raycast.get_closest_hit().unwrap();
        prop.pos[1] = hit.normal_unit[1] * prop.get_aabb_max_y().abs() + hit.impact_point[1];
    }

    /// offset: 0x30ba0
    fn unk_compute_pitch_rotation(
        &mut self,
        prop: &mut Prop,
        center: &Vec3,
        top: &Vec3,
        pitch_mat: &mut Mat4,
        raycast_ref: &RaycastRef,
    ) -> bool {
        let mut raycast = raycast_ref.borrow_mut();

        raycast.load_ray(&center, &top);
        if raycast.ray_hits_zone(prop.get_unattached_transform()) != 0 {
            let hit = raycast.get_closest_hit().unwrap();

            let mut normal_unit = hit.normal_unit;
            vec3_inplace_zero_small(&mut normal_unit, 1e-5);

            // TODO_PITCH: is it correct to negate the z coordinate?
            let mut normal_lateral = vec3_unit_xz!(normal_unit);
            normal_lateral[2] = -normal_lateral[2];

            let pitch_angle = acos_f32(hit.normal_unit[1]);
            mat4_from_rotation_sim(pitch_mat, -pitch_angle, &normal_lateral);

            return true;
        } else {
            mat4::identity(pitch_mat);

            return false;
        }
    }

    /// offset: 0x3b340
    fn update_state_init_turn_in_place(&mut self, prop: &mut Prop, raycast_ref: RaycastRef) {
        const TURN_ANGLES_DEGREES: [f32; 7] = [45.0, -45.0, 90.0, -90.0, 135.0, -135.0, 180.0];
        let mut raycast = raycast_ref.borrow_mut();

        self.yaw_target = 0.0;
        self.forward_before_turn_unit = self.UNK_lateral_forward_unit;

        for turn_angle_deg in TURN_ANGLES_DEGREES {
            let turn_angle = turn_angle_deg * FRAC_PI_180;
            self.yaw_target = turn_angle;

            let mut yaw_rot_mat = [0.0; 16];
            mat4::from_y_rotation(&mut yaw_rot_mat, turn_angle);

            vec3::transform_mat4(
                &mut self.forward_unit,
                &self.forward_before_turn_unit,
                &yaw_rot_mat,
            );

            raycast.find_zone_below_point(
                &prop.pos,
                prop.get_radius(),
                &prop.get_unattached_transform(),
            );

            // TODO_PARAM
            let lookahead_dist = self.forward_speed * 5.0;
            let mut lookahead_pt = [0.0; 3];
            vec3::scale_and_add(
                &mut lookahead_pt,
                &prop.pos,
                &self.forward_unit,
                lookahead_dist,
            );

            let lookahead_zone = raycast.zone_containing_prop_at(prop, &lookahead_pt);
            if lookahead_zone == self.zone {
                break;
            }
        }

        let yaw_target = self.yaw_target;

        // TODO_PARAM
        // the number of frames it will take for the prop's yaw angle to reach its target
        const TURN_FRAMES: u32 = 12;
        let yaw_speed = self.yaw_target / (TURN_FRAMES as f32);

        self.yaw_turned = 0.0;
        self.yaw_speed = yaw_speed;

        self.yaw_turned = if yaw_speed >= 0.0 {
            PI.min(yaw_speed).min(yaw_target)
        } else {
            (-PI).max(yaw_speed).max(yaw_target)
        };

        let mut target_rot_mat = [0.0; 16];
        mat4::from_y_rotation(&mut target_rot_mat, self.yaw_turned);

        vec3::transform_mat4(
            &mut self.UNK_lateral_forward_unit,
            &self.forward_before_turn_unit,
            &target_rot_mat,
        );
        vec3_inplace_normalize(&mut self.UNK_lateral_forward_unit);

        self.forward_unit = self.UNK_lateral_forward_unit;

        let lateral_forward_unit = vec3_unit_xz!(self.forward_unit);
        let similarity = vec3::dot(&lateral_forward_unit, &self.right);
        let mut facing_angle = acos_f32(similarity);

        if !is_not_facing_target(facing_angle, &lateral_forward_unit, &self.right) {
            facing_angle = -facing_angle;
        }

        prop.rotation_vec[1] = facing_angle;
        self.state = RoamState::TurnInPlace;
    }

    /// offset: 0x3bcd0
    fn update_state_turn_in_place(&mut self, prop: &mut Prop) {
        let next_yaw_turned = self.yaw_turned + self.yaw_speed;
        let yaw_target = self.yaw_target;

        self.yaw_turned = if self.yaw_speed >= 0.0 {
            PI.min(next_yaw_turned).min(yaw_target)
        } else {
            (-PI).max(next_yaw_turned).max(yaw_target)
        };

        let mut yaw_rot_mat = [0.0; 16];
        mat4::from_y_rotation(&mut yaw_rot_mat, self.yaw_turned);

        vec3::transform_mat4(
            &mut self.UNK_lateral_forward_unit,
            &self.forward_before_turn_unit,
            &yaw_rot_mat,
        );
        vec3_inplace_normalize(&mut self.UNK_lateral_forward_unit);

        self.forward_unit = self.UNK_lateral_forward_unit;

        let lateral_forward_unit = vec3_unit_xz!(self.forward_unit);
        let similarity = vec3::dot(&lateral_forward_unit, &self.right);
        let mut facing_angle = acos_f32(similarity);

        if !is_not_facing_target(facing_angle, &lateral_forward_unit, &self.right) {
            facing_angle = -facing_angle;
        }

        prop.rotation_vec[1] = facing_angle;

        if self.yaw_turned == self.yaw_target {
            self.state = RoamState::Roam;
            self.just_started_moving = true;
            self.moving_duration = 0;
        }
    }
}
