use gl_matrix::{common::{Mat4, Vec4}, vec4};

use crate::constants::RESCALE;


#[derive(Debug, Default)]
pub struct Katamari {
  /// The player who owns this katamari.
  /// offset: 0x44
  player: u8,

  /// The index of the katamari mesh being used.
  /// offset: 0x47
  mesh_index: u8,

  /// The volume of the katamari (in m^3).
  /// offset: 0x50
  vol_m3: f32,

  /// The maximum prop volume that can be collected (in m^3).
  /// offset: 0x54
  max_pickup_vol_m3: f32,

  /// The exact diameter of the katamari (in cm).
  /// offset: 0x5c
  diam_cm: f32,

  /// The truncated diameter of the katamari (in mm).
  /// offset: 0x60
  diam_trunc_mm: i32,

  /// The initial exact diameter of the katamari (in cm).
  /// offset: 0x64
  init_diam_cm: f32,

  /// The radius of the katamari (in cm).
  /// offset: 0x68
  rad_cm: f32,

  /// The visual radius of the katamari "ball" (in cm).
  /// offset: 0x70
  display_rad_cm: f32,

  /// The circumference of the katamari (in cm).
  /// offset: 0x74
  circumf_cm: f32,

  /// The speed of the katamari on the current tick.
  /// offset: 0x78
  speed: f32,

  /// The speed of the katamari on the previous tick.
  /// offset: 0x7c
  last_speed: f32,

  /// The center point of the katamari on the current tick.
  /// offset: 0x460
  center: Vec4,

  /// The center point of the katamari on the previous tick.
  /// offset: 0x470
  last_center: Vec4,

  /// The katamari's transform matrix.
  /// offset: 0x710
  transform: Mat4,

  /// (??) The point on a surface directly below the katamari where the shadow should be drawn.
  /// offset: 0x86c
  shadow_pos: Vec4,
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

  pub fn get_translation(&self, x: &mut f32, y: &mut f32, z: &mut f32, sx: &mut f32, sy: &mut f32, sz: &mut f32) -> () {
    // sort of hacky to read the translation directly out of the matrix but whatever.
    // the builtin `mat4::get_translation` writes the values to a `Vec3` instead of individual floats.
    // (see: https://docs.rs/gl_matrix/latest/src/gl_matrix/mat4.rs.html#1030-1036)
    *x = self.transform[12];
    *y = self.transform[13];
    *z = self.transform[14];

    *sx = self.shadow_pos[0];
    *sy = self.shadow_pos[1];
    *sz = self.shadow_pos[2];
  }

  pub fn set_translation(&mut self, x: f32, y: f32, z: f32) {
    let trans = [x * RESCALE, y * RESCALE, z * RESCALE, 1.0];

    // set the center and last center points
    vec4::copy(&mut self.center, &trans);
    vec4::copy(&mut self.last_center, &trans);

    // set the translation component of the transform matrix
    self.transform[12] = trans[0];
    self.transform[13] = trans[1];
    self.transform[14] = trans[2];
  }

  pub fn get_matrix(&self, xx: &mut f32, xy: &mut f32, xz: &mut f32, yx: &mut f32, yy: &mut f32, yz: &mut f32, zx: &mut f32, zy: &mut f32, zz: &mut f32) -> () {
    *xx = self.transform[0];
    *xy = self.transform[1];
    *xz = self.transform[2];
    *yx = self.transform[4];
    *yy = self.transform[5];
    *yz = self.transform[6];
    *zx = self.transform[8];
    *zy = self.transform[9];
    *zz = self.transform[10];
  }
}
