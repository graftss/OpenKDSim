use gl_matrix::{
    common::{Mat4, Vec3},
    mat4, vec3,
};

use crate::{
    constants::{FRAC_PI_2, PI, TAU},
    macros::{max, set_y},
    math::{
        normalize_bounded_angle, vec3_inplace_add_vec, vec3_inplace_normalize,
        vec3_inplace_zero_small,
    },
    mission::{stage::Stage, state::MissionState},
    player::prince::Prince,
};

use super::{collision::ray::KatCollisionRayType, flags::KatInclineMoveType, Katamari};

/// 0.9998
const ALMOST_1: f32 = f32::from_bits(0x3f7ff2e5);

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

        let mut next_vel = self.velocity.vel_accel;

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
                next_vel = self.velocity.vel_accel_grav;
            }

            self.speed = vec3::length(&next_vel);

            self.cache_sizes();
            self.update_rotation_speed(&next_vel);
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

    fn update_rotation_speed(&mut self, vel: &Vec3) {
        if self.physics_flags.braking {
            return self.spin_rotation_speed = 0.0;
        }

        let vel_len = vec3::len(vel);
        let pivot_circumf = max!(self.fc_ray_len, 0.1) * TAU;

        if !self.physics_flags.airborne {
            let mut net_normal_unit = [0.0, 0.0, 0.0];

            if !self.physics_flags.climbing_wall {
                // if not airborne and not climbing a wall:
                vec3::add(
                    &mut net_normal_unit,
                    &self.contact_floor_normal_unit,
                    &self.contact_wall_normal_unit,
                );
                vec3_inplace_normalize(&mut net_normal_unit);
                // TODO: `kat_update_rotation_speed:76-87` (this seems like it's just a no-op)
            } else {
                // if not airborne and climbing a wall, set net normal to `<0,1,0>`
                set_y!(net_normal_unit, 1.0);
            }

            let mut net_normal_rot = Mat4::default();
            mat4::from_rotation(&mut net_normal_rot, FRAC_PI_2, &net_normal_unit);

            let mut vel_unit = Vec3::default();
            if !self.physics_flags.immobile {
                // if the katamari is not airborne and moving:
                vec3::normalize(&mut vel_unit, &vel);
            } else {
                set_y!(vel_unit, -1.0);
            }

            if vec3::dot(&vel_unit, &net_normal_unit) >= ALMOST_1 {
                return self.spin_rotation_speed = 0.0;
            }

            // compute spin rotation axis
            vec3::transform_mat4(&mut self.spin_rotation_axis, &vel_unit, &net_normal_rot);

            // set y component to zero and renormalize
            set_y!(self.spin_rotation_axis, 0.0);
            let spin_rot_len = vec3::len(&self.spin_rotation_axis);

            if spin_rot_len < 0.5 {
                vec3::copy(&mut self.spin_rotation_axis, &self.camera_side_vector);
            }

            if self.speed <= 0.0 {
                return self.spin_rotation_speed = 0.0;
            }

            self.spin_rotation_speed = normalize_bounded_angle(vel_len / pivot_circumf);
        } else {
            // if katamari is airborne:
            // TODO: `kat_update_rotation_speed:171-221`
        }

        self.spin_rotation_speed = self.spin_rotation_speed.clamp(-PI, PI);
    }
}
