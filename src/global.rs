use std::{fmt::Display};

use gl_matrix::{common::{Vec3, Vec4}};

use crate::{mission::{Mission, GameMode}};

/// Miscellaneous global game state.
#[derive(Debug, Default)]
pub struct GlobalState {
  /// The current mission.
  pub mission: Option<Mission>,

  /// The current game mode.
  pub gamemode: Option<GameMode>,

  /// The current game time (in *real time*, not ticks).
  /// offset: 0xff12c
  pub game_time_ms: i32,

  /// The number of ticks before the mission timer expires.
  /// offset: 0xff120
  pub remain_time_ticks: i32,

  /// If true, ticking the physics engine has no effect (it's "frozen").
  /// offset: 0x10daea
  pub freeze: bool,

  /// The "theme object" score in constellation levels (e.g. number of crabs in Make Cancer).
  pub catch_count_b: i32,

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

  /// (??) Global camera delay along x, y, and z axes.
  pub camera_delay: Vec3,

  /// The y-coordinate of the death plane. Anything below this value should be warped.
  pub death_plane_y: f32,

  /// Gravity direction.
  pub gravity: Vec4,

  /// Negative gravity direction.
  pub neg_gravity: Vec4,

  /// (??)
  /// offset: 0xd35580
  pub map_loop_rate: f32,
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

    self.camera_delay[0] = cam_x;
    self.camera_delay[1] = cam_y;
    self.camera_delay[2] = cam_z;
  }

  pub fn set_gravity(&mut self, x: f32, y: f32, z: f32) {
    self.gravity[0] = x;
    self.gravity[1] = y;
    self.gravity[2] = z;

    self.neg_gravity[0] = -x;
    self.neg_gravity[1] = -y;
    self.neg_gravity[2] = -z;
  }

  pub fn set_gamemode(&mut self, gamemode: i32) {
    self.gamemode = Some(gamemode.try_into().unwrap());
  }
}
