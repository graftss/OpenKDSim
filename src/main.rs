#![feature(const_float_bits_conv)]
#![feature(vec_into_raw_parts)]
#![allow(non_snake_case, dead_code, unused_imports)]

use std::{cell::RefCell, fs::File, io::Write, num::Wrapping, ops::Range, rc::Rc};

use constants::{FRAC_PI_2, FRAC_PI_90, PI};
use gamestate::GameState;
use gl_matrix::{
    common::{Mat4, Vec3},
    mat3, mat4, quat, vec3,
};
use macros::{inv_lerp_clamp, lerp};
use math::{
    acos_f32, vec3_inplace_normalize, vec3_inplace_scale, vec3_reflection, vec3_unit_proj_and_rej,
};
use mission::{config::MissionConfig, Mission};
use mono_data::MonoData;
use player::{katamari::spline::compute_spline_accel_mult, prince::Prince};
use props::{
    config::NamePropConfig,
    motion::{
        actions::{
            common::is_not_facing_target,
            path::{FollowPath, PathMotion},
        },
        data::prop_paths::{PropPathData, PROP_PATH_DATA},
    },
    prop::{AddPropArgs, Prop},
};
use serde::Serialize;

use crate::{
    collision::raycast_state::RaycastState,
    constants::{VEC3_ZERO, VEC3_Z_POS},
    macros::{f32_close_enough, max, min, set_translation, temp_debug_log, vec3_from},
    math::{vec3_inplace_zero_small, vec3_times_mat4},
    mono_data::PropMonoData,
    props::motion::actions::path::yaw_angle_to_target,
};

// reference this first so it's available to all other modules
pub mod macros;

mod collision;
mod constants;
mod debug;
mod delegates;
mod gamestate;
mod global;
mod math;
mod mission;
mod mono_data;
mod player;
mod props;
mod savestate;
mod util;

thread_local! {
    static STATE: RefCell<GameState> = RefCell::new(GameState::default());
}

// temporary hard copy of monodata for testing.
// monodata for each mission is passed to the simulation from unity when the mission
// is loading, so the simulation itself doesn't actually need a copy of any monodata.
static MAS1_MONO_DATA: &'static [u8] = include_bytes!("./bin/monodata/mission1.bin");

const CHILD_PROP_ARGS: AddPropArgs = AddPropArgs {
    pos_x: 1.0,
    pos_y: 2.0,
    pos_z: 3.0,
    rot_x: 4.0,
    rot_y: 5.0,
    rot_z: 5.0,
    rot_w: 5.0,
    scale_x: 6.0,
    scale_y: 6.0,
    scale_z: 6.0,
    name_idx: 782, // mandarin slice
    loc_pos_type: 0,
    random_group_id: u16::MAX,
    mono_move_type: u16::MAX,
    mono_hit_on_area: u16::MAX,
    link_action: 1,
    extra_action_type: 2,
    unique_name_id: u16::MAX,
    disp_off_area_no: 9,
    vs_drop_flag: 12,
    comment_id: 1,
    comment_group_id: u16::MAX,
    twin_id: u16::MAX,
    shake_off_flag: 1,
};

const PARENT_PROP_ARGS: AddPropArgs = AddPropArgs {
    pos_x: 100.0,
    pos_y: 200.0,
    pos_z: 300.0,
    rot_x: 400.0,
    rot_y: 500.0,
    rot_z: 500.0,
    rot_w: 500.0,
    scale_x: 6.0,
    scale_y: 6.0,
    scale_z: 6.0,
    name_idx: 1251,
    loc_pos_type: u16::MAX,
    random_group_id: u16::MAX,
    mono_move_type: u16::MAX,
    mono_hit_on_area: u16::MAX,
    link_action: 1,
    extra_action_type: 2,
    unique_name_id: u16::MAX,
    disp_off_area_no: 9,
    vs_drop_flag: 12,
    comment_id: 1,
    comment_group_id: u16::MAX,
    twin_id: u16::MAX,
    shake_off_flag: 1,
};

