use gl_matrix::{
    common::{Mat4, Vec3},
    mat4, vec3,
};

use crate::{
    constants::{FRAC_PI_2, PI, TAU},
    macros::{lerp, max, set_y},
    math::{
        normalize_bounded_angle, vec3_inplace_add_vec, vec3_inplace_normalize,
        vec3_inplace_zero_small,
    },
    mission::{stage::Stage, state::MissionState},
    player::{
        camera::Camera,
        prince::{Prince, PushDir},
    },
};

use super::{
    collision::ray::KatCollisionRayType, flags::KatInclineMoveType, KatBoostEffectState, Katamari,
};

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
    pub(super) fn update_velocity(
        &mut self,
        prince: &mut Prince,
        camera: &Camera,
        mission_state: &MissionState,
    ) {
        let vel_accel_len = vec3::length(&self.velocity.vel_accel);

        mission_state
            .mission_config
            .get_kat_scaled_params(&mut self.scaled_params, self.diam_cm);

        if self.physics_flags.vs_mode_state == 2 {
            return;
        }

        if prince.oujistate.dash_start {
            // TODO: self.init_boost();
            self.boost_effect_state = Some(super::KatBoostEffectState::Build);
            self.boost_effect_timer = 0;
        }

        prince.oujistate.dash_effect = false;
        if prince.oujistate.dash || prince.oujistate.dash_start {
            // if dashing:
            if let Some(state) = self.boost_effect_state {
                match state {
                    KatBoostEffectState::Build => {
                        if !self.physics_flags.in_water || self.boost_effect_timer > 0 {
                            prince.oujistate.dash_effect = true;
                            self.boost_effect_timer += 1;
                            if self.boost_effect_timer > self.params.boost_build_duration {
                                self.boost_effect_state = Some(KatBoostEffectState::StopBuilding);
                            }
                            if self.physics_flags.in_water {
                                self.boost_effect_state = Some(KatBoostEffectState::Release);
                                self.boost_effect_timer =
                                    self.params.boost_release_duration_in_water;
                            }
                        }
                    }
                    KatBoostEffectState::StopBuilding => {
                        prince.oujistate.dash_effect = true;
                        if self.physics_flags.braking || !prince.oujistate.wheel_spin {
                            self.boost_effect_state = Some(KatBoostEffectState::Release);
                            self.boost_effect_timer = self.params.boost_release_duration;
                        }
                        if self.physics_flags.in_water {
                            self.boost_effect_state = Some(KatBoostEffectState::Release);
                            self.boost_effect_timer = self.params.boost_release_duration_in_water;
                        }
                    }
                    KatBoostEffectState::Release => {
                        prince.oujistate.dash_effect = false;
                        self.boost_effect_timer -= 1;
                        if self.boost_effect_timer == 0 {
                            self.boost_effect_state = Some(KatBoostEffectState::End);
                        }
                    }
                    KatBoostEffectState::End => {
                        prince.oujistate.dash_effect = false;
                    }
                }
            }

            // update `oujistate.sw_speed_disp` and its associated timer
            if !mission_state.is_vs_mode && prince.oujistate.dash {
                if !prince.oujistate.wheel_spin && self.sw_speed_disp_timer > 0 {
                    self.sw_speed_disp_timer -= 1;
                    prince.oujistate.sw_speed_disp = self.sw_speed_disp_timer > 0;
                }
            } else {
                prince.oujistate.sw_speed_disp = false;
            }
        }

        prince.oujistate.camera_mode = camera.get_mode().into();
        prince.oujistate.climb_wall = self.physics_flags.climbing_wall;
        prince.oujistate.hit_water = self.physics_flags.in_water;
        prince.oujistate.submerge = self.physics_flags.under_water;
        prince.oujistate.camera_state = camera.get_r1_jump_state().map_or(0, |s| s.into());
        prince.oujistate.vs_attack = self.physics_flags.vs_attack;

        if mission_state.is_tutorial() {
            // TODO_TUTORIAL: `kat_update_velocity:155-165`
        }

        let (mut push_accel, push_mag) = if !prince.oujistate.dash {
            // if not boosting:
            let accel = if let Some(push_dir) = prince.get_push_dir() {
                self.scaled_params.get_push_accel(push_dir)
            } else {
                vel_accel_len
            };

            (accel, prince.input_avg_push_len)
        } else {
            // if boosting:
            let accel = if prince.oujistate.wheel_spin {
                0.0
            } else {
                self.scaled_params.boost_accel
            };

            (accel, 1.0)
        };

        self.physics_flags.no_input_push = push_mag <= 0.0;

        if prince.get_flags() & 0x40000 != 0 {
            // if quick shifting:
            // TODO: (or pinching?)
            // rotate the `vel_accel` velocity by the angle the prince is turning
            let mut yaw_rot = [0.0; 16];
            mat4::from_y_rotation(&mut yaw_rot, prince.get_angle_speed());
            let vel_accel = self.velocity.vel_accel;
            vec3::transform_mat4(&mut self.velocity.vel_accel, &vel_accel, &yaw_rot);
        }

        // (??) compute speed multiplier
        let mut base_speed_mult = if !prince.oujistate.dash {
            let sideways_speed = self.scaled_params.get_push_max_speed(PushDir::Sideways)
                * self.params.get_speed_mult(PushDir::Sideways);

            let push_speed = if prince.get_push_dir() == Some(PushDir::Forwards) {
                self.scaled_params.get_push_max_speed(PushDir::Forwards)
                    * self.params.get_speed_mult(PushDir::Forwards)
            } else {
                self.scaled_params.get_push_max_speed(PushDir::Backwards)
                    * self.params.get_speed_mult(PushDir::Backwards)
            };

            let pre_speed = lerp!(prince.get_push_strength(), sideways_speed, push_speed);
            // TODO: `kat_update_velocity:240-256` (annoying lerp crap)
            pre_speed
        } else {
            self.scaled_params.max_boost_speed * self.params.boost_speed_mult
        };

        // apply a max speed penalty while huffing
        base_speed_mult *= prince.get_huff_speed_penalty();

        if self.physics_flags.airborne {
            // if the katamari is airborne, its speed shouldn't change
            push_accel = 0.0;
            base_speed_mult = 1.0;
        }

        let climb_mult = match self.physics_flags.climbing_wall {
            true => self.params.wallclimb_speed_penalty,
            false => 1.0,
        };

        let _speed = self.scaled_params.base_max_speed * base_speed_mult * climb_mult;

        // TODO_VS: `kat_update_velocity:304-324`

        // compute max speeds from scaled params
        let base_speed = self.scaled_params.base_max_speed;
        self.base_speed = base_speed;
        self.max_forwards_speed = base_speed * self.scaled_params.max_forwards_speed;
        self.max_boost_speed = base_speed * self.scaled_params.max_boost_speed;
        self.max_sideways_speed = base_speed * self.scaled_params.max_sideways_speed;
        self.max_backwards_speed = base_speed * self.scaled_params.max_backwards_speed;

        // TODO: `kat_compute_brake_state()`
        // TODO: `kat_update_velocity:342-494` (use result of computing brake state)
        // note that the above code will modify `accel` if the katamari is braking
        // TODO: hopefully this block also modifies `vel_accel`??
        let _is_vs_mode_shoot = false; // TODO_VS: this value is updated above

        if !self.physics_flags.braking {
            self.input_push_dir = prince.get_push_dir();
        }
        prince.oujistate.brake = self.physics_flags.braking;

        // compute acceleration penalty when moving uphill
        let incline_accel_mult = match self.physics_flags.incline_move_type {
            KatInclineMoveType::MoveUphill => prince.get_uphill_accel_penalty(),
            _ => 1.0,
        };

        // compute spine-derived acceleration multiplier (used to smooth the acceleration
        // out i guess)
        let spline_mult = match self.physics_flags.braking {
            true => 1.0,
            false => self.compute_spline_accel_mult(prince),
        };

        let _accel = push_accel * push_mag * incline_accel_mult * spline_mult;

        // TODO: `kat_update_velocity:508-586` (apply acceleration from ground contact)

        self.velocity.last_vel_accel = self.velocity.vel_accel;

        // TODO: `kat_update_velocity:595-601`

        // the katamari's speed cap is greatly increased when moving downhill on floors
        // with the `SpeedCheckOff` hit attribute. (e.g. the mas5/mas8 hill)
        let _uncap_speed = self.hit_flags.speed_check_off
            && self.physics_flags.incline_move_type == KatInclineMoveType::MoveDownhill;

        // TODO: `kat_update_velocity:608-639`

        // if the katamari just bonked, set velocity equal to the initial bonk velocity
        if self.physics_flags.bonked {
            self.velocity.velocity = self.init_bonk_velocity;
            self.physics_flags.bonked = false;
        }

        vec3::normalize(&mut self.velocity.velocity_unit, &self.velocity.velocity);
    }

    /// Computes a multiplier on the katamari's acceleration derived from a spline.
    /// offset: 0x232d0
    fn compute_spline_accel_mult(&self, _prince: &Prince) -> f32 {
        // TODO
        1.0
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
