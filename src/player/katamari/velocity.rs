use gl_matrix::{
    common::{Mat4, Vec3},
    mat4, vec3,
};

use crate::{
    constants::{FRAC_PI_2, PI, TAU, VEC3_Y_NEG, VEC3_Y_POS, VEC3_Z_POS},
    delegates::{has_delegates::HasDelegates, sound_id::SoundId, vfx_id::VfxId},
    macros::{inv_lerp, inv_lerp_clamp, lerp, mark_call, max, panic_log, set_y, vec3_from},
    math::{
        acos_f32, normalize_bounded_angle, vec3_inplace_add_scaled, vec3_inplace_add_vec,
        vec3_inplace_normalize, vec3_inplace_scale, vec3_inplace_zero_small, vec3_projection,
    },
    mission::{stage::Stage, state::MissionState, GameMode},
    player::{
        camera::{mode::CameraMode, Camera},
        katamari::{spline::compute_spline_accel_mult, CamRelativeDir},
        prince::{Prince, PushDir},
    },
};

use super::{
    flags::{GroundedRay, KatInclineMoveType},
    KatBoostEffectState, Katamari,
};

/// 0.9998
const ALMOST_1: f32 = f32::from_bits(0x3f7ff2e5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub vel: Vec3,

    /// Current unit velocity
    /// offset: 0x10
    pub vel_unit: Vec3,

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
            .field("velocity", &self.vel)
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
        vec3::zero(&mut self.vel);
        vec3::zero(&mut self.vel_unit);
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
        if !self.physics_flags.climbing {
            if !self.physics_flags.airborne {
                // if not climbing a wall and not airborne:
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
                            true => self.velocity.vel,
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
                                    KatInclineMoveType::Flatground
                                } else if similarity <= -threshold {
                                    // case 2: `similarity <= -threshold`: moving against incline (i.e. uphill)
                                    KatInclineMoveType::Uphill
                                } else if similarity <= threshold {
                                    // case 3: `-threshold < similarity <= threshold`: moving neutral
                                    // with respect to incline (i.e. on flat ground)
                                    KatInclineMoveType::Flatground
                                } else {
                                    // case 4: `threshold < similarity`: moving with incline (i.e. downhill)
                                    KatInclineMoveType::Downhill
                                }
                            } else {
                                // if the player isn't moving, start downhill acceleration
                                // along the incline.
                                self.physics_flags.immobile = false;
                                self.move_downhill_ticks = 10;
                                KatInclineMoveType::Downhill
                            };

                        // update the katamari's incline acceleration based on its incline movetype
                        match self.physics_flags.incline_move_type {
                            KatInclineMoveType::Uphill => {
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
                            KatInclineMoveType::Downhill => {
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
                            KatInclineMoveType::Flatground => {
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
                // if not wallclimbing and airborne:
                // apply gravity acceleration
                self.velocity.vel_grav[1] += self.scaled_params.accel_grav;
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
        self.physics_flags.incline_move_type = KatInclineMoveType::Flatground;
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
            self.init_boost(&prince);
            self.boost_effect_state = Some(super::KatBoostEffectState::Build);
            self.boost_effect_timer = 0;
        }

        prince.oujistate.dash_effect = false;
        if prince.oujistate.dash || prince.oujistate.dash_start {
            match self.boost_effect_state {
                Some(KatBoostEffectState::Build) => {
                    if !self.physics_flags.in_water || self.boost_effect_timer > 0 {
                        prince.oujistate.dash_effect = true;
                        self.boost_effect_timer += 1;
                        if self.boost_effect_timer > self.params.boost_build_duration {
                            self.boost_effect_state = Some(KatBoostEffectState::StopBuilding);
                        }
                        if self.physics_flags.in_water {
                            self.boost_effect_state = Some(KatBoostEffectState::Release);
                            self.boost_effect_timer = self.params.boost_release_duration_in_water;
                        }
                    }
                }
                Some(KatBoostEffectState::StopBuilding) => {
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
                Some(KatBoostEffectState::Release) => {
                    prince.oujistate.dash_effect = false;
                    self.boost_effect_timer -= 1;
                    if self.boost_effect_timer == 0 {
                        self.boost_effect_state = Some(KatBoostEffectState::End);
                    }
                }
                Some(KatBoostEffectState::End) => {
                    prince.oujistate.dash_effect = false;
                }
                None => (),
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

        prince.oujistate.camera_mode = camera.get_mode().into();
        prince.oujistate.climb_wall = self.physics_flags.climbing;
        prince.oujistate.hit_water = self.physics_flags.in_water;
        prince.oujistate.submerge = self.physics_flags.under_water;
        prince.oujistate.camera_state = camera.get_r1_jump_state().into();
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

        let climb_mult = match self.physics_flags.climbing {
            true => self.params.wallclimb_speed_penalty,
            false => 1.0,
        };

        let max_speed = self.scaled_params.base_max_speed * base_speed_mult * climb_mult;

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

        self.brake_accel *= prince.get_uphill_accel_penalty();
        vec3_inplace_normalize(&mut accel);

        if !self.physics_flags.braking {
            self.input_push_dir = prince.get_push_dir();
        }
        prince.oujistate.brake = self.physics_flags.braking;

        // compute acceleration penalty when moving uphill
        let incline_accel_mult = match self.physics_flags.incline_move_type {
            KatInclineMoveType::Uphill => prince.get_uphill_accel_penalty(),
            _ => 1.0,
        };

        // compute spine-derived acceleration multiplier (used to smooth the acceleration
        // out i guess)
        self.update_max_speed_ratio(prince);
        let spline_mult = match self.physics_flags.braking {
            true => 1.0,
            false => compute_spline_accel_mult(self.max_speed_ratio),
        };

        let accel_magnitude = push_accel * push_mag * incline_accel_mult * spline_mult;

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

        if !self.physics_flags.climbing {
            // if not climbing wall, use the acceleration computed above, scaled by the
            // acceleration magnitude also computed above.
            vec3_inplace_scale(&mut accel, accel_magnitude)
        } else {
            // if climbing wall, accelerate in the opposite direction of the wall normal (??)
            vec3::copy(&mut accel, &self.climb_normal_unit);
            vec3_inplace_scale(&mut accel, -1.0);
        }

        self.velocity.last_vel_accel = self.velocity.vel_accel;

        let mut next_velocity = vec3_from!(+, accel, self.velocity.vel_accel);
        let next_speed = vec3::len(&next_velocity);

        // the katamari's speed cap is greatly increased when moving downhill on floors
        // with the `SpeedCheckOff` hit attribute. (e.g. the mas5/mas8 hill)
        let uncap_speed = self.hit_flags.speed_check_off
            && self.physics_flags.incline_move_type == KatInclineMoveType::Downhill;

        if !is_shoot_brake {
            if !uncap_speed && next_speed > max_speed && next_speed > init_vel_accel_len {
                // if the katamari's speed is capped and the next speed
                // apply the speed cap by rescaling the next velocity to `max_speed`
                // TODO: ghidra has two cases here but they seem equivalent? (line ~630)
                let capped_next_speed = if max_speed <= init_vel_accel_len {
                    match self.physics_flags.climbing {
                        true => max_speed,
                        false => init_vel_accel_len,
                    }
                } else {
                    max_speed
                };

                vec3_inplace_normalize(&mut next_velocity);
                vec3_inplace_scale(&mut next_velocity, capped_next_speed);
            }

            self.velocity.vel = next_velocity;
        }

        // if the katamari just bonked, set velocity equal to the initial bonk velocity
        if self.physics_flags.bonked {
            self.velocity.vel = self.init_bonk_velocity;
            self.physics_flags.bonked = false;
        }

        vec3::normalize(&mut self.velocity.vel_unit, &self.velocity.vel);
    }

    /// Initialize boost speed and play the sfx/vfx.
    /// offset: 0x237c0
    fn init_boost(&mut self, prince: &Prince) {
        // TODO_PARAM
        let SW_SPEED_DISP_DURATION = 0xf;

        let prince_to_kat = vec3_from!(-, self.bottom, prince.get_pos());
        vec3::normalize(&mut self.velocity.vel_accel_unit, &prince_to_kat);
        vec3::scale(
            &mut self.velocity.vel_accel,
            &self.velocity.vel_accel_unit,
            self.speed,
        );
        self.sw_speed_disp_timer = SW_SPEED_DISP_DURATION;

        self.play_sound_fx(SoundId::Boost, 1.0, 0);
        self.play_boost_vfx();
    }

    /// Play the VFX associated to the start of a boost.
    /// offset: 0x6f70
    pub fn play_boost_vfx(&self) {
        static DIR: Vec3 = [0.0, 0.0, 0.0];

        self.play_vfx(VfxId::Boost, &self.center, &DIR, self.diam_cm, 1, 0);
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
            // TODO_LOW: these conditions are not in the same order as in the original simulation,
            // which means that if more than one of them is true at the same time, the
            // branch that gets taken might be different.
            let (max_speed, brake_accel, brake_vfx_id) = if prince.oujistate.dash {
                // braking boost movement
                (
                    self.max_boost_speed,
                    self.scaled_params.brake_boost_force,
                    VfxId::BrakeForward,
                )
            } else if vel_to_cam_angle >= FRAC_PI_2 + angle {
                // braking forwards movement with backwards input
                (
                    self.max_backwards_speed,
                    self.scaled_params.brake_backwards_force,
                    VfxId::BrakeBackward,
                )
            } else if vel_to_cam_angle < FRAC_PI_2 - angle {
                // braking backwards movement with forwards input
                (
                    self.max_forwards_speed,
                    self.scaled_params.brake_forwards_force,
                    VfxId::BrakeForward,
                )
            } else {
                // braking sideways movement with sideways input
                (
                    self.max_sideways_speed,
                    self.scaled_params.brake_sideways_force,
                    VfxId::BrakeSideways,
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
                            self.play_vfx_at_bottom(brake_vfx_id);
                        }

                        self.brake_push_dir = prince.get_push_dir();
                        self.brake_accel = brake_accel;
                        // TODO_LOW: there's a bunch of random flag checks here, probably no-ops though

                        if !self.physics_flags.braking {
                            // TODO_PARAM
                            let brake_volume = match self.brake_push_dir {
                                Some(PushDir::Forwards) => 0.5,
                                Some(PushDir::Sideways) => 0.7,
                                Some(PushDir::Backwards) => 1.0,
                                _ => {
                                    panic_log!(
                                        "unexpected brake push dir: {:?}",
                                        self.brake_push_dir
                                    );
                                }
                            };
                            self.play_sound_fx(SoundId::Brake, brake_volume, 0);
                        }
                        self.brake_vfx_timer = 0;
                    } else {
                        self.brake_vfx_timer -= 1;
                        if self.brake_vfx_timer < 1 {
                            self.brake_vfx_timer = self.params.brake_vfx_cooldown as i16;
                            self.play_vfx_at_bottom(brake_vfx_id);
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

    /// Play the vfx `vfx_id` at the bottom of the katamari.
    /// offset: 0x6a30
    fn play_vfx_at_bottom(&self, vfx_id: VfxId) {
        static DIR: Vec3 = [0.0, 0.0, 0.0];

        // I guess there's no point playing vfx at the bottom if the katamari is underwater and
        // you can't even see it
        if self.physics_flags.in_water {
            return;
        }

        self.play_vfx(vfx_id, &self.bottom, &DIR, self.diam_cm, -1, self.player)
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
            let mut next_vel = vec3_from!(+, self.velocity.vel, self.velocity.accel_incline);
            vec3_inplace_add_vec(&mut next_vel, &self.bonus_vel);
            let next_speed = vec3::length(&next_vel);

            if next_speed > self.params.min_speed_to_move || next_speed - self.last_speed > 0.0 {
                // if the katamari is moving fast enough to apply friction:
                self.physics_flags.immobile = false;
                let mut t;

                match self.physics_flags.grounded_ray_type {
                    GroundedRay::Bottom => {
                        // TODO_VS: `kat_update_friction:41-45`
                        t = self.params.bottom_ray_friction * self.speed;
                    }
                    _ => {
                        let t_inner = match prince.oujistate.dash {
                            true => {
                                1.0 - inv_lerp!(
                                    self.speed,
                                    self.max_forwards_speed,
                                    self.max_boost_speed
                                )
                                .clamp(0.0, 1.0)
                            }
                            false => 1.0,
                        };
                        let bottom_friction = self.params.bottom_ray_friction * self.speed;
                        let max_length_ratio = self.vault_ray_max_len_ratio;
                        let angle_btwn_rejs = self.vault_rej_angle_t;
                        let k =
                            max_length_ratio * angle_btwn_rejs * self.params.nonbottom_ray_friction;
                        t = lerp!(t_inner, bottom_friction, bottom_friction * k);
                    }
                };

                // apply significantly less friction when the prince is either quick shifting
                // or pinching
                if prince.get_flags() & 0x40000 != 0 {
                    // TODO_PARAM
                    t *= 0.1234
                }

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
            && self.physics_flags.incline_move_type == KatInclineMoveType::Downhill
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
        let mut vel_accel = vec3_from!(+, self.velocity.vel, self.velocity.accel_incline);

        let speed0 = vec3::length(&vel_accel);
        if speed0 > 0.0 && !self.physics_flags.climbing {
            // if moving and not climbing a wall, apply ground friction
            vec3_inplace_add_vec(&mut vel_accel, &self.velocity.accel_ground_friction);
        }

        if self.hit_flags.speed_check_off
            && self.physics_flags.incline_move_type == KatInclineMoveType::Downhill
        {
            // while the `speedcheckoff` flag is on, the katamari constantly accelerates.
            // this block caps max speed to a multiple of its usual value (by default, 3x).

            // TODO_PARAM
            let speed_check_off_speed_boost = 3.0;
            let speed_cap = self.max_forwards_speed
                * speed_check_off_speed_boost
                * self.params.forwards_speed_mult;
            let speed = vec3::length(&vel_accel);
            if speed > speed_cap {
                vec3_inplace_scale(&mut vel_accel, speed_cap / speed);
            }
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

        if self.physics_flags.grounded_ray_type.is_bottom() {
            // if grounded via the "bottom" ray, meaning the katamari isn't vaulting:
            // TODO_VS: `kat_apply_acceleration:79-90`
            // TODO_ENDING: `kat_apply_acceleration:91-96`
            // TODO_VS: weird conditional here depending on vs mode, but it's always true in single player
            if !self.physics_flags.climbing {
                // if not wall climbing:
                vec3_inplace_add_vec(&mut self.center, &self.velocity.vel_accel);
                vec3_inplace_add_vec(&mut self.center, &self.velocity.vel_grav);
            } else {
                // if wall climbing:
                if !self.physics_flags.at_max_climb_height {
                    // if still gaining height from the wall climb:
                    vec3_inplace_add_vec(&mut self.center, &self.velocity.vel_accel);
                }
                self.update_climb_position();
            }

            if self.physics_flags.airborne {
                vec3_inplace_add_vec(&mut next_vel, &self.velocity.vel_grav);
            }

            self.speed = vec3::length(&next_vel);

            self.update_size_features();
            self.update_rotation_speed(&next_vel);
            self.update_transform_unvaulted();
        } else {
            self.update_size_features();
            self.update_transform_vaulted();
        }

        vec3_inplace_zero_small(&mut self.center, 0.001);
        self.base_speed_ratio = self.speed / self.base_speed;
        self.vault_prop_decay_mult =
            1.0 - self.base_speed_ratio * self.params.vault_prop_pull_to_center_mult;

        self.update_shell_points();

        // TODO_VS: `kat_apply_acceleration:166-196`

        if mission_state.stage == Stage::World {
            self.physics_flags.can_emit_smoke = self.diam_cm > 1200.0;
        }
    }

    /// Compute the shell collision points around the katamari's boundary. The shell points are
    /// positioned based on how the katamari's center moved since the previous tick.
    /// offset: 0x23b70
    fn update_shell_points(&mut self) {
        // compute shell top and bottom
        vec3::scale(&mut self.shell_top, &VEC3_Y_POS, self.radius_cm);
        vec3::scale(&mut self.shell_bottom, &VEC3_Y_POS, -self.radius_cm);

        // compute the distance moved and the unit vector in the direction moved
        vec3::subtract(&mut self.delta_pos_unit, &self.center, &self.last_center);
        self.delta_pos_len = vec3::length(&self.delta_pos_unit);
        vec3_inplace_normalize(&mut self.delta_pos_unit);

        if self.physics_flags.immobile {
            // if the katamari isn't moving:
            // set `shell_vec` to the zero vector
            vec3::zero(&mut self.shell_vec);
        } else {
            // if the katamari is moving:
            // set `shell_vec` to `delta_pos_unit`.
            // the original simulation recomputes it here, so i guess we will too
            vec3::subtract(&mut self.shell_vec, &self.center, &self.last_center);
            vec3_inplace_normalize(&mut self.shell_vec)
        }

        // scale `shell_vec` to have the same length as the katamari's radius (unless it's 0)
        vec3_inplace_scale(&mut self.shell_vec, self.radius_cm);

        // compute the `left_lateral_unit` vector
        let mut left_lateral_unit = [0.0; 3];
        if self.physics_flags.immobile {
            // if the katamari isn't moving:
            // the original simulation does a bunch of stuff here, but it seems to
            // ultimately just set `left_lateral_unit` to 0 (which it already is)
        } else {
            // if the katamari is moving:
            let mut move_lateral_unit = self.delta_pos_unit;
            set_y!(move_lateral_unit, 0.0);
            vec3_inplace_normalize(&mut move_lateral_unit);

            // compute the rotation matrix to rotate a point 90 degrees to the left
            let mut left_rot_mat = [0.0; 16];
            mat4::from_y_rotation(&mut left_rot_mat, -FRAC_PI_2);

            vec3::transform_mat4(&mut left_lateral_unit, &move_lateral_unit, &left_rot_mat);
        }

        // compute left and right points as multiples of `left_lateral_unit`
        vec3::scale(&mut self.shell_left, &left_lateral_unit, self.radius_cm);
        vec3::scale(&mut self.shell_right, &left_lateral_unit, -self.radius_cm);

        // compute top-left/right points by normalizing the sum of the top and the left/right points
        vec3::add(&mut self.shell_top_left, &self.shell_top, &self.shell_left);
        vec3_inplace_normalize(&mut self.shell_top_left);
        vec3_inplace_scale(&mut self.shell_top_left, self.radius_cm);

        vec3::add(
            &mut self.shell_top_right,
            &self.shell_top,
            &self.shell_right,
        );
        vec3_inplace_normalize(&mut self.shell_top_right);
        vec3_inplace_scale(&mut self.shell_top_right, self.radius_cm);
    }

    pub fn update_rotation_speed(&mut self, velocity: &Vec3) {
        mark_call!("update_rotation_speed", self.debug_should_log());

        if self.physics_flags.braking {
            return self.rotation_speed = 0.0;
        }

        let speed = vec3::len(velocity);
        let pivot_circumf = max!(self.fc_ray_len, 0.1) * TAU;

        if !self.physics_flags.airborne {
            // 0x200ba
            let mut net_normal_unit = [0.0, 0.0, 0.0];

            if !self.physics_flags.climbing {
                // if not airborne and not climbing a wall:
                vec3::add(
                    &mut net_normal_unit,
                    &self.contact_floor_normal_unit,
                    &self.contact_wall_normal_unit,
                );
                vec3_inplace_normalize(&mut net_normal_unit);
                if vec3::len(&net_normal_unit) < 0.9 {
                    vec3::copy(&mut net_normal_unit, &VEC3_Y_NEG);
                }
            } else {
                // if not airborne and climbing a wall, set net normal to `<0,1,0>`
                set_y!(net_normal_unit, 1.0);
            }

            let mut net_normal_rot = Mat4::default();
            mat4::from_rotation(&mut net_normal_rot, FRAC_PI_2, &net_normal_unit);

            let mut vel_unit = Vec3::default();
            if !self.physics_flags.immobile {
                // if the katamari is not airborne and moving:
                vec3::normalize(&mut vel_unit, &velocity);
            } else {
                vel_unit = VEC3_Y_NEG;
            }

            if vec3::dot(&vel_unit, &net_normal_unit) >= ALMOST_1 {
                return self.rotation_speed = 0.0;
            }

            // compute spin rotation axis
            vec3::transform_mat4(&mut self.rotation_axis_unit, &vel_unit, &net_normal_rot);

            // set y component to zero and renormalize
            set_y!(self.rotation_axis_unit, 0.0);
            let spin_rot_len = vec3::len(&self.rotation_axis_unit);

            if spin_rot_len < 0.5 {
                vec3::copy(&mut self.rotation_axis_unit, &self.camera_side_vector);
            }

            if self.speed <= 0.0 {
                return self.rotation_speed = 0.0;
            }

            self.rotation_speed = normalize_bounded_angle((speed / pivot_circumf) * TAU);
        } else {
            // if katamari is airborne:

            let mut vel_unit = velocity.clone();
            vec3_inplace_normalize(&mut vel_unit);

            // if falling almost entirely vertically, no rot speed
            if vel_unit[1] >= ALMOST_1 {
                return self.rotation_speed = 0.0;
            }

            let lateral_vel_unit = (vel_unit[0] * vel_unit[0] + vel_unit[2] * vel_unit[2]).sqrt();
            if lateral_vel_unit <= 0.0 {
                vec3::copy(&mut self.rotation_axis_unit, &self.camera_side_vector);
            } else {
                let mut rotation_mat = mat4::create();
                mat4::from_rotation(&mut rotation_mat, FRAC_PI_2, &VEC3_Y_POS);

                set_y!(vel_unit, 0.0);
                vec3::transform_mat4(&mut self.rotation_axis_unit, &vel_unit, &rotation_mat);
                vec3_inplace_normalize(&mut self.rotation_axis_unit);
            }

            self.rotation_speed = if speed <= 0.0001 {
                0.0
            } else {
                normalize_bounded_angle(speed / pivot_circumf * TAU)
            };
        }

        self.rotation_speed = self.rotation_speed.clamp(-PI, PI);
    }

    /// Forcibly set the katamari's velocity to `vel`.
    /// offset: 0x1fd70
    pub fn set_velocity(&mut self, vel: &Vec3) {
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
        self.wallclimb_cooldown_timer = 0;
        self.last_speed = self.speed;
        self.velocity.reset();
        self.last_velocity.reset();
        self.last_center = self.center;
        self.last_velocity = self.velocity;
        self.bottom = self.center;
        self.bottom[1] -= self.radius_cm;
        self.apply_acceleration(mission_state);
    }

    /// Compute and store the katamari's velocity relative to the camera
    /// offset: 0x23520
    pub fn update_cam_relative_dir(&mut self, camera: &Camera) {
        self.cam_relative_dir = if self.physics_flags.immobile {
            // case 1: katamari is immobile
            None
        } else {
            // compute the camera-forward direction
            let mut cam_forward = [0.0; 3];
            vec3::transform_mat4(
                &mut cam_forward,
                &VEC3_Z_POS,
                &camera.transform.lookat_yaw_rot_inv,
            );

            // compute unit lateral katamari velocity
            let mut kat_lateral_vel = self.velocity.vel_accel_grav;
            set_y!(kat_lateral_vel, 0.0);
            vec3_inplace_normalize(&mut kat_lateral_vel);

            let similarity = vec3::dot(&cam_forward, &kat_lateral_vel);
            let threshold = self.params.cam_relative_vel_sideways_threshold;
            if similarity < -threshold {
                // case 1: velocity is moving backwards relative to camera
                Some(CamRelativeDir::Backwards)
            } else if similarity > threshold {
                // case 2: velocity is moving forwards relative to camera
                Some(CamRelativeDir::Forwards)
            } else {
                // case 3: velocity is within the interval `[-threshold, threshold]`, meaning that
                // the katamari is moving sideways relative to camera forward.

                // (??) Re-transform the lateral velocity relative relative to the camera's yaw rot
                // so that sideways left and right movement are distinguished via the x component.
                let mut relative_vel = [0.0; 3];
                vec3::transform_mat4(
                    &mut relative_vel,
                    &kat_lateral_vel,
                    &camera.transform.lookat_yaw_rot,
                );

                if relative_vel[0] >= 0.0 {
                    Some(CamRelativeDir::Right)
                } else {
                    Some(CamRelativeDir::Left)
                }
            }
        }
    }
}
