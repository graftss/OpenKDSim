use serde::{Deserialize, Serialize};

use crate::{
    collision::raycast_state::RaycastRef, global::GlobalState, mission::state::MissionState,
    props::prop::Prop,
};

use self::{path::FollowPath, roam::Roam, sway::SwayAction, zone_trigger::ZoneTrigger};

use super::global_path::GlobalPathState;

pub mod common;
pub mod path;
pub mod roam;
pub mod sway;
pub mod zone_trigger;

pub trait ActionUpdate {
    /// Updates the prop's motion action.
    fn update(&mut self, prop: &mut Prop);

    // this might be worded wrongly? it's recording if the prop is in its alt motion
    fn should_do_alt_motion(&self) -> bool;

    /// Returns the zone associated to the prop's motion action.
    /// For example, if a prop randomly roams, the zone it roams in will be returned.
    /// Returns `None` if the action has no associated zone.
    fn get_zone(&self) -> Option<u8>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MotionAction {
    FollowPath(FollowPath),
    Roam(Roam),
    ZoneTrigger(ZoneTrigger),
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
            0x4 => Self::ZoneTrigger(ZoneTrigger::default()),
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
            MotionAction::ZoneTrigger(zone_trigger) => zone_trigger.update(prop),
            MotionAction::MiscSway(sway) => sway.update(prop),
            MotionAction::Unimplemented(_) => {}
        }
    }
}
