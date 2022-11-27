use crate::macros::panic_log;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Normal = 0,
    Tutorial = 1,
    TutorialB = 2,
    Ending = 3,
    Load = 4,
}

impl From<u8> for GameMode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::Tutorial,
            2 => Self::TutorialB,
            3 => Self::Ending,
            4 => Self::Load,
            _ => {
                panic_log!("encountered unknown `GameMode` value: {}", value);
            }
        }
    }
}

impl GameMode {
    pub fn can_update_view_mode(&self) -> bool {
        match self {
            GameMode::Normal | GameMode::Tutorial | GameMode::TutorialB => true,
            GameMode::Ending | GameMode::Load => false,
        }
    }
}
