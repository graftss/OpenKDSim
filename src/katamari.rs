pub struct Katamari {
  /// The player who owns this katamari.
  player: u8,

  mesh_index: u8,

  /// The volume of the katamari (in m^3).
  vol_m3: f32,

  /// The maximum prop volume that can be collected (in m^3).
  max_pickup_vol_m3: f32,

  /// The exact diameter of the katamari (in cm).
  diam_cm: f32,

  /// The truncated diameter of the katamari (in mm).
  diam_trunc_mm: i32,

  /// The initial exact diameter of the katamari (in cm).
  init_diam_cm: f32,

  /// The radius of the katamari (in cm).
  rad_cm: f32,

  /// The visual radius of the katamari "ball" (in cm).
  display_rad_cm: f32,

  /// The circumference of the katamari (in cm).
  circumf_cm: f32,
}

impl Default for Katamari {
    fn default() -> Self {
        Self { player: Default::default(), mesh_index: Default::default(), vol_m3: Default::default(), max_pickup_vol_m3: Default::default(), diam_cm: Default::default(), diam_trunc_mm: Default::default(), init_diam_cm: Default::default(), rad_cm: Default::default(), display_rad_cm: Default::default(), circumf_cm: Default::default() }
    }
}

impl Katamari {
  pub fn get_radius(&self) -> f32 {
    self.rad_cm
  }

  pub fn get_display_radius(&self) -> f32 {
    self.display_rad_cm
  }

  pub fn get_diam_int(&self) -> i32 {
    self.diam_trunc_mm
  }

  pub fn get_vol(&self) -> f32 {
    self.vol_m3
  }
}
