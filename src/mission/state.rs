use super::{
    config::MissionConfig, ending::EndingState, stage::StageConfig, tutorial::TutorialState,
    vsmode::VsModeState,
};

#[derive(Debug, Default)]
pub struct MissionState {
    pub mission: Option<MissionConfig>,
    pub stage: Option<StageConfig>,
    pub ending: EndingState,
    pub tutorial: TutorialState,
    pub vsmode: VsModeState,
}
