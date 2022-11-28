use gl_matrix::common::Vec3;

use crate::player::prince::Prince;

use super::Katamari;

/// Katamari velocity and acceleration values.
#[derive(Debug, Default, Copy, Clone)]
pub struct KatVelocity {
    /// Current velocity
    /// offset: 0x0
    pub velocity: Vec3,

    /// Current unit velocity
    /// offset: 0x10
    pub velocity_unit: Vec3,

    /// (??)
    /// offset: 0x20
    pub speed_orth_on_floor: Vec3,

    /// (??)
    /// offset: 0x30
    pub speed_proj_on_floor: Vec3,

    /// (??)
    /// offset: 0x40
    pub last_vel_accel: Vec3,

    /// (??) Velocity + player acceleration
    /// offset: 0x50
    pub vel_accel: Vec3,

    /// Unit vector of `vel_accel`.
    /// offset: 0x60
    pub vel_accel_unit: Vec3,

    /// (??) Velocity + player accel + gravity
    /// offset: 0x70
    pub vel_accel_grav: Vec3,

    /// Unit vector of `vel_accel_grav`.
    /// offset: 0x80
    pub vel_accel_grav_unit: Vec3,

    /// (??)
    /// offset: 0x90
    pub push_vel_on_floor_unit: Vec3,

    /// Acceleration from gravity
    /// offset: 0xa0
    pub accel_gravity: Vec3,

    /// Acceleration from the contacted floor incline
    /// offset: 0xb0
    pub accel_incline: Vec3,

    /// (??) Acceleration from the contacted floor friction (or some kind of similar force)
    /// offset: 0xc0
    pub accel_ground_friction: Vec3,
}

impl Katamari {
    pub(super) fn update_velocity(&mut self, _prince: &Prince) {}
}
