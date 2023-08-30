use std::slice;

use gl_matrix::common::Mat4;
use serde::{Deserialize, Serialize};

use crate::{
    constants::ZERO,
    delegates::{has_delegates::HasDelegates, DelegatesRef},
    global::GlobalState,
    mission::state::MissionState,
    mono_data::MonoData,
    player::Player,
};

use self::{
    comments::KingCommentState,
    config::{NamePropConfig, NAME_PROP_CONFIGS},
    motion::{actions::MotionAction, global_path::GlobalPathState},
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
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PropsState {
    // TODO_REFACTOR: group `props` and `prop_motions` (and any other `ctrl_idx`-indexed data)
    // into a separate `Props` struct
    // TODO: replace `PropRef` with `Prop` here
    pub props: Vec<PropRef>,
    pub prop_motions: Vec<Option<MotionAction>>,

    pub gps: GlobalPathState,

    /// NOTE: the simulation sort of tracks comment groups, but they aren't actually
    /// used since unity also tracks them and doesn't query the simulation's data.
    /// they also appear to be inaccurately tracked.
    /// so this field isn't used in this simulation at present.
    #[serde(skip)]
    pub comments: KingCommentState,

    pub random: RandomPropsState,

    #[serde(skip)]
    pub config: Option<&'static Vec<NamePropConfig>>,

    pub params: PropParams,

    #[serde(skip)]
    pub delegates: Option<DelegatesRef>,
}

impl PropsState {
    /// Reset ephemeral fields of the props state between attempts.
    pub fn reset(&mut self) {
        self.props.clear();
        self.prop_motions.clear();
        self.gps.init();
        self.random.reset();
        self.config = Some(&NAME_PROP_CONFIGS);
    }
}

impl HasDelegates for PropsState {
    fn get_delegates_ref(&self) -> Option<&DelegatesRef> {
        self.delegates.as_ref()
    }

    fn set_delegates_ref(&mut self, delegates_ref: &DelegatesRef) {
        self.delegates = Some(delegates_ref.clone());
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

        let prop_ref = Prop::new_ref(
            ctrl_idx,
            args,
            area,
            mono_data,
            global,
            &mut self.comments,
            &mut self.random,
            mission_state,
        );

        self.add_prop_motion(&prop_ref);
        self.props.push(prop_ref);
    }

    fn add_prop_motion(&mut self, prop_ref: &PropRef) {
        let prop = prop_ref.borrow();

        let motion_action = prop.get_motion_action().map(MotionAction::parse_id);
        self.prop_motions.push(motion_action);
    }

    pub fn change_next_area(&mut self, area: u8) {
        // TODO_BUG: we can't `retain` here without ruining the property that `ctrl_idx` is an index
        // into `self.props`
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
            self.update_nonending(player, mission_state);
        }
    }

    /// Root function to update all props when not in the `Ending` game mode.
    /// offset: 0x50050
    pub fn update_nonending(&mut self, player: &Player, mission_state: &MissionState) {
        for prop_ref in self.props.iter_mut() {
            let mut prop = prop_ref.borrow_mut();
            if prop.is_disabled() {
                continue;
            }

            let ctrl_idx = prop.get_ctrl_idx();
            let motion_action = self.prop_motions[ctrl_idx as usize].as_mut();

            prop.update_last_pos_and_rotation();
            prop.update_global_state();
            prop.update_child_link();
            prop.update_name_index_motion(motion_action, &self.gps, mission_state);

            // if let Some(_script) = prop.innate_script.as_ref() {
            //     // TODO_PROP_MOTION: call `innate_script`
            // }

            prop.cache_distance_to_players(player);
            prop.update_delta_pos();
            prop.update_transform_unattached();
        }

        // TODO: `props_update_nonending:142-` (updating global path state flags)
    }

    /// Root function to update all props when in the `Ending` game mode.
    /// offset: 0x259f0 (note: this offset is in the middle of a function in the original simulation)
    pub fn update_ending(&mut self) {}

    /// offset: 0x24be0
    pub fn update_prop_alphas(&mut self) {
        // TODO_PROP_ALPHA
        for prop_ref in self.props.iter_mut() {
            let mut prop = prop_ref.borrow_mut();
            if !prop.is_disabled() {
                prop.set_visible(true);
            }
        }
    }
}
