use serde::{Deserialize, Serialize};

use crate::{
    collision::raycast_state::RaycastRef, global::GlobalState, mission::state::MissionState,
    props::prop::Prop,
};

use self::{path::FollowPath, roam::Roam, sway::SwayAction};

use super::global_path::GlobalPathState;

pub mod common;
pub mod path;
pub mod roam;
pub mod sway;

pub trait ActionUpdate {
    fn update(&mut self, prop: &mut Prop);
    fn should_do_alt_motion(&self) -> bool;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MotionAction {
    FollowPath(FollowPath),
    Roam(Roam),
    // TODO_LINK: in `pmot_misc_init` the prop decides if it's `stationary` based on its root prop's
    // link action this needs to happen for all misc motions
    MiscSway(SwayAction),
    Unimplemented(u16),
}

impl MotionAction {
    pub fn parse_id(action_id: u16) -> Self {
        match action_id {
            0x2 => Self::FollowPath(FollowPath::default()),
            0x3 => Self::Roam(Roam::default()),
            // TODO_BUG: there are several misc actions with `action_id` 0x16. this function also needs
            // access to the behavior id to distinguish between these misc actions.
            0x16 => Self::MiscSway(SwayAction::default()),
            _ => Self::Unimplemented(action_id),
        }
    }

    pub fn update(
        &mut self,
        prop: &mut Prop,
        gps: &GlobalPathState,
        mission_state: &MissionState,
        global_state: &mut GlobalState,
        raycasts: RaycastRef,
    ) {
        match self {
            MotionAction::FollowPath(follow_path) => {
                follow_path.update(prop, gps, mission_state.mission)
            }
            MotionAction::Roam(roam) => roam.update(prop, global_state, raycasts),
            MotionAction::MiscSway(sway) => sway.update(prop),
            MotionAction::Unimplemented(_) => {}
        }
    }
}
