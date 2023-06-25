#![feature(const_float_bits_conv)]
#![feature(vec_into_raw_parts)]
#![allow(non_snake_case, dead_code, unused_imports)]

use std::cell::RefCell;

use gamestate::GameState;
use gl_matrix::{common::Vec3, mat4, quat, vec3};
use props::prop::AddPropArgs;

use crate::collision::raycast_state::RaycastState;

// reference this first so it's available to all other modules
pub mod macros;

mod collision;
mod constants;
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
static MAS1_MONO_DATA: &'static [u8] = include_bytes!("../bin/mono_data_MAS1.bin");

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
    name_idx: 837, // B pencil
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
    // let mono_data_ptr = MAS1_MONO_DATA.as_ptr();

    STATE.with(|_state| {
        // let mut state = state.borrow_mut();

        // state.mono_init_start(mono_data_ptr, 1, 2, 3, 4, 5, 6);
        // state.add_prop(CHILD_PROP_ARGS);
        // let prop = state.props[0].as_ref().borrow();
        // println!("{}", prop.max_aabb_y());
        // println!("{:#?}", x);
        // println!("{:?}", state.mono_data.props.get(12).unwrap().aabbs)

        let mut sc = StageConfig::default();
        StageConfig::get(&mut sc, 2);
        println!("stage config: {sc:?}");
    });

    let mut s = mission::config::MissionConfig::default();
    mission::config::MissionConfig::get(&mut s, 1);
    // let mut params = mission::config::CamScaledCtrlPt::default()

    // println!("max size: {:#?}", s.scaled_params_max_size);

    let mesh = &player::katamari::collision::mesh::KAT_MESHES[1];
    println!("mesh points:\n {:?}", mesh.points);
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

fn test_triangle_hit() {
    // let triangle = [[-2.6363091468811,0.020901577547193,-4.1993327140808],[-2.6363091468811,-6.4182171821594,4.274188041687],[-2.6363091468811,-6.4182171821594,-4.199333190918]];
    // let ray = [[-26.963624954224,-24.175483703613,17.492353439331],[-25.76362991333,-25.552066802979,15.770400047302]];
    // let result=[[-26.267070770264,-24.974540710449,1.4569648504257],[0.70710015296936,2.0945599032984e-007,0.70711332559586]];
    // let t = 1.46;
}

fn main() {
    println!("start");

    // let delegate: fn(a: f32, b: f32) -> i32 = |a, b| 331;

    // let rc_delegate = Rc::new(delegate);
    // let mut raycast_state = crate::collision::raycast_state::RaycastState::default();

    {
        test_triangle_hit();
    }
}
