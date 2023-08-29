use gl_matrix::{common::Vec3, mat4, vec3};
use serde::{Deserialize, Serialize};

use crate::props::{
    motion::global_path::{GlobalPathFlags, GlobalPathState},
    prop::{Prop, PropAnimationType, PropFlags2},
};

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
    path_index: u16,

    /// (??) next path point index after reversing?
    /// offset: 0x18
    field_0x18: u16,

    /// offset: 0x32
    do_alt_motion: bool,

    /// offset: 0xc0
    target_point: Vec3,
}

impl FollowPath {
    /// The main update behavior for the `FollowPath` action.
    /// ofset: 0x399c0
    fn update(&mut self, prop: &mut Prop, global_path_state: GlobalPathState) {
        if prop.get_move_type().is_none() {
            return;
        }

        // propagate the path's global state's reversed-ness to each prop on that path
        let global_path = global_path_state.get_path(self.path_index as usize);
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

    fn update_by_state(&mut self, prop: &mut Prop, global_path_state: GlobalPathState) {
        match self.state {
            FollowPathState::Init => self.update_state_init(prop, global_path_state),
        }
    }

    fn update_state_init(&mut self, prop: &mut Prop, _global_path_state: GlobalPathState) {
        self.do_alt_motion = false;
        mat4::identity(&mut prop.rotation_mat);
        mat4::identity(&mut prop.init_rotation_mat);
        // TODO: `prop_motion_2/23_subroutine`
        prop.animation_type = PropAnimationType::MovingForward;
        vec3::copy(&mut prop.pos, &self.target_point);
    }
}
