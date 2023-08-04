use crate::mission::Mission;

use self::data::{
    behaviors::BEHAVIOR_ACTIONS,
    move_types::{MissionMoveType, MISSION_MOVE_TYPES},
};

pub mod actions;
pub mod data;
pub mod global_path;
pub mod name_idx;

// TODO_REFACTOR: the `motion` module is really disorganized, need to move stuff around before
// scaling it up to include all motion actions

pub fn get_mission_move_type_data(mission: Mission, move_type: u16) -> Option<MissionMoveType> {
    if let Some(mission_move_types) = MISSION_MOVE_TYPES.get(mission as u8 as usize) {
        mission_move_types
            .get(move_type as usize)
            .map(|result| result.clone())
    } else {
        None
    }
}

pub fn get_behavior_motion_actions(behavior: i16) -> [u16; 2] {
    BEHAVIOR_ACTIONS[behavior as usize]
}

pub enum RotationAxis {
    X,
    Y,
    Z,
}
