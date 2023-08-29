use serde::{Deserialize, Serialize};

const NUM_GLOBAL_PATH_STATES: usize = 256;

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct GlobalPathFlags: u8 {
        const Reversing = 0x1;
        const Unk_0x2 = 0x2;
    }
}

/// State about a single path, which may have multiple props moving along it at once.
/// Updating this state allows individual props on the path to affect all such props.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GlobalPath {
    /// Flags: &1:reversed, &2:stalled.
    /// offset: 0x0
    pub flags: GlobalPathFlags,

    /// (??) If >0, all objects on the path are stalled.
    /// offset: 0x1
    pub stalled: u8,

    /// If true, all objects on the path are moving double speed.
    /// offset: 0x2
    pub double_speed: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GlobalPathState {
    paths: Vec<GlobalPath>,
}

impl GlobalPathState {
    pub const MAX_PATHS: u32 = 256;

    /// Initialize the vector of global path states.
    /// The original simulation allows for 256 such paths.
    pub fn init(&mut self) {
        self.paths.clear();

        for _ in 0..Self::MAX_PATHS {
            self.paths.push(GlobalPath::default());
        }
    }

    pub fn get_path(&self, path_index: usize) -> &GlobalPath {
        &self.paths[path_index]
    }
}
