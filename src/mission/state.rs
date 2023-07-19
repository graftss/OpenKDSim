use serde::{Deserialize, Serialize};

use super::{
    config::MissionConfig,
    ending::EndingState,
    stage::{Stage, StageConfig},
    tutorial::{TutorialMove, TutorialState},
    vsmode::VsModeState,
    GameMode, Mission,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MissionState {
    /// Mission-specific immutable values.
    #[serde(skip)]
    pub mission_config: MissionConfig,

    /// Stage-specific immutable values.
    #[serde(skip)]
    pub stage_config: StageConfig,

    /// Ending-specific state.
    pub ending: EndingState,

    /// Tutorial-specific state.
    pub tutorial: TutorialState,

    /// VS mode-specific state.
    pub vsmode: VsModeState,

    /// The current mission.
    /// offset: 0xff104
    pub mission: Mission,

    /// The *unique* id of the currently loaded area among all other areas in the game.
    /// The House stage has ids 0-4, Town has 5-8, and World has 9+.
    /// offset: 0xff106
    pub stage_area: u8,

    /// The current stage (which is the map - house, town, world, etc.)
    /// offset: 0xff108
    pub stage: Stage,

    /// The current loaded area of the current stage, where the smallest
    /// area of each stage is 0.
    /// offset: 0xff109
    pub area: u8,

    /// If true, the current mission is in VS mode.
    /// offset: 0xff0f1
    pub is_vs_mode: bool,

    /// (??) too lazy to document this right now
    /// offset: 0x10daf9
    pub vs_mission_idx: Option<u8>,

    /// The current game mode.
    /// offset: 0x10daf5
    pub gamemode: GameMode,
}

impl MissionState {
    pub fn is_tutorial(&self) -> bool {
        self.gamemode == GameMode::Tutorial
    }

    pub fn is_ending(&self) -> bool {
        self.gamemode == GameMode::Ending
    }

    pub fn set_gamemode(&mut self, gamemode: u8) {
        self.gamemode = gamemode.into();
    }

    pub fn mono_init_start(&mut self, mission: u8, area: u8, stage: u8) {
        self.mission = mission.into();
        self.stage = stage.into();
        self.area = area;

        // initialize the mission and stage configs
        MissionConfig::get(&mut self.mission_config, mission);
        StageConfig::get(&mut self.stage_config, stage);
    }

    /// Set the tutorial move `tut_move` as being held.
    pub fn set_tutorial_move_held(&mut self, tut_move: TutorialMove) {
        if self.is_tutorial() {
            self.tutorial.set_move_held(tut_move);
        }
    }
}
