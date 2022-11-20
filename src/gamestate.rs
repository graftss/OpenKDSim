use crate::{global::GlobalState, katamari::Katamari, camera::Camera, preclear::PreclearState, ending::EndingState, delegates::Delegates, prince::Prince, input::Input};

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
  pub inputs: [Input; PLAYERS],
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

  pub fn read_camera(&self, player: i32) -> &Camera {
    &self.cameras[player as usize]
  }

  pub fn write_camera(&mut self, player: i32) -> &mut Camera {
    &mut self.cameras[player as usize]
  }

  pub fn read_input(&self, player: i32) -> &Input {
    &self.inputs[player as usize]
  }

  pub fn write_input(&mut self, player: i32) -> &mut Input {
    &mut self.inputs[player as usize]
  }

  /// Mimicks the `SetGameTime` API function.
  pub fn set_game_time(&mut self, game_time_ms: i32, remain_time_ticks: i32, freeze: i32, cam_eff_1P: i32) {
    self.global.game_time_ms = game_time_ms;
    self.global.remain_time_ticks = remain_time_ticks;
    self.global.freeze = freeze > 0;
    self.cameras[0 as usize].set_cam_eff_1P(cam_eff_1P);
  }

  /// Mimicks the `GetPrice` API function.
  pub fn get_prince(&self, 
    player: i32,
    xx: &mut f32, xy: &mut f32, xz: &mut f32, 
    yx: &mut f32, yy: &mut f32, yz: &mut f32, 
    zx: &mut f32, zy: &mut f32, zz: &mut f32, 
    tx: &mut f32, ty: &mut f32, tz: &mut f32,
    view_mode: &mut i32, _face_mode: &mut i32,
    alarm_mode: &mut i32, alarm_type: &mut i32,
    hit_water: &mut i32, map_loop_rate: &mut f32,
  ) {
    let prince = self.read_prince(player);
    prince.get_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz, tx, ty, tz);
    *view_mode = prince.get_view_mode() as i32;

    // TODO: update `face_mode`

    let katamari = self.read_katamari(player);
    katamari.get_alarm(alarm_mode, alarm_type);
    *hit_water = katamari.is_in_water() as i32;

    *map_loop_rate = self.global.map_loop_rate;
  }
}
