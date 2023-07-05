#![feature(const_float_bits_conv)]
#![feature(vec_into_raw_parts)]
#![allow(non_snake_case, dead_code, unused_imports)]

use std::cell::RefCell;

use collision::raycast_state::ray_hits_aabb;
use gamestate::GameState;
use gl_matrix::{common::Vec3, mat3, mat4, quat, vec3};
use mono_data::MonoData;
use props::prop::AddPropArgs;

use crate::{
    collision::raycast_state::RaycastState,
    constants::VEC3_ZERO,
    macros::{set_translation, vec3_from},
    math::vec3_inplace_zero_small,
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
    name_idx: 269, // mandarin slice
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

#[derive(Debug, Clone, Copy)]
struct Test {
    x: [f32; 4],
    y: [i32; 4],
}

unsafe fn test() {
    use mission::stage::*;
    let mono_data_ptr = MAS1_MONO_DATA.as_ptr();

    STATE.with(|_state| {
        let mut state = _state.borrow_mut();

        state.mono_init_start(mono_data_ptr, 1, 2, 3, false, false, false);
        state.add_prop(&CHILD_PROP_ARGS);
        let prop = state.props.get_prop(0).unwrap().as_ref().borrow();

        println!("prop: {:?}", prop);
    });
}

fn test_cam_pos(
    mut pos: &mut Vec3,
    mut target: &mut Vec3,
    kat_center: &Vec3,
    prince_pos: &Vec3,
    kat_to_pos: &Vec3,
    kat_to_target: &Vec3,
) {
    let mut pri_to_kat_unit = [0.0; 3];
    vec3::sub(&mut pri_to_kat_unit, &kat_center, &prince_pos);
    pri_to_kat_unit[1] = 0.0;
    math::vec3_inplace_normalize(&mut pri_to_kat_unit);

    // TODO: `camera_compute_normal_pos_and_target:65-187` (a bunch of unusual cases)
    let mut scaled_pos_offset = [0.0; 3];
    scaled_pos_offset[0] = pri_to_kat_unit[0] * kat_to_pos[2];
    scaled_pos_offset[1] = kat_to_pos[1];
    scaled_pos_offset[2] = pri_to_kat_unit[2] * kat_to_pos[2];

    vec3::add(&mut pos, &kat_center, &scaled_pos_offset);

    // TODO: `camera_compute_normal_pos_and_target:198-213` (handle `SpecialCamera` flag)

    let scaled_target_offset = [
        pri_to_kat_unit[0] * kat_to_target[2],
        kat_to_target[1],
        pri_to_kat_unit[2] * kat_to_target[2],
    ];
    vec3::add(&mut target, &kat_center, &scaled_target_offset);
}

unsafe fn test_monodata() {
    // let mut md = MonoData::default();
    // md.init(MAS1_MONO_DATA.as_ptr());

    // for pd in md.props.iter() {
    //     if let Some(mesh) = pd.collision_mesh {
    //         for _sector in mesh.sectors.iter() {}
    //     }
    // }
}

fn replicate_triangle_hit() {
    let mut raycast_state = RaycastState::default();
    let ray_pts = [
        [-7.9008, -63.08491516, -164.3920288],
        [-9.870688438, -62.24033737, -165.6983337],
    ];
    let triangle = [
        [1.71, 3.0645614, 1.71],
        [1.71, -3.0645616, -1.71],
        [1.71, -3.0645614, 1.71],
    ];
    let transform = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -11.2581, -61.9546, -166.1094,
        1.0,
    ];

    let mut transformed_tri: [[f32; 3]; 3] = Default::default();
    for i in 0..3 {
        vec3::transform_mat4(&mut transformed_tri[i], &triangle[i], &transform);
    }
    println!("transformed_tri: {:?}", transformed_tri);

    let real_impact_point = [-10.19229221, -62.24033737, -165.6983337];

    let ray = vec3_from!(-, ray_pts[1], ray_pts[0]);
    let ray_len = vec3::length(&ray);
    let mut ray_unit = vec3::create();
    vec3::normalize(&mut ray_unit, &ray);

    raycast_state.load_ray(&ray_pts[0], &ray_pts[1]);
    let _t = raycast_state.ray_hits_triangle(&triangle, &transform, false);

    let hit = raycast_state.ray_to_triangle_hit;
    let mut normal_unit = hit.normal_unit;
    vec3_inplace_zero_small(&mut normal_unit, 0.00001);

    let impact_dist = hit.impact_point[2];
    let impact_dist_ratio = impact_dist / ray_len;

    let ray_dot_normal = vec3::dot(&normal_unit, &ray_unit);
    let clip_len = (1.0 - impact_dist_ratio - 0.0005) * ray_len;

    println!("impact_dist: {impact_dist}, impact_dist_ratio: {impact_dist_ratio}");
    println!("dot: {ray_dot_normal}, clip_len: {clip_len}");

    let mut impact_point = vec3::create();
    vec3::scale_and_add(
        &mut impact_point,
        &ray_pts[1],
        &normal_unit,
        clip_len * ray_dot_normal,
    );
    let diff = vec3_from!(-, impact_point, real_impact_point);
    println!("impact point: {:?}, diff: {:?}", impact_point, diff);
    println!("normal_unit: {:?}", normal_unit);
}

fn test_triangle_hit() {
    let mut raycast_state = RaycastState::default();

    let ray = [
        [-7.64054, -62.921543, -164.34523],
        [-9.983479, -62.070057, -164.91994],
    ];
    let triangle = [
        [-1.71, 3.0645614, 1.71],
        [-1.71, -3.0645616, 1.71],
        [-1.71, 3.0645614, -1.71],
    ];
    let transform = [
        -3.6199901e-6,
        1.0,
        0.0,
        0.0,
        -1.0,
        -3.6199901e-6,
        0.0,
        0.0,
        0.0,
        0.0,
        0.99999994,
        0.0,
        63.4754,
        -24.2936,
        1.7704,
        1.0,
    ];

    raycast_state.load_ray(&ray[0], &ray[1]);
    let t = raycast_state.ray_hits_triangle(&triangle, &transform, false);
    println!("t: {t:?}");
    // let triangle = [[-2.6363091468811,0.020901577547193,-4.1993327140808],[-2.6363091468811,-6.4182171821594,4.274188041687],[-2.6363091468811,-6.4182171821594,-4.199333190918]];
    // let ray = [[-26.963624954224,-24.175483703613,17.492353439331],[-25.76362991333,-25.552066802979,15.770400047302]];
    // let result=[[-26.267070770264,-24.974540710449,1.4569648504257],[0.70710015296936,2.0945599032984e-007,0.70711332559586]];
    // let t = 1.46;

    // println!("init state {:?}", raycast_state);
    // let t = raycast_state.ray_hits_triangle(&triangle, &None);
    // println!("result: {:?}", raycast_state.ray_to_triangle_hit);
    // println!("t={}", t);
}

fn main() {
    println!("start");

    // let delegate: fn(a: f32, b: f32) -> i32 = |a, b| 331;

    // let rc_delegate = Rc::new(delegate);
    // let mut raycast_state = crate::collision::raycast_state::RaycastState::default();

    {
        replicate_triangle_hit();
    }
}
