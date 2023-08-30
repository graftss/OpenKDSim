use serde::{Deserialize, Serialize};

use crate::{mission::state::MissionState, props::prop::Prop};

use self::{path::FollowPath, sway::SwayAction};

use super::global_path::GlobalPathState;

pub mod path;
pub mod sway;

pub trait ActionUpdate {
    fn update(&mut self, prop: &mut Prop);
    fn should_do_alt_motion(&self) -> bool;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MotionAction {
    FollowPath(FollowPath),
    // TODO_LINK: in `pmot_misc_init` the prop decides if it's `stationary` based on its root prop's
    // link action this needs to happen for all misc motions
    MiscSway(SwayAction),
    Unimplemented(u16),
}

impl MotionAction {
    pub fn parse_id(action_id: u16) -> Self {
        match action_id {
            0x2 => Self::FollowPath(FollowPath::default()),
            0x16 => Self::MiscSway(SwayAction::default()),
            _ => Self::Unimplemented(action_id),
        }
    }

    pub fn update(&mut self, prop: &mut Prop, gps: &GlobalPathState, mission_state: &MissionState) {
        match self {
            MotionAction::FollowPath(follow_path) => {
                follow_path.update(prop, gps, mission_state.mission)
            }
            MotionAction::MiscSway(sway) => sway.update(prop),
            MotionAction::Unimplemented(_) => {}
        }
    }
}
