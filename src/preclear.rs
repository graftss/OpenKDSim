#[derive(Debug)]
enum PreclearStage {
  MAS4 = 0,
  MAS8 = 1,
  MTM = 2,
}

#[derive(Debug, Default)]
pub struct PreclearState {
  enabled: bool,
  is_pullback: bool,
  stage: Option<PreclearStage>,
  force_disable: bool,
  fog_alpha: f32,
  cam_pullback_dist: f32,
  cam_pullback_speed: f32,
  post_end_timer: i32,
  cam_pullback_post_end: f32,
}

impl PreclearState {
  pub fn get_alpha(&self) -> f32 {
    self.fog_alpha
  }
}