const SIB_PROP_ARGS: AddPropArgs = AddPropArgs {
    pos_x: 100.0,
    pos_y: 200.0,
    pos_z: 300.0,
    rot_x: 400.0,
    rot_y: 500.0,
    rot_z: 500.0,
    rot_w: 500.0,
    scale_x: 6.0,
    scale_y: 6.0,
    scale_z: 6.0,
    name_idx: 1251,
    loc_pos_type: u16::MAX,
    random_group_id: u16::MAX,
    mono_move_type: u16::MAX,
    mono_hit_on_area: u16::MAX,
    link_action: 1,
    extra_action_type: 2,
    unique_name_id: u16::MAX,
    disp_off_area_no: u16::MAX,
    vs_drop_flag: 12,
    comment_id: u16::MAX,
    comment_group_id: u16::MAX,
    twin_id: u16::MAX,
    shake_off_flag: 1,
};

unsafe fn test_new_years_card() {
    use mission::stage::*;
    let mono_data_ptr = MAS1_MONO_DATA.as_ptr();

    STATE.with(|_state| {
        let mut state = _state.borrow_mut();

        state.mono_init_start(mono_data_ptr, 1, 2, 3, false, false, false);
        state.add_prop(&CHILD_PROP_ARGS);
        let prop = state.props.get_prop(0).unwrap().as_ref().borrow();

        println!("prop: {:?}", prop);
        println!("root aabb: {:?}", prop.get_aabb_mesh());
    });
}

