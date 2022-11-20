#[derive(Debug, Default)]
pub struct OujiState {
  pub dash_start: u8,
  pub wheel_spin_start: u8,
  pub dash: u8,
  pub wheel_spin: u8,
  pub jump_180: u8,
  pub sw_speed_disp: u8,
  pub climb_wall: u8,
  pub dash_tired: u8,
  pub camera_mode: u8,
  pub dash_effect: u8,
  pub hit_water: u8,
  pub submerge: u8,
  pub camera_state: u8,
  pub jump_180_leap: u8,
  pub brake: u8,
  pub tutorial_flag_1: u8,
  pub tutorial_flag_2: u8,
  pub tutorial_trigger_1: u8,
  pub tutorial_trigger_2: u8,
  pub power_charge: u8,
  pub fire_to_enemy: u8,
  pub search: u8,
  pub attack_1: u8,
  pub attack_2: u8,
  pub tarai: u8,
  pub attack_wait: u8,
  pub vs_attack: u8,
}

#[derive(Debug, Default)]
pub struct Prince {
  /// The player index controlling this prince.
  /// offset: 0x0
  player: u8, 

  /// Various 1-byte fields that are shared with the Unity code.
  /// offset: 0xa2
  pub oujistate: OujiState,

  /// The previous frame's values of various 1-byte fields that are shared with the Unity code.
  /// offset: 0xbd
  last_oujistate: OujiState,
}

impl Prince {
  pub fn get_oujistate_ptr(&mut self, oujistate: &mut *mut OujiState, data_size: &mut i32) {
    *data_size = 0x1b;
    *oujistate = &mut self.oujistate as *mut OujiState;
  }
}
