#![allow(non_snake_case, dead_code)]

use crate::gamestate::GameState;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

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
mod util;

thread_local! { static STATE: GameState = GameState::default(); }

pub fn debug_log(str: &str) {
    let path = Path::new(
        "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Katamari Damacy REROLL\\debug.log",
    );
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();

    if let Err(_e) = writeln!(file, "{}", str) {
        eprintln!("oopsie");
    }
}

pub fn main() {
    println!("hi");

    STATE.with(|state| {
        let prop = &state.props.get(2000).unwrap();
        println!("radius: {:?}", prop.get_radius());
    });
}