fn replicate_init_vault() {
    let contact_floor_normal_unit = [0.0, 1.0, 0.0];
    let vel_unit = [-0.33118119835854, 0.0, -0.94356715679169];
    let vel_accel = [-0.22452616691589, 0.0, -0.63969671726227];
    let fc_ray = [-0.58637666702271, -2.8934364318848, -1.3940563201904];
    let max_forwards_speed = 0.8875373005867;
    let max_boost_speed = 2.6494677066803;
    let fc_ray_len = 3.2648437023163;
    let max_ray_len = 6.7128653526306;
    let radius_cm = 2.6851460933685;
    let speed = 0.67795568704605;
    let vault_tuning_0x7b208 = 0.30000001192093;

    let mut fc_ray_unit = vec3::create();
    vec3::normalize(&mut fc_ray_unit, &fc_ray);

    let mut vel_proj_floor = [0.0; 3];
    let mut vel_rej_floor = [0.0; 3];
    vec3_unit_proj_and_rej(
        &mut vel_proj_floor,
        &mut vel_rej_floor,
        &vel_unit,
        &contact_floor_normal_unit,
    );
    println!("++ vel_rej_floor: {vel_rej_floor:?}");
    vec3_inplace_normalize(&mut vel_rej_floor);

    // compute the unit `fc_ray`
    let mut fc_ray_unit = fc_ray;
    vec3_inplace_normalize(&mut fc_ray_unit);
    vec3_inplace_zero_small(&mut fc_ray_unit, 1e-05);

    println!("++ fc_ray_unit: {fc_ray_unit:?}");

    let mut fc_proj_floor = [0.0; 3];
    let mut fc_rej_floor = [0.0; 3];
    vec3_unit_proj_and_rej(
        &mut fc_proj_floor,
        &mut fc_rej_floor,
        &fc_ray_unit,
        &contact_floor_normal_unit,
    );
    println!("++ fc_rej_floor: {fc_rej_floor:?}");
    // println!("fc_proj_floor: {fc_proj_floor:?}");
    vec3_inplace_zero_small(&mut fc_rej_floor, 1e-05);

    let rej_similarity = vec3::dot(&vel_rej_floor, &fc_rej_floor); // [-1, 1]
                                                                   // let rej_angle = acos_f32(rej_similarity); // [PI, 0]
                                                                   // self.vault_rej_angle_t = inv_lerp_clamp!(rej_angle, 0.0, FRAC_PI_2);
    let vault_rej_angle_t = if rej_similarity < 1.0 {
        let rej_angle = if rej_similarity > -1.0 {
            acos_f32(rej_similarity)
        } else {
            PI
        };
        if rej_angle > FRAC_PI_2 {
            0.0
        } else {
            (FRAC_PI_2 - rej_angle) / FRAC_PI_2
        }
    } else {
        1.0
    };

    println!("++ rej_similarity: {rej_similarity}");
    println!("++ vault_rej_angle_t: {vault_rej_angle_t}");

    let mut new_vel_accel = vec3::create();
    if rej_similarity > FRAC_PI_90 {
        let speed_t = inv_lerp_clamp!(speed, max_forwards_speed, max_boost_speed);
        let ray_len_t = inv_lerp_clamp!(fc_ray_len, radius_cm, max_ray_len);
        let ray_len_k = lerp!(ray_len_t, 1.0, vault_tuning_0x7b208);
        let k = lerp!(speed_t, ray_len_k, 1.0);

        println!("++ speed_t: {speed_t}");
        println!("++ ray_len_t: {ray_len_t}");
        println!("++ ray_len_k: {ray_len_k}");
        println!("++ k: {k}");

        let mut vel_reflect_floor = [0.0; 3];
        vec3_reflection(&mut vel_reflect_floor, &fc_rej_floor, &vel_rej_floor);
        vec3_inplace_scale(&mut vel_reflect_floor, -1.0);

        let dot2 = vec3::dot(&vel_rej_floor, &fc_rej_floor) * 2.0;
        let v1 = [
            vel_rej_floor[0] * dot2 - fc_rej_floor[0], // -(u - dot2*v)
            vel_rej_floor[1] * dot2 - fc_rej_floor[1],
            vel_rej_floor[2] * dot2 - fc_rej_floor[2],
        ];

        let v2 = [
            (1.0 - k) * v1[0] + vel_rej_floor[0] * k,
            (1.0 - k) * v1[1] + vel_rej_floor[1] * k,
            (1.0 - k) * v1[2] + vel_rej_floor[2] * k,
        ];

        println!("vel_reflect_floor: {vel_reflect_floor:?}");
        println!("v1: {v1:?}");
        println!("v2: {v2:?}");

        let __old_vel = vel_accel;

        let mut vel_accel_unit = [0.0; 3];
        vec3::lerp(
            &mut vel_accel_unit,
            &vel_rej_floor,
            &vel_reflect_floor,
            1.0 - k,
        );
        println!("vel_accel after lerp: {vel_accel_unit:?}");
        vec3_inplace_normalize(&mut vel_accel_unit);

        vec3::scale(&mut new_vel_accel, &vel_accel_unit, speed);
    }

    println!("new_vel_accel: {:?}", new_vel_accel);
}

