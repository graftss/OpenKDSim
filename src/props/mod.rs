use std::slice;

use gl_matrix::common::Mat4;

use crate::{
    constants::ZERO, delegates::DelegatesRef, global::GlobalState, mission::state::MissionState,
    mono_data::MonoData, player::Player,
};

use self::{
    comments::KingCommentState,
    config::{NamePropConfig, NAME_PROP_CONFIGS},
    motion::global_path::GlobalPathState,
    params::PropParams,
    prop::{AddPropArgs, Prop, PropRef},
    random::RandomPropsState,
};

mod comments;
pub mod config;
pub mod debug;
pub mod motion;
pub mod params;
pub mod prop;
pub mod random;

/// State of all props in the current mission.
#[derive(Debug, Default)]
pub struct PropsState {
    pub props: Vec<PropRef>,
    pub global_paths: GlobalPathState,

    /// NOTE: the simulation sort of tracks comment groups, but they aren't actually
    /// used since unity also tracks them and doesn't query the simulation's data.
    /// they also appear to be inaccurately tracked.
    /// so this field isn't used in this simulation at present.
    pub comments: KingCommentState,

    pub random: RandomPropsState,

    pub config: Option<&'static Vec<NamePropConfig>>,
    pub params: PropParams,
    pub delegates: Option<DelegatesRef>,
}

impl PropsState {
    /// Reset ephemeral fields of the props state between attempts.
    pub fn reset(&mut self) {
        self.props.clear();
        self.global_paths.init();
        self.random.reset();
        self.config = Some(&NAME_PROP_CONFIGS);
    }
}

impl PropsState {
    /// An immutable iterator over all props.
    pub fn props_iter(&self) -> impl Iterator<Item = &PropRef> {
        self.props.iter()
    }

    /// A mutable iterator over all props.
    pub fn props_iter_mut(&mut self) -> impl Iterator<Item = &mut PropRef> {
        self.props.iter_mut()
    }

    /// Get an immutable reference to the prop with the given `ctrl_idx`.
    pub fn get_prop(&self, ctrl_idx: usize) -> Option<&PropRef> {
        self.props.get(ctrl_idx)
    }

    /// Get a mutable reference to the prop with the given `ctrl_idx`.
    pub fn get_mut_prop(&mut self, ctrl_idx: usize) -> Option<&mut PropRef> {
        self.props.get_mut(ctrl_idx)
    }

    /// Mimicks the `GetPropAttached` API function.
    /// Returns the number of 3-byte prop statuses written to `out`.
    pub unsafe fn get_attach_statuses(&self, out: *mut u8, kat_diam_int: i32) -> i32 {
        let mut num_props = 0;

        for (ctrl_idx, prop_ref) in self.props.iter().enumerate() {
            let prop = prop_ref.borrow();
            if !prop.is_initialized() {
                break;
            }

            let status = out.offset((ctrl_idx * 3).try_into().unwrap());
            prop.get_attach_status(status, kat_diam_int);
            num_props += 1;
        }

        num_props
    }

    /// Creates a new prop from the provided arguments and adds it to the `props` list.
    pub fn add_prop(
        &mut self,
        global: &mut GlobalState,
        mission_state: &MissionState,
        ctrl_idx: u16,
        args: &AddPropArgs,
        area: u8,
        mono_data: &MonoData,
    ) {
        if args.loc_pos_type != 0 {
            self.random.record_random_prop(
                global,
                mission_state.mission as usize,
                args.random_group_id as usize,
            );
        }

        let prop = Prop::new(
            ctrl_idx,
            args,
            area,
            mono_data,
            global,
            &mut self.comments,
            &mut self.random,
        );
        self.props.push(prop);
    }

    pub fn change_next_area(&mut self, area: u8) {
        // destroy props which have the new area as their "display off" area.
        self.props.retain(|prop_ref| {
            let prop_cell = prop_ref.clone();
            let prop = prop_cell.borrow_mut();
            prop.check_destroy_on_area_load(area)
        });
    }

    /// Return a pointer to the internal name string of the prop with control index `ctrl_idx`.
    /// Used by unity in some insanely weird way that possibly doesn't even do anything, but whatever.
    pub fn get_internal_prop_name(&self, ctrl_idx: usize) -> *const u8 {
        if let Some(configs) = self.config {
            let name_idx = self
                .get_prop(ctrl_idx)
                .map_or(0, |prop| prop.clone().borrow().get_name_idx());

            if let Some(config) = configs.get(name_idx as usize) {
                return config.internal_name.as_ptr();
            }
        }

        ZERO.as_ptr()
    }

    /// Mimicks the `GetPropMatrices` API function.
    /// Returns the number
    pub unsafe fn get_prop_matrices(&self, out: *mut f32) -> i32 {
        let mut next_mat = out;
        let mut result = 0;

        for prop_ref in self.props.iter() {
            let prop = prop_ref.borrow();
            if !prop.is_initialized() {
                break;
            }

            // Convert the `f32` pointer into `out` to a matrix pointer.
            let mat: &mut Mat4 = slice::from_raw_parts_mut(next_mat, 16).try_into().unwrap();

            // Copy the next prop's matrix into `out`.
            prop.unsafe_copy_transform(mat);

            // Increment the pointer into `out` to the next matrix.
            next_mat = next_mat.offset(16);
            result += 1;
        }

        result
    }

    /// Root method to update all props.
    /// offset: 0x259c0
    pub fn update(&mut self, player: &Player, mission_state: &MissionState) {
        // for prop_ref in self.props_iter_mut() {
        //     let mut prop = prop_ref.borrow_mut();
        //     if prop.get_ctrl_idx() != 0x11f {
        //         prop.set_disabled(1);
        //     }
        // }

        // TODO_ZONE: `props_update_nonending:28` (`kat_find_zone()`; this should probably be moved somewhere else)
        if mission_state.is_ending() {
            self.update_ending();
        } else {
            self.update_nonending(player)
        }
    }

    /// Root function to update all props when not in the `Ending` game mode.
    /// offset: 0x50050
    pub fn update_nonending(&mut self, player: &Player) {
        for prop_ref in self.props.iter_mut() {
            let mut prop = prop_ref.borrow_mut();
            prop.update_nonending(player);
        }
    }

    /// Root function to update all props when in the `Ending` game mode.
    /// offset: 0x259f0 (note: this offset is in the middle of a function in the original simulation)
    pub fn update_ending(&mut self) {}
}
