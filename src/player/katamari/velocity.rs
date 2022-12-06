use gl_matrix::{
    common::{Mat4, Vec3},
    mat4, vec3,
};

use crate::{
    constants::{FRAC_PI_2, PI, TAU, VEC3_Y_NEG, VEC3_Z_POS},
    macros::{
        inv_lerp, inv_lerp_clamp, lerp, max, panic_log, set_y, temp_debug_log, vec3_from,
        vec3_unit_xz,
    },
    math::{
        acos_f32, normalize_bounded_angle, vec3_inplace_add_scaled, vec3_inplace_add_vec,
        vec3_inplace_normalize, vec3_inplace_scale, vec3_inplace_subtract_vec,
        vec3_inplace_zero_small, vec3_projection,
    },
    mission::{self, stage::Stage, state::MissionState, GameMode},
    player::{
        camera::{mode::CameraMode, Camera},
        katamari::spline::compute_spline_accel_mult,
        prince::{Prince, PushDir},
    },
};

use super::{
    collision::ray::KatCollisionRayType, flags::KatInclineMoveType, KatBoostEffectState, Katamari,
};

/// 0.9998
const ALMOST_1: f32 = f32::from_bits(0x3f7ff2e5);

enum BrakeState {
    /// Not pushing hard enough to elicit katamari movement
    NoPush,

    /// Pushing against velocity
    PushBrake,

    /// Pushing towards velocity
    PushNoBrake,

    /// TODO_VS: i have no clue
    Shoot,
}

/// Katamari velocity and acceleration values.
#[derive(Default, Copy, Clone)]
pub struct KatVelocity {
    /// Current velocity
    /// offset: 0x0
    pub velocity: Vec3,

    /// Current unit velocity
    /// offset: 0x10
    pub velocity_unit: Vec3,

    /// (??)
    /// offset: 0x20
    pub vel_rej_floor: Vec3,

    /// (??)
    /// offset: 0x30
    pub vel_proj_floor: Vec3,

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

    /// (Downwards) velocity from gravity
    /// offset: 0xa0
    pub vel_grav: Vec3,

    /// Acceleration from the contacted floor incline
    /// offset: 0xb0
    pub accel_incline: Vec3,

    /// (??) Acceleration from the contacted floor friction (or some kind of similar force)
    /// offset: 0xc0
    pub accel_ground_friction: Vec3,
}

impl core::fmt::Debug for KatVelocity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KatVelocity")
            .field("velocity", &self.velocity)
            .field("vel_rej_floor", &self.vel_rej_floor)
            .field("vel_proj_floor", &self.vel_proj_floor)
            .field("last_vel_accel", &self.last_vel_accel)
            .field("vel_accel", &self.vel_accel)
            .field("vel_accel_grav", &self.vel_accel_grav)
            .field("push_vel_on_floor_unit", &self.push_vel_on_floor_unit)
            .field("vel_grav", &self.vel_grav)
            .field("accel_incline", &self.accel_incline)
            .field("accel_ground_friction", &self.accel_ground_friction)
            .finish()
    }
}

impl KatVelocity {
    /// Reset all velocities and accelerations to 0
    pub fn reset(&mut self) {
        vec3::zero(&mut self.velocity);
        vec3::zero(&mut self.velocity_unit);
        vec3::zero(&mut self.vel_rej_floor);
        vec3::zero(&mut self.vel_proj_floor);
        vec3::zero(&mut self.last_vel_accel);
        vec3::zero(&mut self.vel_accel);
        vec3::zero(&mut self.vel_accel_unit);
        vec3::zero(&mut self.vel_accel_grav);
        vec3::zero(&mut self.vel_accel_grav_unit);
        vec3::zero(&mut self.push_vel_on_floor_unit);
        vec3::zero(&mut self.vel_grav);
        vec3::zero(&mut self.accel_incline);
        vec3::zero(&mut self.accel_ground_friction);
    }
}

