pub struct Katamari {
  /// The player who owns this katamari.
  pub player: u8,

  pub mesh_index: u8,

  /// The volume of the katamari (in m^3).
  pub vol_m3: f32,

  /// The maximum prop volume that can be collected (in m^3).
  pub max_pickup_vol_m3: f32,

  /// The exact diameter of the katamari (in cm).
  pub diam_cm: f32,

  /// The truncated diameter of the katamari (in mm).
  pub diam_trunc_mm: i32,

  /// The initial exact diameter of the katamari (in cm).
  pub init_diam_cm: f32,

  /// The radius of the katamari (in cm).
  pub rad_cm: f32,

  /// The visual radius of the katamari "ball" (in cm).
  pub ball_rad_cm: f32,

  /// The circumference of the katamari (in cm).
  pub circumf_cm: f32
}

impl Default for Katamari {
    fn default() -> Self {
        Self { player: Default::default(), mesh_index: Default::default(), vol_m3: Default::default(), max_pickup_vol_m3: Default::default(), diam_cm: Default::default(), diam_trunc_mm: Default::default(), init_diam_cm: Default::default(), rad_cm: Default::default(), ball_rad_cm: Default::default(), circumf_cm: Default::default() }
    }
}