fn replicate_flip_camera() {
    let kat_center = [-22.700000762939, -24.060358047485, 28.0];
    let kat_to_pos = [0.0, 7.1999998092651, -22.39999961853];
    let kat_to_target = [0.0, 0.0, 8.1999998092651];
    let flip_timer = 11;
    let flip_duration = 15;
    let flip_lateral_kat_offset_unit = [0.64278769493103, 0.0, -0.76604443788528];

    let mut pos = vec3::create();
    let target;

    let flip_progress = (flip_duration as f32 - flip_timer as f32) / flip_duration as f32;
    let VEC3_Y_POS = [0.0, 1.0, 0.0];

    let mut pos_to_kat_unit = vec3::create();
    vec3::scale(&mut pos_to_kat_unit, &kat_to_pos, -1.0);
    vec3_inplace_normalize(&mut pos_to_kat_unit);

    println!("++pos_to_kat_unit: {pos_to_kat_unit:?}");

    let mut kat_to_yrefl_pos_unit = kat_to_pos;
    kat_to_yrefl_pos_unit[1] *= -1.0;
    vec3_inplace_normalize(&mut kat_to_yrefl_pos_unit);

    println!("++kat_to_yrefl_pos_unit: {kat_to_yrefl_pos_unit:?}");

    let similarity = vec3::dot(&pos_to_kat_unit, &kat_to_yrefl_pos_unit);
    let base_angle = acos_f32(similarity);

    println!("++similarity: {similarity:?}");
    println!("++base_angle: {base_angle:?}");

    let mut rotation_axis_unit = [1.0, 0.2, 0.0];
    vec3_inplace_normalize(&mut rotation_axis_unit);

    println!("++rotation_axis_unit: {rotation_axis_unit:?}");

    let rotation_angle = base_angle * flip_progress;
    println!("++rotation_angle: {rotation_angle:?}");

    let mut transform = mat4::create();
    mat4::from_rotation(&mut transform, rotation_angle, &rotation_axis_unit);

    println!("++transform: {transform:?}");

    let flip_offset = flip_lateral_kat_offset_unit;

    // FIX: sometimes add pi to `flip_offset_angle` (when flip_offset[0] is negative?)

    let flip_offset_angle = (flip_offset[0] as f32).atan2(flip_offset[2]); //f32::atan2(flip_offset[0], flip_offset[2]);
    println!("flip_offset_angle: {flip_offset_angle:?}");

    ///////////////// if not underwater
    let mut world_kat_to_pos = vec3::create();
    vec3_times_mat4(&mut world_kat_to_pos, &kat_to_pos, &transform);
    println!("++ world_kat_to_pos: {world_kat_to_pos:?}");

    let mut flip_angle_rotation = mat4::create();
    mat4::from_rotation(&mut flip_angle_rotation, -flip_offset_angle, &VEC3_Y_POS);
    println!("++ mat2: {flip_angle_rotation:?}");

    let mut vec1 = vec3::create();
    vec3_times_mat4(&mut vec1, &world_kat_to_pos, &flip_angle_rotation);
    vec1[1] *= -1.0;
    vec1[1] = max!(vec1[1], kat_to_pos[1]);

    println!("++ vec1 final: {vec1:?}");

    vec3::add(&mut pos, &kat_center, &vec1);
    println!("++ final pos: {pos:?}");

    let mut world_kat_to_target = vec3::create();
    vec3_times_mat4(&mut world_kat_to_target, &kat_to_target, &transform);
    println!("world_kat_to_target: {world_kat_to_target:?}");

    let mut kat_to_target = vec3::create();
    vec3_times_mat4(
        &mut kat_to_target,
        &world_kat_to_target,
        &flip_angle_rotation,
    );
    println!("kat_to_target: {kat_to_target:?}");

    target = [
        kat_center[0] + kat_to_target[0],
        kat_center[1] - kat_to_target[1],
        kat_center[2] + kat_to_target[2],
    ];

    println!("final target: {target:?}");
}

#[derive(Debug)]
struct MockProp {
    pub value: u8,
    pub motion: Option<PropMotion>,
}

trait MotionUpdate {
    fn update(prop: &mut MockProp);
}

#[derive(Debug)]
enum PropMotion {
    Oscillate(Oscillate),
}

// master update that switches on every type of motion
impl MotionUpdate for PropMotion {
    fn update(prop: &mut MockProp) {
        match &prop.motion {
            Some(PropMotion::Oscillate(_)) => Oscillate::update(prop),
            None => {}
        }
    }
}

#[derive(Debug)]
enum OscillateState {
    GoUp,
    Chill,
    GoDown,
}

#[derive(Debug)]
struct Oscillate {
    pub state: OscillateState,
    pub chill_timer: u8,
}

impl Default for Oscillate {
    fn default() -> Self {
        Self {
            state: OscillateState::GoUp,
            chill_timer: 0,
        }
    }
}

