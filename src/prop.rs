use gl_matrix::{common::{Vec4, Mat4, Vec3}, mat4};

use crate::util::scale_sim_transform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropGlobalState {
  /// Normal unattached state.
  Unattached = 1,

  /// Normal attached state.
  Attached = 2,

  /// Briefly, just after being knocked airborne, while intangible to the katamari.
  AirborneIntangible = 4,
}

impl Default for PropGlobalState {
    fn default() -> Self {
        Self::Unattached
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropLinkAction {

}

#[derive(Debug, Default)]
struct PropSubobject {
    /// The next subobject in the linked list, if one exists.
    /// offset: 0x10
    pub next: Option<Box<PropSubobject>>,

    /// (??) The position of this subobject (presumably relative to the prop).
    /// offset: 0x18
    pub pos: Vec4,

    /// (??) The Euler agnles of this subobject 
    /// offset: 0x28
    pub rot_vec: Vec4,
}

#[derive(Debug, Default)]
pub struct Prop {
  /// The unique id of this prop.
  /// offset: 0x0
  ctrl_idx: u16,

  /// The object type of this prop.
  /// offset: 0x2
  name_idx: u16,

  /// (??) some flags
  /// offset: 0x6
  flags: u8,

  /// offset: 0x7
  global_state: PropGlobalState,

  /// (??) some more flags
  /// offset: 0x8
  flags2: u8,

  /// If true, the prop won't update.
  /// offset: 0x9
  force_disabled: bool,

  /// (??) If true, the prop won't be displayed.
  /// offset: 0xa
  display_off: bool,

  /// The alpha level of the prop (fades out as it gets further from the player camera).
  /// offset: 0x14
  alpha: f32,

  mono_move_type_no: u16,

  /// (??) The prop is intangible until the player loads this area.
  /// offset: 0x1a
  hit_on_area: u8,

  /// (??) encodes the behavior of this prop relative to its child/parent
  /// offset: 0x1b
  link_action: Option<PropLinkAction>,

  /// Encodes motion behavior innate to this prop's name index.
  /// offset: 0x1c
  innate_motion_type: u8,

  /// The state index of the innate motion action.
  /// offset: 0x1d
  innate_motion_state: u8,

  /// True if the prop's motion action follows a path.
  /// offset: 0x20
  has_path_motion_action: bool,

  /// If true, the prop cannot wobble.
  /// offset: 0x21
  disable_wobble: bool,

  /// True if the prop's motion action applies any translation.
  /// offset: 0x22
  has_moving_motion: bool,

  /// The next sibling of this prop in its family tree.
  /// offset: 0x28
  next_sibling: Option<Box<Prop>>,

  /// The first child of this prop in its family tree.
  /// offset: 0x30
  first_child: Option<Box<Prop>>,

  /// The area in which this prop loaded.
  /// offset: 0x38
  init_area: u8,

  /// The position at which this prop loaded.
  /// offset: 0x40
  init_pos: Vec4,

  /// The prop's rotation as a matrix.
  /// offset: 0x50
  rotation_mat: Mat4,

  /// The prop's position.
  /// offset: 0x90
  pos: Vec4,

  /// The prop's rotation as Euler angles.
  /// offset: 0xa0
  rotation_vec: Vec4,

  /// The prop's scale. (unused in simulation)
  /// offset: 0xb0
  scale: Vec4,

  /// The prop's position on the previous tick.
  /// offset: 0xc0
  last_pos: Vec4,

  /// The prop's rotation as Euler angles on the previous tick.
  /// offset: 0xd0
  last_rotation_vec: Vec4,

  /// The prop's transform matrix while unattached from the katamari.
  /// offset: 0x110
  unattached_transform: Mat4,

  /// The prop's initial rotation matrix when it loaded.
  /// offset: 0x150
  init_rotation_mat: Mat4,

  /// A pointer to the prop's first subobject.
  /// offset: 0x558
  first_subobject: Option<Box<PropSubobject>>,

  /// The radius of the prop's bounding sphere.
  /// offset: 0x910
  radius: f32,

  /// The volume of the prop's AABB (in m^3).
  /// offset: 0x918
  aabb_vol_m3: f32,

  /// (??) The volume of the prop used when comparing to the katamari's volume (in m^3).
  /// offset: 0x91c
  compare_vol_m3: f32,

  /// The *base* volume added to the katamari when this prop is attached (in m^3).
  /// This value will still be scaled by the mission's penalty.
  /// offset: 0x920
  added_vol_m3: f32,

  /// The exact katamari diameter needed to collect this object (in cm), obtained by
  /// comparing the prop's AABB volume to the katamari's volume.
  /// The prop's true collection diameter is obtained by truncating this value to an integral value in mm.
  /// offset: 0x930
  exact_collect_diam_cm: f32,

  /// The minimum katamari diameter needed to collect this object (in mm).
  /// offset: 0x934
  collect_diam_mm: i32,

  /// The sizes of the prop's AABB
  /// offset: 0x938
  aabb_size: Vec3,

  /// The true volume added to the katamari, computed *at the time the prop is attached*.
  /// This value is saved so that if the prop is lost, the volume can be recovered and subtracted
  /// from the katamari's volume.
  /// offset: 0x944
  onattach_added_vol: f32,

  /// The number of ticks remaining on the mission timer when the prop was attached.
  /// offset: 0x950
  onattach_remain_ticks: i32,

  /// The real-time game time when the prop was attached (in ms).
  /// offset: 0x954
  onattach_game_time_ms: i32,

  /// The prop's triangle mesh used to collide the prop with the katamari while the
  /// katamari is too small to attach the prop.
  /// The mesh is stored in the "mono data" glob.
  /// offset: 0x960
  collision_mesh: (),

  /// (??) The additional transform applied to the prop while it is attached to the katamari.
  /// offset: 0x968
  attached_transform: Mat4,

  /// While attached, the offset from the katamari center to the prop's center.
  /// offset: 0x9e8
  kat_center_offset: Vec4,

  /// (??) The velocity of the katamari when it collided with the prop.
  /// offset: 0x9f8
  kat_collision_vel: Vec4,

  /// The prop's distance to player 0 on the previous frame.
  /// offset: 0xa08
  last_dist_to_p0: f32,

  /// The prop's distance to player 1 on the previous frame.
  /// offset: 0xa0c
  last_dist_to_p1: f32,

  /// The prop's distance to player 0.
  /// offset: 0xa10
  dist_to_p0: f32,
  
  /// The prop's distance to player 1.
  /// offset: 0xa14
  dist_to_p1: f32,

  /// The remaining "bonk force" needed to detach this prop from the katamari.
  /// Initialized to the prop's unscaled volume when the prop is attached.
  /// offset: 0xa18
  remain_knockoff_volume: f32,

  /// If this prop is attached, points to the prop that was attached before this (if one exists).
  /// offset: 0xa28
  collected_before: Box<Prop>,

  /// If this prop is attached, points to the prop that was attached after this (if one exists).
  /// offset: 0xa30
  collected_next: Box<Prop>,

  /// True if this prop has a twin prop on the Gemini mission.
  /// offset: 0xb10
  has_twin: bool,

  /// If this prop has a Gemini twin, points to the twin prop.
  /// offset: 0xb18
  twin: Box<Prop>,

  /// The transform matrix of this prop when it was loaded.
  /// offset: 0xb24
  init_transform: Mat4,

  /// The Euler angles of this prop when it was loaded.
  /// offset: 0xbb8
  init_rotation_vec: Vec4,
}

impl Prop {
    pub fn is_initialized(&self) -> bool {
        self.name_idx != u16::MAX
    }

    /// Traverses the prop's linked list of subobjects to compute the list's length.
    pub fn count_subobjects(&self) -> i32 {
        let mut result = 0;
        let mut try_next_subobj = &self.first_subobject;

        loop {
            if let Some(next_subobj) = try_next_subobj {
                result += 1;
                try_next_subobj = &next_subobj.next;
            } else {
                break;
            }
        }

        return result;
    }

    /// Traverse the prop's linked list of subobject to find the subobject at index `subobj_idx`.
    fn get_subobject(&self, subobj_idx: i32) -> &Option<Box<PropSubobject>> {
        let mut next_idx = 0;
        let mut try_next_subobj = &self.first_subobject;

        loop {
            if let Some(next_subobj) = try_next_subobj {
                if next_idx == subobj_idx {
                    return try_next_subobj
                } else {
                    next_idx += 1;
                    try_next_subobj = &next_subobj.next;
                }
            } else {
                return &None;
            }
        }
    }

    /// Mimicks the `GetSubobjectPosition` API function.
    pub fn get_subobject_position(&self,
        subobj_idx: i32,
        pos_x: &mut f32, pos_y: &mut f32, pos_z: &mut f32,
        rot_x: &mut f32, rot_y: &mut f32, rot_z: &mut f32,
    ) {
        if let Some(subobj) = self.get_subobject(subobj_idx) {
            *pos_x = subobj.pos[0];
            *pos_y = subobj.pos[1];
            *pos_z = subobj.pos[2];
            *rot_x = subobj.rot_vec[0];
            *rot_y = subobj.rot_vec[1];
            *rot_z = subobj.rot_vec[2];
        }
    }

    pub fn is_attached(&self) -> bool {
        self.global_state == PropGlobalState::Attached
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    /// Writes the active transform to `out`.
    /// This can either be the unattached transform or the attached transform. 
    pub unsafe fn unsafe_copy_transform(&self, out: *mut Mat4) {
        let mut transform = if self.is_attached() {
            self.attached_transform.clone()
        } else {
            self.unattached_transform.clone()
        };

        scale_sim_transform(&mut transform);

        mat4::copy(&mut *out, &transform); 
    }
}