impl Katamari {
    /// offset: 0x20cd0
    pub(super) fn update_incline_accel_and_gravity(
        &mut self,
        prince: &mut Prince,
        mission_state: &MissionState,
    ) {
        if mission_state.gamemode == GameMode::Ending {
            // in the ending stage, there is no gravity
            vec3::zero(&mut self.velocity.vel_grav);
            return;
        }

        self.airborne_prop_gravity = mission_state
            .stage_config
            .get_airborne_prop_gravity(self.diam_cm);

        vec3::zero(&mut self.velocity.accel_incline);
        if !self.physics_flags.climbing_wall {
            if !self.physics_flags.airborne {
                // if not climbing a wall and grounded (i.e. not airborne):
                vec3::zero(&mut self.velocity.vel_grav);

                if !self.hit_flags.force_flatground {
                    // not wallclimbing, grounded, and not on forced flatground.
                    // then we can check the currently contacted floor to see if it's
                    // steep enough to automatically accelerate the katamari downwards.

                    // consider the slope of the contacted floor, computed as the angle
                    // of the floor normal's y component:
                    let floor_slope = acos_f32(self.contact_floor_normal_unit[1]);

                    if self.params.min_slope_grade_causing_accel < floor_slope / FRAC_PI_2 {
                        // if the slope is steep enough to cause acceleration:

                        // compute the unit rejection of gravity onto the floor:
                        let mut floor_rej_down = [0.0; 3];
                        let mut floor_proj_down = [0.0; 3];
                        vec3_projection(
                            &mut floor_proj_down,
                            &mut floor_rej_down,
                            &VEC3_Y_NEG,
                            &self.contact_floor_normal_unit,
                        );
                        vec3_inplace_zero_small(&mut floor_rej_down, 1e-05);
                        vec3_inplace_normalize(&mut floor_rej_down);

                        // compute some kind of lateral unit velocity and the lateral speed
                        // in its direction
                        let mut vel_xz_unit = match self.physics_flags.no_input_push {
                            true => self.velocity.velocity,
                            false => self.velocity.push_vel_on_floor_unit,
                        };
                        set_y!(vel_xz_unit, 0.0);
                        let speed_xz = vec3::length(&vel_xz_unit);
                        vec3_inplace_normalize(&mut vel_xz_unit);

                        // compute the current incline move type
                        self.physics_flags.incline_move_type =
                            if !self.physics_flags.immobile && speed_xz > 0.0 {
                                // if the player is moving, the incline movetype is determined
                                // by the similarity between the direction of their velocity and
                                // the (??) direction the slope is facing.
                                let similarity = vec3::dot(&vel_xz_unit, &floor_proj_down);
                                let threshold = match self.physics_flags.no_input_push {
                                    true => 0.0,
                                    false => 0.258819,
                                };

                                // compute the incline move type:
                                if self.physics_flags.contacts_wall {
                                    // case 1: if the katamari contacts a wall, set the movetype to
                                    // flatground regardless of the similarity.
                                    KatInclineMoveType::MoveFlatground
                                } else if similarity <= -threshold {
                                    // case 2: `similarity <= -threshold`: moving against incline (i.e. uphill)
                                    KatInclineMoveType::MoveUphill
                                } else if similarity <= threshold {
                                    // case 3: `-threshold < similarity <= threshold`: moving neutral
                                    // with respect to incline (i.e. on flat ground)
                                    KatInclineMoveType::MoveFlatground
                                } else {
                                    // case 4: `threshold < similarity`: moving with incline (i.e. downhill)
                                    KatInclineMoveType::MoveDownhill
                                }
                            } else {
                                // if the player isn't moving, start downhill acceleration
                                // along the incline.
                                self.physics_flags.immobile = false;
                                self.move_downhill_ticks = 10;
                                KatInclineMoveType::MoveDownhill
                            };

                        // update the katamari's incline acceleration based on its incline movetype
                        match self.physics_flags.incline_move_type {
                            KatInclineMoveType::MoveUphill => {
                                // if moving uphill:
                                // incline acceleration is a multiple of the unit rejection.
                                // the multiple is determined by the prince's push strength and the number of
                                // ticks that the katamari has been moving uphill.
                                self.move_uphill_ticks += 1;
                                self.move_downhill_ticks = 0;

                                // TODO: wasn't this already computed
                                let slope = acos_f32(self.contact_floor_normal_unit[1]);
                                let accel_t = inv_lerp_clamp!(
                                    slope,
                                    self.params.min_slope_grade_causing_accel,
                                    self.params.effective_max_slope_grade * FRAC_PI_2
                                );
                                prince.decrease_push_uphill_strength(accel_t);

                                let incline_base_accel = if prince.input_avg_push_len <= 0.0 {
                                    self.scaled_params.not_push_uphill_accel
                                } else {
                                    self.scaled_params.push_uphill_accel
                                };

                                let easein_accel = (self.move_uphill_ticks as f32
                                    / self.params.uphill_accel_easein_duration)
                                    .clamp(0.0, 1.0);
                                let incline_mult =
                                    easein_accel * (self.diam_cm / 50.0) * incline_base_accel;
                                vec3::scale(
                                    &mut self.velocity.accel_incline,
                                    &floor_rej_down,
                                    incline_mult,
                                );
                            }
                            KatInclineMoveType::MoveDownhill => {
                                // if moving downhill:
                                self.move_downhill_ticks += 1;
                                self.move_uphill_ticks = 0;

                                let easein_accel = (self.move_downhill_ticks as f32
                                    / self.params.downhill_accel_easein_duration)
                                    .clamp(0.0, 1.0);
                                let incline_mult = (self.scaled_params.not_push_uphill_accel
                                    * self.diam_cm)
                                    / 50.0
                                    * easein_accel;
                                vec3::scale(
                                    &mut self.velocity.accel_incline,
                                    &floor_rej_down,
                                    incline_mult,
                                );
                            }
                            KatInclineMoveType::MoveFlatground => {
                                self.end_incline_movement(prince);
                            }
                        }
                    } else {
                        // if the contact floor isn't steep enough to accelerate the katamari
                        // downwards automatically:
                        self.end_incline_movement(prince);
                    };
                } else {
                    // if contacting a surface that's forced flatground (i.e. regardless of its
                    // slope, it won't accelerate the katamari downwards automatically)
                    self.end_incline_movement(prince);
                }
            } else {
                // if not wallclimbining and airborne:
                // apply gravity acceleration
                self.velocity.vel_grav[1] -= self.scaled_params.accel_grav;
                self.end_incline_movement(prince);
            }
        } else {
            // if climbing wall:
            self.move_downhill_ticks = 0;
            self.move_uphill_ticks = 0;
            vec3::zero(&mut self.velocity.vel_grav);
        }
    }

