use crate::{global::GlobalState, katamari::Katamari, camera::Camera, preclear::PreclearState, ending::EndingState, delegates::Delegates, prince::Prince};

const PLAYERS: usize = 2;

#[derive(Debug, Default)]
pub struct GameState {
  pub global: GlobalState,
  pub katamaris: [Katamari; PLAYERS],
  pub princes: [Prince; PLAYERS],
  pub cameras: [Camera; PLAYERS],
  pub preclear: PreclearState,
  pub ending: EndingState,
  pub delegates: Delegates,
}

impl GameState {
  pub fn read_katamari(&self, player: i32) -> &Katamari {
    &self.katamaris[player as usize]
  }

  pub fn write_katamari(&mut self, player: i32) -> &mut Katamari {
    &mut self.katamaris[player as usize]
  }

  pub fn read_prince(&self, player: i32) -> &Prince {
    &self.princes[player as usize]
  }

  pub fn write_prince(&mut self, player: i32) -> &mut Prince {
    &mut self.princes[player as usize]
  }
}
