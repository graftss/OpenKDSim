#![feature(vec_into_raw_parts)]
#![allow(non_snake_case, dead_code)]

use std::cell::RefCell;

use gamestate::GameState;
use prop::AddPropArgs;

// reference this first so it's available to all other modules
mod macros;

mod camera;
mod constants;
mod delegates;
mod ending;
mod gamestate;
mod global;
mod input;
mod katamari;
mod mission;
mod mono_data;
mod name_prop_config;
mod preclear;
mod prince;
mod prop;
mod prop_motion;
mod tutorial;
mod util;

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

unsafe fn test() {
    let mono_data_ptr = MAS1_MONO_DATA.as_ptr();

    STATE.with(|state| {
        let mut state = state.borrow_mut();

        state.mono_init_start(mono_data_ptr, 1, 2, 3, 4, 5, 6);

        state.add_prop(CHILD_PROP_ARGS);
        state.add_prop(PARENT_PROP_ARGS);
        state.add_prop(SIB_PROP_ARGS);
        state.add_prop(PARENT_PROP_ARGS);
        state.add_prop(PARENT_PROP_ARGS);
        state.add_prop_set_parent(2, 1);
        state.add_prop_set_parent(0, 1);
        state.add_prop_set_parent(1, 3);
        state.add_prop_set_parent(4, 3);

        state.read_prop_ref(0).clone().borrow_mut().print_links("0");
        state.read_prop_ref(1).clone().borrow_mut().print_links("1");
        state.read_prop_ref(2).clone().borrow_mut().print_links("2");
        state.read_prop_ref(3).clone().borrow_mut().print_links("3");
        state.read_prop_ref(4).clone().borrow_mut().print_links("4");
    });
}

fn main() {
    println!("start");
    unsafe {
        test();
    }
}
