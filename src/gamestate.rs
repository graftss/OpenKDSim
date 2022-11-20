use crate::{globalstate::GlobalState, katamari::Katamari, camera::Camera, preclear_mode::PreclearState};

const PLAYERS: usize = 2;

pub struct GameState {
  pub global: GlobalState,
  pub katamaris: [Katamari; PLAYERS],
  pub cameras: [Camera; PLAYERS],
  pub preclear: PreclearState,
}

impl Default for GameState {
    fn default() -> Self {
        Self { 
          global: Default::default(), 
          katamaris: Default::default(), 
          cameras: Default::default(), 
          preclear: Default::default(),
        }
    }
}

impl GameState {
  /// API function.
  pub fn GetKatamariCatchCountB(&self) -> i32 {
    self.global.catch_count_b
  }

  /// API function.
  /// This is divided by 100 because the reroll devs are morons.
  pub fn GetKatamariRadius(&self, player: usize) -> f32 {
    self.katamaris[player].get_radius() / 100.0
  }

  /// API function.
  pub fn GetKatamariDiameterInt(&self, player: usize) -> i32 {
    self.katamaris[player].get_diam_int()
  }

  pub fn GetKatamariDisplayRadius(&self, player: usize) -> f32 {
    self.katamaris[player].get_display_radius()
  }

  /// API function.
  pub fn GetKatamariVolume(&self, player: usize) -> f32 {
    self.katamaris[player].get_vol()
  }

  pub fn GetPreclearAlpha(&self) -> f32 {
    self.preclear.get_alpha()
  }
}
