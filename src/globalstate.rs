use crate::mission::Mission;

/// Miscellaneous global game state.
pub struct GlobalState {
  pub catch_count_b: i32,
  pub mission: Option<Mission>,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self { catch_count_b: 0, mission: None }
    }
}
