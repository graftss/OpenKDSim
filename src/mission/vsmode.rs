use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VsModeState {
    /// (??) Some kind of timer
    /// offset: 0x10bf10
    pub timer_0x10bf10: i16,
}
