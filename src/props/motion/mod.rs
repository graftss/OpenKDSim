use std::{collections::HashMap, slice::Iter};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathStage {
    House,
    Town,
    World,
    UrsaMajor,
    ShopDemo,
    VsMode,
    Gameshow,
}

#[derive(Debug, Clone, Copy)]
pub struct MaxPathIndices {
    house: u16,
    town: u16,
    world: u16,
    ursa_major: u16,
    shopdemo: u16,
    vs: u16,
    gameshow: u16,
}

impl PathStage {
    const HOUSE_MISSIONS: [u16; 7] = [1, 2, 3, 11, 12, 13, 22];
    const TOWN_MISSIONS: [u16; 7] = [4, 5, 8, 14, 15, 16, 23];
    const WORLD_MISSIONS: [u16; 9] = [6, 7, 9, 10, 18, 19, 20, 21, 24];
    const URSA_MAJOR_MISSIONS: [u16; 1] = [17];
    const SHOPDEMO_MISSIONS: [u16; 1] = [25];
    const VSMODE_MISSIONS: [u16; 8] = [31, 32, 33, 34, 35, 36, 37, 38];
    const GAMESHOW_MISSIONS: [u16; 1] = [39];

    /// The `u16`-encoded missions that use this group of paths.
    pub fn missions(self) -> &'static [u16] {
        match self {
            PathStage::House => Self::HOUSE_MISSIONS.as_slice(),
            PathStage::Town => Self::TOWN_MISSIONS.as_slice(),
            PathStage::World => Self::WORLD_MISSIONS.as_slice(),
            PathStage::UrsaMajor => Self::URSA_MAJOR_MISSIONS.as_slice(),
            PathStage::ShopDemo => Self::SHOPDEMO_MISSIONS.as_slice(),
            PathStage::VsMode => Self::VSMODE_MISSIONS.as_slice(),
            PathStage::Gameshow => Self::GAMESHOW_MISSIONS.as_slice(),
        }
    }

    pub fn iter() -> Iter<'static, PathStage> {
        use self::PathStage::*;
        static VALUES: [PathStage; 7] = [House, Town, World, UrsaMajor, ShopDemo, VsMode, Gameshow];
        VALUES.iter()
    }
}

/// Compute the max path index in each
/// If a mission has no move type data, its max path index is recorded as `None`.
pub fn get_max_path_indices() -> HashMap<PathStage, u16> {
    let max_path_idx_by_mission: Vec<u16> = MISSION_MOVE_TYPES
        .iter()
        .map(|mission_data| mission_data.iter().max_by_key(|item| item.path_idx))
        .map(|max_item| max_item.map_or(0, |max_item| max_item.path_idx))
        .collect();

    println!("{}", max_path_idx_by_mission.len());

    let max_path_idx_over_missions = |indices: &[u16]| -> u16 {
        indices
            .iter()
            .map(|idx| *max_path_idx_by_mission.get(*idx as usize).unwrap_or(&0))
            .max()
            .unwrap_or(0)
    };

    let mut result = HashMap::<PathStage, u16>::new();
    for path_stage in PathStage::iter() {
        result.insert(
            *path_stage,
            max_path_idx_over_missions(path_stage.missions()),
        );
    }
    result
}

pub fn get_behavior_motion_actions(behavior: i16) -> [u16; 2] {
    BEHAVIOR_ACTIONS[behavior as usize]
}

pub enum RotationAxis {
    X,
    Y,
    Z,
}