    fn end_incline_movement(&mut self, prince: &mut Prince) {
        self.move_downhill_ticks = 0;
        self.move_uphill_ticks = 0;
        self.physics_flags.incline_move_type = KatInclineMoveType::MoveFlatground;
        prince.reset_push_uphill_strength();
    }

    /// offset: 0x22130
    pub(super) fn update_velocity(
        &mut self,
        prince: &mut Prince,
        camera: &Camera,
        mission_state: &MissionState,
    ) {
        let init_vel_accel_len = vec3::length(&self.velocity.vel_accel);

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
                init_vel_accel_len
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

        temp_debug_log!("   push_accel={push_accel}, push_mag={push_mag}");

        self.physics_flags.no_input_push = push_mag <= 0.0;

        if prince.get_flags() & 0x40000 != 0 {
            // if quick shifting or pinching:
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

            prince.get_params().push_mag_speed_mult(push_mag, pre_speed)
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

        let max_speed = self.scaled_params.base_max_speed * base_speed_mult * climb_mult;

        temp_debug_log!("   base_speed_mult={base_speed_mult}, max_speed={max_speed}");

        // TODO_VS: `kat_update_velocity:304-324`

        // compute max speeds from scaled params
        let base_speed = self.scaled_params.base_max_speed;
        self.base_speed = base_speed;
        self.max_forwards_speed = base_speed * self.scaled_params.max_forwards_speed;
        self.max_boost_speed = base_speed * self.scaled_params.max_boost_speed;
        self.max_sideways_speed = base_speed * self.scaled_params.max_sideways_speed;
        self.max_backwards_speed = base_speed * self.scaled_params.max_backwards_speed;

        let mut accel = [0.0; 3];
        let mut is_shoot_brake = false;
        match self.compute_brake_state(prince, camera) {
            BrakeState::NoPush => {
                // case 1: katamari isn't pushing very hard
                self.brake_accel = 0.0;
                let mut cam_forward = [0.0; 3];
                vec3::transform_mat4(
                    &mut cam_forward,
                    &VEC3_Z_POS,
                    &camera.transform.lookat_yaw_rot_inv,
                );
                vec3::transform_mat4(&mut accel, &cam_forward, &prince.get_boost_push_yaw_rot());
            }
            BrakeState::PushBrake => {
                // case 2: katamari is pushing against the velocity, and hard enough to brake
                let mut vel_unit = self.velocity.vel_accel;
                vec3_inplace_normalize(&mut vel_unit);
                push_accel = self.brake_accel;
                vec3::scale(&mut accel, &vel_unit, -1.0);
            }
            BrakeState::PushNoBrake => {
                // case 3: katamari is pushing with the velocity
                self.brake_accel = 0.0;
                let mut cam_forward = [0.0; 3];
                vec3::transform_mat4(
                    &mut cam_forward,
                    &VEC3_Z_POS,
                    &camera.transform.lookat_yaw_rot_inv,
                );
                vec3::transform_mat4(
                    &mut accel,
                    &cam_forward,
                    &prince.get_nonboost_push_yaw_rot(),
                );
            }
            BrakeState::Shoot => {
                // TODO_VS: `kat_update_velocity:433-481`
                is_shoot_brake = true;
            }
        };

        temp_debug_log!("   accel={accel:?}");

        self.brake_accel *= prince.get_uphill_accel_penalty();
        vec3_inplace_normalize(&mut accel);

        if !self.physics_flags.braking {
            self.input_push_dir = prince.get_push_dir();
        }
        prince.oujistate.brake = self.physics_flags.braking;

        // compute acceleration penalty when moving uphill
        let incline_accel_mult = match self.physics_flags.incline_move_type {
            KatInclineMoveType::MoveUphill => prince.get_uphill_accel_penalty(),
            _ => 1.0,
        };

        temp_debug_log!("   incline_accel_mult={incline_accel_mult:?}");

        // compute spine-derived acceleration multiplier (used to smooth the acceleration
        // out i guess)
        self.update_max_speed_ratio(prince);
        let spline_mult = match self.physics_flags.braking {
            true => 1.0,
            false => compute_spline_accel_mult(self.max_speed_ratio),
        };

        temp_debug_log!("   spline_mult={spline_mult}");

        let accel_magnitude = push_accel * push_mag * incline_accel_mult * spline_mult;

        temp_debug_log!("   accel_magnitude={accel_magnitude}");

        if self.physics_flags.contacts_floor {
            // if the katamari contacts a floor, adjust the acceleration direction to be the
            // rejection of the floor from the original acceleration direction.
            let vel_dot_floor = vec3::dot(&accel, &self.contact_floor_normal_unit);
            vec3_inplace_add_scaled(&mut accel, &self.contact_floor_normal_unit, -vel_dot_floor);
            vec3_inplace_zero_small(&mut accel, 1e-05);
            vec3_inplace_normalize(&mut accel);
        }

        if !self.physics_flags.no_input_push {
            // if input is pushing the katamari:
            self.velocity.push_vel_on_floor_unit = accel;
        } else {
            // else if input isn't pushing the katamari:
            vec3::zero(&mut self.velocity.push_vel_on_floor_unit);
            vec3::zero(&mut accel);
        }

        if !self.physics_flags.climbing_wall {
            // if not climbing wall, use the acceleration computed above, scaled by the
            // acceleration magnitude also computed above.
            vec3_inplace_scale(&mut accel, accel_magnitude)
        } else {
            // if climbing wall, accelerate in the opposite direction of the wall normal (??)
            vec3::copy(&mut accel, &self.wallclimb_normal_unit);
            vec3_inplace_scale(&mut accel, -1.0);
        }

        self.velocity.last_vel_accel = self.velocity.vel_accel;

        let mut next_velocity = vec3_from!(+, accel, self.velocity.vel_accel);
        let next_speed = vec3::len(&next_velocity);

        // the katamari's speed cap is greatly increased when moving downhill on floors
        // with the `SpeedCheckOff` hit attribute. (e.g. the mas5/mas8 hill)
        let uncap_speed = self.hit_flags.speed_check_off
            && self.physics_flags.incline_move_type == KatInclineMoveType::MoveDownhill;

        if !is_shoot_brake {
            if !uncap_speed && next_speed > max_speed && next_speed > init_vel_accel_len {
                // apply the speed cap by rescaling the next velocity to `max_speed`
                // TODO: ghidra has two cases here but they seem equivalent? (line ~630)
                let capped_speed = match self.physics_flags.climbing_wall {
                    true => max_speed,
                    false => init_vel_accel_len,
                };

                vec3_inplace_normalize(&mut next_velocity);
                vec3_inplace_scale(&mut next_velocity, capped_speed);
            }

            self.velocity.velocity = next_velocity;
        }

        // if the katamari just bonked, set velocity equal to the initial bonk velocity
        if self.physics_flags.bonked {
            self.velocity.velocity = self.init_bonk_velocity;
            self.physics_flags.bonked = false;
        }

        vec3::normalize(&mut self.velocity.velocity_unit, &self.velocity.velocity);
    }

    fn compute_brake_state(&mut self, prince: &mut Prince, camera: &Camera) -> BrakeState {
        // early exit when the camera is in the "shoot" mode
        if camera.get_mode() == CameraMode::Shoot {
            return BrakeState::Shoot;
        }

        // early exit when the prince isn't pushing the katamari
        if !prince.is_pushing_for_brake() || self.physics_flags.airborne {
            self.physics_flags.braking = false;
            return BrakeState::NoPush;
        }

        // from here, the prince is pushing the katamari and the katamari is grounded.
        let mut vel_accel_unit = self.velocity.vel_accel;
        vec3_inplace_normalize(&mut vel_accel_unit);

        // compute the "unit push forward" direction by taking into account both the camera's forward
        // direction and the prince's input push direction.
        let mut cam_forward = [0.0; 3];
        let mut push_forward_unit = [0.0; 3];
        vec3::transform_mat4(
            &mut cam_forward,
            &VEC3_Z_POS,
            &camera.transform.lookat_yaw_rot_inv,
        );
        vec3::transform_mat4(
            &mut push_forward_unit,
            &cam_forward,
            prince.get_nonboost_push_yaw_rot(),
        );
        vec3_inplace_normalize(&mut push_forward_unit);

        if vec3::dot(&vel_accel_unit, &push_forward_unit) < 0.0 {
            // if the katamari's velocity and the direction being pushed have a negative dot
            // product, that means we're pushing *against* the katamari's velocity, i.e. braking.
            let vel_to_cam_angle = acos_f32(vec3::dot(&vel_accel_unit, &cam_forward));
            let angle = prince.push_sideways_angle_threshold;

            // compute max speed and brake acceleration based on input push direction
            let (max_speed, brake_accel) = if prince.oujistate.dash {
                // braking boost movement
                (self.max_boost_speed, self.scaled_params.brake_boost_force)
            } else if vel_to_cam_angle >= FRAC_PI_2 + angle {
                // braking forwards movement with backwards input
                (
                    self.max_backwards_speed,
                    self.scaled_params.brake_backwards_force,
                )
            } else if vel_to_cam_angle < FRAC_PI_2 - angle {
                // braking backwards movement with forwards input
                (
                    self.max_forwards_speed,
                    self.scaled_params.brake_forwards_force,
                )
            } else {
                // braking sideways movement with sideways input
                (
                    self.max_sideways_speed,
                    self.scaled_params.brake_sideways_force,
                )
            };

            let min_brakeable_speed = max_speed * self.params.brakeable_max_speed_ratio;
            if self.physics_flags.braking || self.speed >= min_brakeable_speed {
                // if either: - the katamari is already braking, or
                //            - it's moving fast enough to start a brake:
                let vel_dot_cam_lateral = -(vel_accel_unit[0] * push_forward_unit[0]
                    + vel_accel_unit[2] * push_forward_unit[2]);

                if self.params.min_brake_angle <= vel_dot_cam_lateral {
                    // if the angle between velocity and push is past the threshold:
                    // all conditions to be braking are satisfied.

                    if !self.physics_flags.braking {
                        // if we aren't already braking:
                        // begin a new brake
                        if !prince.oujistate.wheel_spin {
                            // TODO: compute a VFX id in the above `(max_speed, brake_accel)`
                            // computation, and play that VFX here with the vfx delegate
                        }

                        self.brake_push_dir = prince.get_push_dir();
                        self.brake_accel = brake_accel;
                        // TODO: there's a bunch of random flag checks here, probably no-ops though
                        // (`kat_compute_brake_state:191-193)

                        let _brake_volume = match self.brake_push_dir {
                            Some(PushDir::Forwards) => 0.5,
                            _ => 0.7,
                        };
                        // TODO: play the brake SFX here with volume `brake_volume`

                        // TODO: the simulation sets this timer to 0 here, but does that just mean the brake
                        // vfx plays twice in a row? should this be the regular cooldown?
                        self.brake_vfx_timer = 1;
                    } else {
                        self.brake_vfx_timer -= 1;
                        if self.brake_vfx_timer == 0 {
                            self.brake_vfx_timer = self.params.brake_vfx_cooldown as u16;
                        }
                    }

                    self.physics_flags.braking = true;
                    BrakeState::PushBrake
                } else {
                    // if the angle between velocity and push doesn't yield a brake:
                    // stop braking.
                    self.physics_flags.braking = false;
                    BrakeState::NoPush
                }
            } else {
                // if we're not already braking and we're moving too slow to start a brake,
                // apparently that qualifies as the "no push" result.
                BrakeState::NoPush
            }
        } else {
            // if the dot product was nonnegative, that means the katamari velocity is
            // moving in the same direction as the input push, so we're pushing, but not braking.
            self.physics_flags.braking = false;
            prince.oujistate.dash = false;
            BrakeState::PushNoBrake
        }
    }

    fn update_max_speed_ratio(&mut self, prince: &Prince) {
        let max_speed = if prince.oujistate.dash {
            self.max_boost_speed
        } else {
            match prince.get_push_dir() {
                Some(PushDir::Backwards) => self.max_backwards_speed,
                Some(PushDir::Sideways) => self.max_sideways_speed,
                _ => self.max_forwards_speed,
            }
        };

        self.max_speed_ratio = (self.speed / max_speed).clamp(0.0, 1.0);
    }

    /// Compute the katamari's acceleration due to friction.
    /// offset: 0x21590
    pub(super) fn update_friction_accel(&mut self, prince: &Prince, mission_state: &MissionState) {
        if !self.physics_flags.airborne {
            // if not airborne:
            // next velocity is `velocity + accel_incline + bonus_vel`
            let mut next_vel = vec3_from!(+, self.velocity.velocity, self.velocity.accel_incline);
            vec3_inplace_add_vec(&mut next_vel, &self.bonus_vel);
            let next_speed = vec3::length(&next_vel);

            if next_speed > self.params.min_speed_to_move || next_speed - self.last_speed > 0.0 {
                // if the katamari is moving fast enough to apply friction:
                self.physics_flags.immobile = false;
                let mut t = 1.0;

                match self.physics_flags.grounded_ray_type {
                    Some(KatCollisionRayType::Bottom) => {
                        // TODO_VS: `kat_update_friction:41-45`
                        t = self.params.bottom_ray_friction * self.speed;
                    }
                    Some(_) => {
                        let t = match prince.oujistate.dash {
                            true => {
                                // TODO: could be wrong
                                1.0 - inv_lerp!(
                                    self.speed,
                                    self.max_forwards_speed,
                                    self.max_boost_speed
                                )
                                .clamp(0.0, 1.0)
                            }
                            false => 1.0,
                        };

                        // TODO: remove this when `kat_try_init_vault_speed` is implemented
                        // let max_length_ratio = 1.0;
                        // let angle_btwn_rejs = 1.0;
                        // let k =
                        //     max_length_ratio * angle_btwn_rejs * self.params.nonbottom_ray_friction;
                        // lerp!(t, bottom_friction, bottom_friction * k)
                    }
                    None => {
                        panic_log!("this should not happen");
                    }
                };

                // TODO: ??
                if prince.get_flags() & 0x40000 != 0 {
                    t *= 0.1234
                }

                temp_debug_log!("t={t}, flag={}", prince.get_flags() & 0x40000 != 0);

                vec3::scale(
                    &mut self.velocity.accel_ground_friction,
                    &self.velocity.vel_accel_unit,
                    -t,
                );
            } else {
                self.set_immobile(mission_state);
            }
        } else {
            // if airborne:
            vec3::zero(&mut self.velocity.accel_ground_friction);
        }

        if self.hit_flags.speed_check_off
            && self.physics_flags.incline_move_type == KatInclineMoveType::MoveDownhill
        {
            vec3_inplace_scale(
                &mut self.velocity.accel_ground_friction,
                self.params.speed_check_off_friction_reduction,
            );
        }
    }

    /// Updates the katamari's velocity by applying its acceleration.
    /// offset: 0x1e6a0
    pub(super) fn apply_acceleration(&mut self, mission_state: &MissionState) {
        vec3::zero(&mut self.bonus_vel);

        // start with `velocity + accel_incline`
        let mut vel_accel = vec3_from!(+, self.velocity.velocity, self.velocity.accel_incline);

        let speed0 = vec3::length(&vel_accel);
        if speed0 > 0.0 && !self.physics_flags.climbing_wall {
            // if moving and not climbing a wall, apply ground friction
            vec3_inplace_add_vec(&mut vel_accel, &self.velocity.accel_ground_friction);
        }

        temp_debug_log!(
            "vel: {:?}, accel_inc:{:?}, accel_fric:{:?}",
            self.velocity.velocity,
            self.velocity.accel_incline,
            self.velocity.accel_ground_friction
        );

        if self.hit_flags.speed_check_off
            && self.physics_flags.incline_move_type == KatInclineMoveType::MoveDownhill
        {
            // TODO: `kat_apply_acceleration:44-61` (speedcheckoff acceleration)
        }

        self.velocity.vel_accel = vel_accel;
        vec3::normalize(&mut self.velocity.vel_accel_unit, &self.velocity.vel_accel);

        // compute velocity after gravity acceleration
        vec3::add(
            &mut self.velocity.vel_accel_grav,
            &self.velocity.vel_accel,
            &self.velocity.vel_grav,
        );
        vec3::normalize(
            &mut self.velocity.vel_accel_grav_unit,
            &self.velocity.vel_accel_grav,
        );

        let mut next_vel = self.velocity.vel_accel;

        if self.physics_flags.grounded_ray_type == Some(KatCollisionRayType::Bottom)
            || self.physics_flags.grounded_ray_type == None
        {
            // if grounded via the "bottom" ray, meaning the katamari isn't vaulting:
            // TODO_VS: `kat_apply_acceleration:79-90`
            // TODO_ENDING: `kat_apply_acceleration:91-96`
            // TODO_VS: weird conditional here depending on vs mode, but it's always true in single player
            if !self.physics_flags.climbing_wall {
                // if not wall climbing:
                // TODO: some SHUFPS crap going on here, not clear what it's doing
                vec3_inplace_add_vec(&mut self.center, &self.velocity.vel_accel);
                vec3_inplace_add_vec(&mut self.center, &self.velocity.vel_grav);
            } else {
                // if wall climbing:
                if !self.physics_flags.at_max_climb_height {
                    // if still gaining height from the wall climb:
                    // TODO: SHUFPS
                }
                // TODO: `kat_update_wall_climb()`
            }

            if self.physics_flags.airborne {
                vec3_inplace_add_vec(&mut next_vel, &self.velocity.vel_grav);
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
                self.last_rot_vel_unit = vel_unit;
            } else {
                vel_unit = self.last_rot_vel_unit;
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

            self.spin_rotation_speed = normalize_bounded_angle((vel_len / pivot_circumf) * TAU);
        } else {
            // if katamari is airborne:
            // TODO: `kat_update_rotation_speed:171-221`
        }

        self.spin_rotation_speed = self.spin_rotation_speed.clamp(-PI, PI);
    }

    /// Forcibly set the katamari's velocity to `vel`.
    /// offset: 0x1fd70
    pub fn set_velocity(&mut self, vel: &Vec3) {
        temp_debug_log!("setting velocity to {:?}", vel);
        self.physics_flags.immobile = false;

        // compute speed
        self.speed = vec3::len(vel);

        // compute unit velocity
        let mut vel_unit = *vel;
        vec3_inplace_normalize(&mut vel_unit);

        // set cached velocities
        self.velocity.vel_accel = *vel;
        self.velocity.vel_accel_unit = vel_unit;
        self.velocity.vel_accel_grav = *vel;
        self.velocity.vel_accel_grav_unit = vel_unit;
    }

    /// Forcibly end the katamari's movement, if it's moving.
    /// offset: 0x1f390
    pub fn set_immobile(&mut self, mission_state: &MissionState) {
        self.physics_flags.immobile = true;
        self.speed = 0.0;
        self.wallclimb_cooldown_timer = 10;
        self.last_speed = self.speed;
        self.velocity.reset();
        self.last_velocity.reset();
        self.last_center = self.center;
        self.last_velocity = self.velocity;
        self.bottom = self.center;
        self.bottom[1] -= self.radius_cm;
        self.apply_acceleration(mission_state);
    }
}
