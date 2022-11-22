const NUM_GLOBAL_PATH_STATES: usize = 256;

/// State about a single path, which may have multiple props moving along it at once.
/// Updating this state allows individual props on the path to affect all such props.
#[derive(Debug, Default, Clone)]
pub struct GlobalPathState {
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

impl GlobalPathState {
    /// Initialize the vector of global path states in the `GameState`.
    pub fn init(vec: &mut Vec<GlobalPathState>) {
        for _ in 0..256 {
            vec.push(GlobalPathState::default());
        }
    }
}
