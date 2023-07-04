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
    collision::raycast_state::RaycastState, constants::VEC3_ZERO, macros::set_translation,
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

fn test_triangle_hit() {
    // let mut raycast_state = RaycastState::default();

    // let triangle = [[-2.6363091468811,0.020901577547193,-4.1993327140808],[-2.6363091468811,-6.4182171821594,4.274188041687],[-2.6363091468811,-6.4182171821594,-4.199333190918]];
    // let ray = [[-26.963624954224,-24.175483703613,17.492353439331],[-25.76362991333,-25.552066802979,15.770400047302]];
    // let result=[[-26.267070770264,-24.974540710449,1.4569648504257],[0.70710015296936,2.0945599032984e-007,0.70711332559586]];
    // let t = 1.46;

    // raycast_state.load_ray(&ray[0], &ray[1]);
    // println!("init state {:?}", raycast_state);
    // let t = raycast_state.ray_hits_triangle(&triangle, &None);
    // println!("result: {:?}", raycast_state.ray_to_triangle_hit);
    // println!("t={}", t);

    let mut out = [0.0, 0.0, 0.0];
    // let p0 = [-26.9048, -24.06036, 26.7104];
    // let p1 = [-26.904846, -34.10034, 26.71042252];
    // let aabb_min = [-175.171, -65.931, 87.703];
    // let aabb_max = [-26.905, -34.100, 26.7104225];
    let result = ray_hits_aabb(
        &[30.0, 2.0, 1.0],
        &[-30.0, 2.0, 1.0],
        &[0.0, 0.0, 0.0],
        &[10.0, 10.0, 10.0],
        &mut out,
    );
    // let result = ray_hits_aabb(&p0, &p1, &aabb_min, &aabb_max, &mut out);
    println!("result={}, out={:?}", result, out)
}

fn test_mandarin_bbox() {
    let mandarin_transform_init = [
        0.002576,
        0.9961967,
        0.0870953,
        0.0,
        -0.83553123,
        0.0499979,
        -0.547163,
        0.0,
        -0.5494370,
        -0.071361511,
        0.83248,
        0.0,
        -13.9263998,
        -25.134098,
        -7.061699867,
        1.0,
    ];

    let mut mandarin_transform = mat4::create();

    // mat4::transpose(&mut mandarin_transform, &mandarin_transform_init);
    mat4::copy(&mut mandarin_transform, &mandarin_transform_init);

    let mut mandarin_rot = mat3::create();
    mat3::from_mat4(&mut mandarin_rot, &mandarin_transform_init);

    println!("mandarin_rot: {:?}", mandarin_rot);

    let mut mandarin_quat = quat::create();
    quat::from_mat3(&mut mandarin_quat, &mandarin_rot);
    println!("mandarin_quat: {:?}", mandarin_quat);

    let mandarin_aabb_min = [-1.48049, -1.78526, -0.999617];
    let mandarin_aabb_max = [1.48049, 1.78526, 0.999617];
    println!(
        "min: {:?}, len={:?}",
        mandarin_aabb_min,
        vec3::len(&mandarin_aabb_min)
    );
    println!(
        "max: {:?}, len={:?}",
        mandarin_aabb_max,
        vec3::len(&mandarin_aabb_max)
    );

    let mut out_min = vec3::create();
    let mut out_max = vec3::create();

    vec3::transform_mat4(&mut out_min, &mandarin_aabb_min, &mandarin_transform);
    vec3::transform_mat4(&mut out_max, &mandarin_aabb_max, &mandarin_transform);
    println!("out_min: {:?}, len={:?}", out_min, vec3::len(&out_min));
    println!("out_max: {:?}, len={:?}", out_max, vec3::len(&out_max));

    vec3::transform_mat3(&mut out_min, &mandarin_aabb_min, &mandarin_rot);
    vec3::transform_mat3(&mut out_max, &mandarin_aabb_max, &mandarin_rot);
    println!("out_min: {:?}, len={:?}", out_min, vec3::len(&out_min));
    println!("out_max: {:?}, len={:?}", out_max, vec3::len(&out_max));
}

fn test_prop_attached_transform() {
    let attached_transform = [
        -0.19054834544659,
        -0.89341020584106,
        0.40682968497276,
        1.0,
        0.87931722402573,
        0.028920263051987,
        0.47535651922226,
        1.0,
        -0.43645426630974,
        0.4483103454113,
        0.78008151054382,
        1.0,
        -5.0977182388306,
        -22.424993515015,
        15.465428352356,
        1.0,
    ];
    let init_attached_transform = [
        0.27798637747765,
        0.95570695400238,
        -0.096681341528893,
        0.0,
        -0.38871890306473,
        0.20396044850349,
        0.89849746227264,
        0.0,
        0.87841999530792,
        -0.21218830347061,
        0.42820021510124,
        0.0,
        1.6850664615631,
        -0.42052561044693,
        2.0007081031799,
        1.0,
    ];
    let kat_transform = [
        -0.77816718816757,
        0.13420666754246,
        0.6135516166687,
        0.0,
        0.089848108589649,
        -0.94306635856628,
        0.32024013996124,
        0.0,
        0.62159723043442,
        0.30432695150375,
        0.72180438041687,
        0.0,
        -4.9923057556152,
        -23.656593322754,
        13.122102737427,
        1.0,
    ];

    let mut tmp = mat4::create();
    mat4::multiply(&mut tmp, &kat_transform, &init_attached_transform);
    // mat4::get_translation(&mut pos, &attached_transform);

    println!("tmp: {:?}", tmp);
    println!("attached_transform: {:?}", attached_transform);
}

fn main() {
    println!("start");

    // let delegate: fn(a: f32, b: f32) -> i32 = |a, b| 331;

    // let rc_delegate = Rc::new(delegate);
    // let mut raycast_state = crate::collision::raycast_state::RaycastState::default();

    {
        test_mandarin_bbox();
    }
}
