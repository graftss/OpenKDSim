#![feature(const_float_bits_conv)]
#![feature(vec_into_raw_parts)]
#![allow(non_snake_case, dead_code)]

use std::cell::RefCell;

use gamestate::GameState;
use prop::AddPropArgs;

// reference this first so it's available to all other modules
mod macros;

mod camera;
mod collision;
mod constants;
mod delegates;
mod ending;
mod gamestate;
mod global;
mod input;
mod katamari;
mod math;
mod mission;
mod mono_data;
mod name_prop_config;
mod preclear;
mod prince;
mod prop;
mod prop_motion;
mod simulation_params;
mod tutorial;
mod util;
mod vsmode;

thread_local! {
    static STATE: RefCell<GameState> = RefCell::new(GameState::default());
}

static MAS1_MONO_DATA: &'static [u8] = include_bytes!("data/mono_data_MAS1.bin");

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
    let mono_data_ptr = MAS1_MONO_DATA.as_ptr();

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        state.mono_init_start(mono_data_ptr, 1, 2, 3, 4, 5, 6);
        state.add_prop(CHILD_PROP_ARGS);
        let prop = state.props[0].as_ref().borrow();
        println!("{}", prop.max_aabb_y());
        // println!("{:#?}", x);
        // println!("{:?}", state.mono_data.props.get(12).unwrap().aabbs)
    });
}

fn main() {
    println!("start");

    unsafe {
        test();
    }
}
