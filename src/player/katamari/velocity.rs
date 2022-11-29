use gl_matrix::{common::Vec3, vec3};

use crate::{
    math::{vec3_inplace_add_vec, vec3_inplace_zero_small},
    mission::{stage::Stage, state::MissionState},
    player::prince::Prince,
};

use super::{collision::ray::KatCollisionRayType, flags::KatInclineMoveType, Katamari};

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
    pub accel_grav: Vec3,

    /// Acceleration from the contacted floor incline
    /// offset: 0xb0
    pub accel_incline: Vec3,

    /// (??) Acceleration from the contacted floor friction (or some kind of similar force)
    /// offset: 0xc0
    pub accel_ground_friction: Vec3,
}

impl Katamari {
    pub(super) fn update_velocity(&mut self, _prince: &Prince, mission: &MissionState) {
        mission
            .mission_config
            .get_kat_scaled_params(&mut self.scaled_params, self.diam_cm);
    }

    pub(super) fn apply_acceleration(&mut self, mission_state: &MissionState) {
        vec3::zero(&mut self.bonus_vel);

        // start with `velocity + accel_incline`
        let mut next_vel = self.velocity.velocity.clone();
        vec3_inplace_add_vec(&mut next_vel, &self.velocity.accel_incline);

        let speed0 = vec3::length(&next_vel);
        if speed0 > 0.0 && !self.physics_flags.climbing_wall {
            // if moving and not climbing a wall, apply ground friction
            vec3_inplace_add_vec(&mut next_vel, &self.velocity.accel_ground_friction);
        }

        if self.hit_flags.speed_check_off
            && self.physics_flags.incline_move_type == KatInclineMoveType::MoveDownhill
        {
            // TODO: `kat_apply_acceleration:44-61` (speedcheckoff acceleration)
        }

        vec3::normalize(&mut self.velocity.vel_accel_unit, &self.velocity.vel_accel);
        vec3::add(
            &mut self.velocity.vel_accel_grav,
            &self.velocity.vel_accel,
            &self.velocity.accel_grav,
        );
        vec3::normalize(
            &mut self.velocity.vel_accel_grav_unit,
            &self.velocity.vel_accel_grav,
        );

        let mut next_vel = &self.velocity.vel_accel;

        if self.physics_flags.grounded_ray_type == Some(KatCollisionRayType::Bottom) {
            // if grounded via the "bottom" ray, meaning the katamari isn't vaulting:
            // TODO_VS: `kat_apply_acceleration:79-90`
            // TODO_ENDING: `kat_apply_acceleration:91-96`
            // TODO: weird conditional here depending on vs mode, but it's always true in single player
            if !self.physics_flags.climbing_wall {
                // if not wall climbing:
                // TODO: some SHUFPS crap going on here, not clear what it's doing
                vec3_inplace_add_vec(&mut self.center, &self.velocity.vel_accel);
            } else {
                // if wall climbing:
                if !self.physics_flags.at_max_climb_height {
                    // if still gaining height from the wall climb:
                    // TODO: SHUFPS
                }
                // TODO: `kat_update_wall_climb()`
            }

            if self.physics_flags.airborne {
                next_vel = &self.velocity.vel_accel_grav;
            }

            self.speed = vec3::length(&next_vel);

            self.cache_sizes();
            // TODO: `kat_update_rotation_speed(&next_vel)`
            self.update_transform_unvaulted();
        } else {
            self.cache_sizes();
            // TODO: `kat_update_transform_vaulted()`
        }

        vec3_inplace_zero_small(&mut self.center, 0.001);
        self.base_speed_ratio = self.speed / self.base_speed;
        self.vault_prop_decay_mult =
            1.0 - self.base_speed_ratio * self.params.vault_prop_pull_to_center_mult;

        // TODO: `kat_cache_shell_points()`
        // TODO_VS: `kat_apply_acceleration:166-196`

        if mission_state.stage == Stage::World {
            self.physics_flags.can_emit_smoke = self.diam_cm > 1200.0;
        }
    }

    fn update_rotation_speed(&mut self, vel: &Vec3) {}
}
