use std::fmt::Display;

use crate::mission::Mission;

/// Miscellaneous global game state.
#[derive(Debug)]
pub struct GlobalState {
  /// The "theme object" score in constellation levels (e.g. number of crabs in Make Cancer).
  pub catch_count_b: i32,

  /// The current mission.
  pub mission: Mission,

  /// Global forward movement speed multiplier.
  pub forwards_speed: f32,

  /// Global sideways movement speed multiplier.
  pub sideways_speed: f32,

  /// Global backwards movement speed multiplier.
  pub backwards_speed: f32,

  /// Global boost movement speed multiplier.
  pub boost_speed: f32,

  /// Global forward movement acceleration multiplier.
  pub forwards_accel: f32,

  /// Global sideways movement acceleration multiplier.
  pub sideways_accel: f32,

  /// Global backwards movement acceleration multiplier.
  pub backwards_accel: f32,

  /// Global boost movement acceleration multiplier.
  pub boost_accel: f32,

  /// Global multiplier on the speed the prince rotates around the katamari.
  pub rot_speed: f32,

  /// (??) Global camera delay along x axis.
  pub camera_delay_x: f32,

  /// (??) Global camera delay along y axis.
  pub camera_delay_y: f32,

  /// (??) Global camera delay along z axis.
  pub camera_delay_z: f32,

  /// The y-coordinate of the death plane. Anything below this value should be warped.
  pub death_plane_y: f32,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self { 
          mission: Mission::None, 

          catch_count_b: 0, 

          forwards_speed: 1.0, 
          sideways_speed: 1.0, 
          backwards_speed: 1.0, 
          boost_speed: 1.0,

          forwards_accel: 1.0, 
          sideways_accel: 1.0, 
          backwards_accel: 1.0, 
          boost_accel: 1.0,

          rot_speed: 1.0,

          camera_delay_x: 1.0,
          camera_delay_y: 1.0,
          camera_delay_z: 1.0,

          death_plane_y: -5000.0,
        }
    }
}

impl Display for GlobalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl GlobalState {
  pub fn set_speeds(&mut self, forw_s: f32, side_s: f32, back_s: f32, boost_s: f32, 
    forw_a: f32, side_a: f32, back_a: f32, boost_a: f32,
    rot_s: f32, dp_y: f32, cam_x: f32, cam_y: f32, cam_z: f32
  ) {
    self.forwards_speed = forw_s;
    self.sideways_speed = side_s;
    self.backwards_speed = back_s;
    self.boost_speed = boost_s;

    self.forwards_accel = forw_a;
    self.sideways_accel = side_a;
    self.backwards_accel = back_a;
    self.boost_accel = boost_a;

    self.rot_speed = rot_s;
    self.death_plane_y = dp_y;

    self.camera_delay_x = cam_x;
    self.camera_delay_y = cam_y;
    self.camera_delay_z = cam_z;
  }
}
