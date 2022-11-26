const NUM_GLOBAL_PATH_STATES: usize = 256;

/// State about a single path, which may have multiple props moving along it at once.
/// Updating this state allows individual props on the path to affect all such props.
#[derive(Debug, Default, Clone)]
pub struct GlobalPath {
    /// Flags: &1:reversed, &2:stalled.
    /// offset: 0x0
    pub flags: u8,

    /// (??) If >0, all objects on the path are stalled.
    /// offset: 0x1
    pub stalled: u8,

    /// If true, all objects on the path are moving double speed.
    /// offset: 0x2
    pub double_speed: bool,
}

#[derive(Debug, Default)]
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
}

impl GlobalPathState {}
