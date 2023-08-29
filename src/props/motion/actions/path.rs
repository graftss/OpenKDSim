use gl_matrix::{common::Vec3, mat4, vec3};
use serde::{Deserialize, Serialize};

use crate::{
    constants::VEC3_Z_POS,
    macros::{vec3_from, vec3_unit_xz},
    math::acos_f32,
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

const INNATE_PROP_PATH_SPEEDS: [f32; 11] =
    [0.0, 1.0, 2.0, 4.0, 6.0, 8.0, 10.0, 15.0, 20.0, 40.0, 200.0];

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
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
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
    yaw_angle: f32,

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
}

impl FollowPath {
    /// The main update behavior for the `FollowPath` action.
    /// ofset: 0x399c0
    fn update(&mut self, prop: &mut Prop, global_path_state: GlobalPathState) {
        if prop.get_move_type().is_none() {
            return;
        }

        // propagate the path's global state's reversed-ness to each prop on that path
        let global_path = global_path_state.get_path(self.path_idx as usize);
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
            // TODO: update prop based on `self.state`
            prop.update_somethings_coming();
            // TODO: check if alt action should be applied
        }
    }

    fn update_by_state(
        &mut self,
        prop: &mut Prop,
        global_path_state: GlobalPathState,
        mission: Mission,
    ) {
        match self.state {
            FollowPathState::Init => self.update_state_init(prop, global_path_state, mission),
        }
    }

    /// offset: 0x3a330
    fn generic_init_path(&mut self, prop: &mut Prop, mission: Mission) {
        self.target_point_idx = 0;

        if !PathStage::has_paths(mission) {
            // no path data exists for this motion, so just give up trying to initialize the path
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

    /// State 0 update behavior. Initializes the path-based motion.
    /// offset: 0x39b50
    fn update_state_init(
        &mut self,
        prop: &mut Prop,
        _global_path_state: GlobalPathState,
        mission: Mission,
    ) {
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
            let speed_idx = NamePropConfig::get(prop.get_name_idx()).speed_idx;
            INNATE_PROP_PATH_SPEEDS[speed_idx as usize]
        } else {
            path.speed
        };

        PROP_PATH_DATA.load_next_target_point(self, prop, mission);

        // TODO: `pmot_current_facing_angle(prop)` is called here, but seemingly unused?
        // TODO: rest of function
    }

    /// offset: 0x37a20
    fn yaw_towards_target(&self, start: &Vec3, end: &Vec3) -> f32 {
        // TODO: should the first argument be `end` and the second be `start`, since we're doing
        // `start - end` here, and computing the vector from end to start?
        let lateral_target_unit = vec3_unit_xz!(vec3_from!(-, start, end));
        let similarity = vec3::dot(&lateral_target_unit, &self.forward);
        let angle = acos_f32(similarity);
        angle
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
