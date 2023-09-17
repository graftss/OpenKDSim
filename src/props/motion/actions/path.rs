use gl_matrix::{common::Vec3, mat4, vec3};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{PI, VEC3_Z_POS},
    macros::{vec3_from, vec3_unit_xz},
    math::{acos_f32, normalize_bounded_angle, vec3_inplace_add_vec, vec3_inplace_normalize},
    mission::Mission,
    props::{
        config::NamePropConfig,
        motion::{
            data::{
                move_types::MISSION_MOVE_TYPES,
                prop_paths::{PathStage, PROP_PATH_DATA},
            },
            global_path::{GlobalPathFlags, GlobalPathState},
        },
        prop::{Prop, PropAnimationType, PropFlags2},
    },
};

use super::common::is_not_facing_target;

pub trait PathMotion {
    fn get_flags(&self) -> FollowPathFlags;
    fn get_path_idx(&self) -> u16;
    fn set_path_idx(&mut self, value: u16);
    fn get_target_point_idx(&self) -> u16;
    fn set_target_point_idx(&mut self, value: u16);
    fn set_target_point(&mut self, value: &Vec3);
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum FollowPathState {
    Init = 0,
    MoveTowardsTarget = 1,
    TurnInPlace = 2,
    // TODO: rename this when i understand what flag 0x2 means
    WaitWhileFlag0x2 = 3,
}

impl Default for FollowPathState {
    fn default() -> Self {
        Self::Init
    }
}

/// Moving forward on a path means moving towards points of higher index, and backward
/// means moving towards lower index points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PathDirection {
    Forward,
    Backward,
}

impl Default for PathDirection {
    fn default() -> Self {
        Self::Forward
    }
}

impl From<bool> for PathDirection {
    fn from(value: bool) -> Self {
        match value {
            // TODO: could be wrong
            false => PathDirection::Forward,
            true => PathDirection::Backward,
        }
    }
}

