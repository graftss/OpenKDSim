use gl_matrix::{common::{Mat4, Vec4}, vec4};

use crate::constants::{TRANSFORM_X_POS, TRANSFORM_Y_POS, TRANSFORM_Z_POS, UNITY_TO_SIM_SCALE};

/// (??) not sure about this
#[derive(Debug, Clone, Copy)]
pub enum AlarmType {
  Closest,
  Closer,
  Close,
}

/// The extra "shell" collision rays which extend along the top half of the katamari.
/// (see https://discord.com/channels/232268612285497345/805240416894713866/842591732229996544)
#[derive(Debug, Clone, Copy)]
pub enum ShellRay {
  TopCenter = 1,
  Left = 2,
  Right = 3,
  Bottom = 4,
  TopLeft = 5,
  TopRight = 6,
}

/// The different types of rays making up the katamari's collision.
/// `Bottom`: the single ray extending directly downwards from the katamari's center.
///           this ray is used to make sure the katamari moves smoothly along the ground
///           when nothing has been picked up to make the katamari's shape oblong.
/// `Mesh`: one of the normal rays comprising the katamari's boundary mesh.
///         picking up an object will extend the mesh ray nearest to where the object was attached.
/// `Prop`: if a prop with a vault point is collected, the katamari will gain a collision ray
///         corresponding to that prop in the direction of one of its vault points.
#[derive(Debug, Clone, Copy)]
pub enum KatRay {
  Bottom = 0,
  Mesh = 1,
  Prop = 2,
}

#[derive(Debug, Default)]
pub struct PhysicsFlags {
    /// If true, the katamari has no surface contacts.
    pub airborne: bool,

    /// If true, the katamari is climbing a wall.
    pub climbing_wall: bool,

    /// If true, the katamari is at its maximum climb height (so it can't climb higher).
    pub at_max_climb_height: bool,

    /// If true, the katamari is braking.
    pub braking: bool,

    /// If true, the katamari is bonking something (only true the frame it bonks).
    pub bonking: bool,

    /// If true, the katamari is contacting a wall.
    pub contacts_wall: bool,
    
    /// If true, the katamari is contacting a wall.
    pub contacts_floor: bool,
    
    /// If true, the katamari is in water.
    pub in_water: bool,

    /// (??) copy of `in_water`
    pub in_water_copy: bool,

    /// (??) If true, the katamari was hit by a moving prop.
    pub hit_by_moving_prop: bool,

    /// (??) If true, the katamari is contacting a prop.
    pub _contacts_prop: bool,

    /// (??) If true, the katamari is contacting the bottom of a downward-slanting surface.
    /// (e.g. can be triggered under mas1 table by setting simulation+0x71614 to 3, which
    /// changes the definition of how downward-slanting such a surface needs to be)
    pub contacts_slanted_ceiling: bool,

    /// (??) If true, the katamari moved more than its own radius during the last tick.
    pub moved_more_than_radius: bool,

    /// If true, the katamari is contacting a prop.
    pub contacts_prop: bool,

    /// (??) A shell ray which collided with something
    pub hit_shell_ray: Option<ShellRay>,

    /// If true, the katamari is completely submerged underwater.
    pub under_water: bool,

    /// If true, the katamari is not moving.
    pub stationary: bool,

    /// (??) The type of boundary ray currently acting as the pivot.
    pub pivot_ray: Option<KatRay>,

    /// If true, the katamari is contacting a non-flat floor (normal < 0.9999).
    pub on_sloped_floor: bool,

    /// If true, the katamari is contacting a flat floor (normal >= 0.9999).
    pub on_flat_floor: bool,

    /// (??)
    pub moved_too_much_0x14: bool,

    // TODO: fill these out
}

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

  /// Various physics-related flags (some of which aren't actually true/false values, but whatever).
  /// offset: 0xa4
  physics_flags: PhysicsFlags,

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

  /// Whether or not the "something's coming" alarm is going off.
  /// offset: 0x3b84
  alarm_mode: bool,

  /// The "urgency" of the "something's coming" alarm, if the alarm is active.
  /// offset: 0x3b86
  alarm_type: Option<AlarmType>,
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
    let K = UNITY_TO_SIM_SCALE;
    let trans = [x * K, y * K, z * K, 1.0];

    // set the center and last center points
    vec4::copy(&mut self.center, &trans);
    vec4::copy(&mut self.last_center, &trans);

    // set the translation component of the transform matrix
    self.transform[TRANSFORM_X_POS] = trans[0];
    self.transform[TRANSFORM_Y_POS] = trans[1];
    self.transform[TRANSFORM_Z_POS] = trans[2];
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

  pub fn get_alarm(&self, alarm_mode: &mut i32, alarm_type: &mut i32) {
    *alarm_mode = self.alarm_mode as i32;
    *alarm_type = self.alarm_type.unwrap_or(AlarmType::Closest) as i32;
  }

  pub fn is_in_water(&self) -> bool {
    self.physics_flags.in_water
  }
}
