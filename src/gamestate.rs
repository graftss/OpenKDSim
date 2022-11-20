use crate::{globalstate::GlobalState, katamari::Katamari, camera::Camera};

const PLAYERS: usize = 2;

pub struct GameState {
  pub global: GlobalState,
  pub katamaris: [Katamari; PLAYERS],
  pub cameras: [Camera; PLAYERS],
}

impl Default for GameState {
    fn default() -> Self {
        Self { global: Default::default(), katamaris: Default::default(), cameras: Default::default() }
    }
}