impl MotionUpdate for Oscillate {
    fn update(prop: &mut MockProp) {
        if let Some(PropMotion::Oscillate(ref mut motion)) = prop.motion {
            match motion.state {
                OscillateState::GoUp => {
                    prop.value += 1;
                    if prop.value > 3 {
                        motion.state = OscillateState::Chill;
                        motion.chill_timer = 4;
                    }
                }
                OscillateState::Chill => {
                    motion.chill_timer -= 1;
                    if motion.chill_timer <= 0 {
                        if prop.value > 3 {
                            motion.state = OscillateState::GoDown;
                        } else {
                            motion.state = OscillateState::GoUp;
                        }
                    }
                }
                OscillateState::GoDown => {
                    prop.value -= 1;
                    if prop.value < 1 {
                        motion.state = OscillateState::Chill;
                        motion.chill_timer = 2;
                    }
                }
            }
        }
    }
}

fn prop_motion_test() {
    let motion = Oscillate::default();
    let mut prop = MockProp {
        value: 0,
        motion: Some(PropMotion::Oscillate(motion)),
    };

    for i in 0..20 {
        PropMotion::update(&mut prop);
        println!("step {:?}: {:?}", i, prop);
    }
}

fn reformat_prop_paths() -> std::io::Result<()> {
    let old_data = PROP_PATH_DATA.paths;
    let new_file_path = "bin/prop_paths.bin";

    let mut file = File::create(new_file_path)?;

    for (idx, path) in old_data.iter().enumerate() {
        // println!("point_ptr: {:?}", path.point_ptr);
        if idx == 84 || idx == 9 || idx == 144 || idx == 125 || idx == 76 || idx == 112 {
            file.write(&u32::MAX.to_le_bytes())?;
        } else {
            file.write(&path.point_idx.to_le_bytes())?;
        }
        file.write(&path.speed.to_bits().to_le_bytes())?;
    }

    Ok(())
}

fn reformat_prop_points() -> std::io::Result<()> {
    let old_data = PROP_PATH_DATA.points;
    let new_file_path = "bin/prop_path_points.bin";

    let mut file = File::create(new_file_path)?;

    for (_idx, point) in old_data.iter().enumerate() {
        file.write(&(-point[0]).to_bits().to_le_bytes())?;
        file.write(&(-point[1]).to_bits().to_le_bytes())?;
        file.write(&(-point[2]).to_bits().to_le_bytes())?;
        file.write(&point[3].to_bits().to_le_bytes())?;
    }

    Ok(())
}

fn test_prop_path_data() {
    let mission = Mission::MAS1;
    let path_idx = 10;
    let points = &PROP_PATH_DATA.get_mission_path_points(mission, path_idx);
    println!("{:?}", points);
}

fn test_ant_init() {
    let mission = Mission::MAS1;

    let mut motion = FollowPath::default();
    motion.set_path_idx(10);

    let mut ant0 = Prop::default();
    ant0.debug_set_ctrl_idx(69);
    ant0.pos = [160.7232, -65.5534, 137.2686];

    let mut ant1 = Prop::default();
    ant1.debug_set_ctrl_idx(69);
    ant1.pos = [161.1982, -65.5534, 143.1274];

    PROP_PATH_DATA.load_initial_target_point_idx(&mut motion, &mut ant0, mission);
    println!("ant0 (expect 32): {:?}", motion.get_target_point_idx());

    PROP_PATH_DATA.load_initial_target_point_idx(&mut motion, &mut ant1, mission);
    println!("ant1 (expect 71): {:?}", motion.get_target_point_idx());
}

fn test_is_not_facing_target() {
    let forward_unit = &[0.0, 0.0, 1.0];
    let to_target_unit = &[-1.0, 0.0, 0.0];

    let N = 100000;
    for i in 0..N - 1 {
        let angle = 2.0 * PI * (i as f32) / (N as f32);
        let result = is_not_facing_target(angle, forward_unit, to_target_unit);
        if !result {
            println!(
                "i={}, angle (deg)={}, result={}",
                i,
                angle * 180.0 / PI,
                result
            );
        }
    }
}

fn main() {
    println!("start");

    // let delegate: fn(a: f32, b: f32) -> i32 = |a, b| 331;

    // let rc_delegate = Rc::new(delegate);
    // let mut raycast_state = crate::collision::raycast_state::RaycastState::default();
    {}
}