impl From<PathDirection> for bool {
    fn from(value: PathDirection) -> Self {
        match value {
            // TODO: could be wrong
            PathDirection::Forward => false,
            PathDirection::Backward => true,
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct FollowPathFlags: u8 {
        const Reversed = 0x1;
        const Unk_0x2 = 0x2;
        const Unk_0x8 = 0x8;
        const UpdatePitch = 0x10;
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FollowPath {
    /// offset: 0x0
    state: FollowPathState,

    /// (?????) If true, the prop is currently reversing direction along the path.
    /// Note that this can occur when the prop was previously moving in either direction; this
    /// flag just means that that direction is going to change.
    /// offset: 0x1
    reversing: bool,

    /// offset: 0x2
    target_point_idx: u16,

    /// offset: 0x4
    speed: f32,

    /// offset: 0x8
    yaw_current: f32,

    /// offset: 0xc
    yaw_target: f32,

    /// offset: 0x10
    yaw_speed: f32,

    /// offset: 0x14
    flags: FollowPathFlags,

    /// The direction a prop is moving along a path.
    /// offset: 0x15
    direction: PathDirection,

    /// offset: 0x16
    path_idx: u16,

    /// (??) next path point index after reversing?
    /// offset: 0x18
    field_0x18: u16,

    /// offset: 0x32
    do_alt_motion: bool,

    /// (??) Set to the prop's position when the path motion is initialized.
    /// offset: 0xb0
    init_pos: Vec3,

    /// The next target point along the path. The prop moves in a straight line towards this point.
    /// offset: 0xc0
    target_point: Vec3,

    /// The prop's velocity towards the next target point along the path.
    /// offset: 0xd0
    vel_to_target: Vec3,

    /// The prop's unit velocity towards the next target point along the path.
    /// offset: 0xe0
    vel_to_target_unit: Vec3,

    /// (??) The constant vector <0,0,1>, i.e. the forward direction in the prop's local space
    /// offset: 0xf0
    forward: Vec3,

    /// If a katamari with at least this volume comes near the prop, the prop will be scared.
    /// offset: 0x100
    scary_kat_vol_m3: f32,
}

/// (??) Computes the (yaw) angle from the `forward` vector at the point `pos` to the point `target`.
/// offset: 0x37a20
pub fn yaw_angle_to_target(forward: &Vec3, pos: &Vec3, target: &Vec3) -> f32 {
    let to_target_unit = vec3_unit_xz!(vec3_from!(-, pos, target));
    let similarity = vec3::dot(&to_target_unit, forward);
    let mut angle = acos_f32(similarity);

    if is_not_facing_target(angle, forward, &to_target_unit) {
        angle *= -1.0;
    }

    angle
}

impl FollowPath {
    /// The main update behavior for the `FollowPath` action.
    /// ofset: 0x399c0
    pub fn update(&mut self, prop: &mut Prop, gps: &GlobalPathState, mission: Mission) {
        if prop.get_move_type().is_none() {
            return;
        }

        // propagate the path's global state's reversed-ness to each prop on that path
        let global_path = gps.get_path(self.path_idx as usize);
        let global_reverse = global_path.flags.contains(GlobalPathFlags::Reversing);

        if global_reverse != Into::<bool>::into(self.direction) {
            self.flags.insert(FollowPathFlags::Reversed);
            self.reversing = true;
            self.direction = match global_reverse {
                true => PathDirection::Backward,
                false => PathDirection::Forward,
            }
        }

        self.flags.set(
            FollowPathFlags::Unk_0x2,
            global_path.flags.contains(GlobalPathFlags::Unk_0x2),
        );

        if !prop.get_flags2().contains(PropFlags2::Wobble) {
            // core motion update depending on the motion's state
            match self.state {
                FollowPathState::Init => self.update_state_init(prop, mission),
                FollowPathState::MoveTowardsTarget => {
                    self.update_state_move_towards_target(prop, gps, mission)
                }
                FollowPathState::TurnInPlace => todo!(),
                FollowPathState::WaitWhileFlag0x2 => todo!(),
            }

            prop.update_somethings_coming();
            // TODO: check if alt action should be applied
        }
    }

    /// Shared behaviour between several path-based motions.
    /// offset: 0x3a330
    fn generic_init_path(&mut self, prop: &mut Prop, mission: Mission) {
        self.target_point_idx = 0;

        if !PathStage::has_paths(mission) {
            return prop.end_motion();
        }

        self.path_idx =
            MISSION_MOVE_TYPES[mission as usize][prop.move_type.unwrap() as usize].path_idx;
        PROP_PATH_DATA.load_initial_target_point_idx(self, prop, mission);
        vec3::copy(&mut self.init_pos, &prop.pos);

        let failed_getting_point = PROP_PATH_DATA.get_mission_path_point(
            &mut self.target_point,
            mission,
            self.target_point_idx as usize,
            self.path_idx as usize,
        );

        if failed_getting_point {
            return prop.end_motion();
        }

        let to_target = vec3_from!(-, self.target_point, prop.pos);
        vec3::normalize(&mut self.vel_to_target_unit, &to_target);
        vec3::scale(
            &mut self.vel_to_target,
            &self.vel_to_target_unit,
            self.speed,
        );
        vec3::copy(&mut self.forward, &VEC3_Z_POS);

        let aabb_top = prop.get_aabb_max_y();
        self.target_point[1] += aabb_top;
    }

    /// State 0 update behavior. Initializes the `FollowPath` motion.
    /// offset: 0x39b50
    fn update_state_init(&mut self, prop: &mut Prop, mission: Mission) {
        self.do_alt_motion = false;
        mat4::identity(&mut prop.rotation_mat);
        mat4::identity(&mut prop.init_rotation_mat);
        self.generic_init_path(prop, mission);
        prop.animation_type = PropAnimationType::MovingForward;
        vec3::copy(&mut prop.pos, &self.target_point);

        let path = PROP_PATH_DATA
            .get_mission_path(mission, self.path_idx as usize)
            .unwrap();

        self.speed = if path.speed < 0.0 {
            NamePropConfig::get(prop.get_name_idx()).get_innate_move_speed()
        } else {
            path.speed
        };

        PROP_PATH_DATA.load_next_target_point(self, prop, mission);

        // TODO_LOW: `pmot_current_facing_angle(prop)` is called here, but seemingly unused?

        let init_yaw = yaw_angle_to_target(&self.forward, &prop.pos, &self.target_point);

        self.yaw_speed = 0.0;
        self.yaw_target = init_yaw;
        self.yaw_current = init_yaw;
        // TODO_PARAM: the 0.1 is KatamariParams::prop_attach_vol_ratio
        self.scary_kat_vol_m3 = prop.get_compare_vol_m3() / 0.1;
        prop.is_following_path = true;
        prop.has_motion = true;
        self.state = FollowPathState::MoveTowardsTarget;
    }

    /// State 1 update behavior. The "main" state that drives movement along the path.
    /// offset: 0x39ce0
    fn update_state_move_towards_target(
        &mut self,
        prop: &mut Prop,
        gps: &GlobalPathState,
        mission: Mission,
    ) {
        if self.flags.contains(FollowPathFlags::Unk_0x2) {
            self.state = FollowPathState::WaitWhileFlag0x2;
        } else if self.reversing {
            // TODO: `pmot_turn_towards_path_target(prop, false)`
            self.state = FollowPathState::TurnInPlace;
            self.reversing = false;
        } else {
            self.move_towards_target(prop, gps, mission);
        }
    }

    /// State 2 update behavior. Turns the prop laterally without moving it.
    /// offset: 0x39d20
    fn update_state_turn_in_place(&mut self, prop: &mut Prop) {
        // TODO: `let angle = self.pmot_path_update_yaw() + PI`
        let angle = 0.0;
        prop.rotation_vec[1] = normalize_bounded_angle(angle);
        if self.yaw_speed == 0.0 {
            self.state = FollowPathState::MoveTowardsTarget;
        }
    }

    /// State 3 update behavior. Does nothing until flag 0x2 is turned off, at which
    /// point the prop returns to the "move towards target" state.
    /// offset: 0x39d80
    fn update_state_wait_while_flag_0x2(&mut self, prop: &mut Prop) {
        if !self.flags.contains(FollowPathFlags::Unk_0x2) {
            prop.animation_type = PropAnimationType::MovingForward;
            self.state = FollowPathState::MoveTowardsTarget;
        }
    }
}

impl FollowPath {
    /// Returns `true` if the prop's `speed` (the distance it will move this tick)
    /// is greater than the distance to its target point.
    /// offset: 0x37610
    fn will_reach_target_pt(&mut self, prop: &mut Prop, speed: f32) -> bool {
        let dist_to_target = vec3::distance(&self.target_point, &prop.pos);
        return speed >= dist_to_target;
    }

    /// Apply the motion's yaw speed to its yaw angle, with the result of turning a prop
    /// slightly towards its target point.
    /// Returns the updated yaw angle.
    /// offset: 0x379a0
    fn apply_yaw_speed(&mut self) -> f32 {
        // TODO_PARAM: really multiplying by `30.0 * FRAME_TIME`
        self.yaw_current += self.yaw_speed * 1.0;

        let done_turning = if self.yaw_speed > 0.0 {
            self.yaw_current >= self.yaw_target
        } else {
            self.yaw_current <= self.yaw_target
        };

        if done_turning && self.yaw_target != self.yaw_current {
            self.yaw_current = self.yaw_target;
            self.yaw_speed = 0.0;
        }

        return normalize_bounded_angle(self.yaw_current);
    }

    /// Move the prop towards its target point.
    /// offset: 0x39440
    fn move_towards_target(
        &mut self,
        prop: &mut Prop,
        gps: &GlobalPathState,
        mission: Mission,
    ) -> bool {
        // TODO_PARAM: really multiplying by `30.0 * FRAME_TIME`
        let mut speed = self.speed * 1.0;
        if gps.get_path(self.path_idx as usize).double_speed {
            speed = speed + speed;
        }

        if self.flags.contains(FollowPathFlags::UpdatePitch) {
            // TODO: `self.pmot_path_update_pitch_if_flag_0x10(prop)`
        }

        if self.will_reach_target_pt(prop, speed) {
            // if we will reach the next target point:
            // teleport to that target point
            vec3::copy(&mut prop.pos, &self.target_point);

            // load the target point after that
            let found_next_target = PROP_PATH_DATA.load_next_target_point(self, prop, mission);
            if !found_next_target {
                return false;
            }

            if !NamePropConfig::get(prop.get_name_idx()).lock_pitch {
                // TODO: `self.pmot_update_pitch(prop)`
            } else {
                self.flags.remove(FollowPathFlags::UpdatePitch);
            }

            let yaw_current = yaw_angle_to_target(&self.forward, &prop.last_pos, &prop.pos);
            self.yaw_current = yaw_current;

            let yaw_target = yaw_angle_to_target(&self.forward, &prop.pos, &self.target_point);
            self.yaw_target = yaw_target;

            // compute the total change in yaw that the prop is required to make as it moves towards
            // its target. this value (along with the target yaw angle) is normalized to lie within
            // [-PI, PI].
            // TODO_REFACTOR: can't we just do `normalize_bounded_angle(yaw_target - yaw_current)`?
            let mut yaw_remain = yaw_target - yaw_current;
            if yaw_current > 0.0 && yaw_target < 0.0 && (yaw_target - yaw_current < -PI) {
                yaw_remain += 2.0 * PI;
                self.yaw_target += 2.0 * PI;
            } else if yaw_current < 0.0 && yaw_target > 0.0 && (yaw_target - yaw_current > PI) {
                yaw_remain -= 2.0 * PI;
                self.yaw_target -= 2.0 * PI;
            }

            let len_to_target = vec3::distance(&self.target_point, &prop.pos);
            let time_to_target = if speed == 0.0 {
                len_to_target / 30.0
            } else {
                len_to_target / speed
            };

            self.yaw_speed = if time_to_target != 0.0 {
                yaw_remain / time_to_target
            } else {
                yaw_remain
            };
        }

        let next_yaw = self.apply_yaw_speed();
        // TODO: is this right? if it is, then this PI could probably be added in `apply_yaw_speed`.
        prop.rotation_vec[1] = normalize_bounded_angle(next_yaw + PI);

        let mut vel_to_target_unit = vec3_from!(-, self.target_point, prop.pos);
        vec3_inplace_normalize(&mut vel_to_target_unit);
        self.vel_to_target_unit = vel_to_target_unit;
        vec3::scale(&mut self.vel_to_target, &self.vel_to_target_unit, speed);
        vec3_inplace_add_vec(&mut prop.pos, &self.vel_to_target);

        true
    }
}

impl PathMotion for FollowPath {
    fn get_flags(&self) -> FollowPathFlags {
        self.flags
    }

    fn get_path_idx(&self) -> u16 {
        self.path_idx
    }

    fn set_path_idx(&mut self, value: u16) {
        self.path_idx = value;
    }

    fn get_target_point_idx(&self) -> u16 {
        self.target_point_idx
    }

    fn set_target_point_idx(&mut self, value: u16) {
        self.target_point_idx = value;
    }

    fn set_target_point(&mut self, point: &Vec3) {
        vec3::copy(&mut self.target_point, point);
    }
}

#[cfg(test)]
mod test {
    use gl_matrix::common::Vec3;

    use crate::{
        constants::VEC3_Z_POS, macros::f32_close_enough,
        props::motion::actions::path::yaw_angle_to_target,
    };

    fn yaw_angle_to_target_test_case(pos: Vec3, target: Vec3, expected: f32) {
        let forward = VEC3_Z_POS;
        let observed = yaw_angle_to_target(&forward, &pos, &target);

        assert!(f32_close_enough!(expected, observed));
    }

    #[test]
    fn test_yaw_angle_to_target() {
        yaw_angle_to_target_test_case(
            [158.202179, -63.865898, 192.403198],
            [158.203995, -63.865898, 192.461899],
            -3.110669,
        );
        yaw_angle_to_target_test_case(
            [162.636993, -65.551697, 126.214401],
            [162.296997, -65.551697, 127.425903],
            2.867990,
        );
    }
}
