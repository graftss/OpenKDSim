use std::{cell::RefCell};

use crate::{
    delegates::has_delegates::HasDelegates,
    gamestate::GameState,
    mission::{config::MissionConfig, stage::StageConfig, state::MissionState},
    player::{
        animation::Animation,
        camera::{Camera, CameraState},
        katamari::Katamari,
        Player,
    },
    props::{config::NamePropConfig, prop::Prop, PropsState},
};

/// The `Hydrate` trait is used to perform extra initialization of a `GameState`
/// after it's been deserialized. This is used to initialize "redundant" struct
/// fields that are `skip`ped by `serde` serialization (for example, pointers to
/// props on the `Katamari`), as well as pass along the values of delegates from
/// the old state.
/// # Arguments
/// `old_state`: the original game state, unrelated to the deserialized, loaded state.
/// `new_state`: the deserialized, *unhydrated* state that is being loaded.
pub trait Hydrate {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>);
}

impl Hydrate for GameState {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>) {
        let old_state = old_state_ref.borrow();
        self.delegates = old_state.delegates.clone();
        self.mono_data = old_state.mono_data.clone();
        self.mission_state.hydrate(old_state_ref);
        self.props.hydrate(old_state_ref);
        for player in self.players.iter_mut() {
            player.hydrate(old_state_ref);
            player.katamari.hydrate_prop_refs(&self.props);
        }
    }
}

impl Hydrate for Prop {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>) {
        let name_idx = self.get_name_idx();

        let prop_mono_data = &old_state_ref.borrow().mono_data.props[name_idx as usize];
        let config = NamePropConfig::get(name_idx);
        self.init_mono_data_fields(prop_mono_data, config)
    }
}

impl Hydrate for PropsState {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>) {
        let old_state = old_state_ref.borrow();
        self.set_delegates_ref(&old_state.delegates);
        self.config = old_state.props.config;
    }
}

impl Hydrate for MissionState {
    fn hydrate(&mut self, _old_state_ref: &RefCell<GameState>) {
        MissionConfig::get(&mut self.mission_config, self.mission as u8);
        StageConfig::get(&mut self.stage_config, self.stage.into());
    }
}

impl Hydrate for CameraState {
    fn hydrate(&mut self, old_state: &RefCell<GameState>) {
        self.set_delegates_ref(&old_state.borrow().delegates);
    }
}

impl Hydrate for Camera {
    fn hydrate(&mut self, old_state: &RefCell<GameState>) {
        self.state.hydrate(old_state);
    }
}

impl Hydrate for Animation {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>) {
        self.set_delegates(&old_state_ref.borrow().delegates);
    }
}

impl Hydrate for Katamari {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>) {
        self.set_delegates_ref(&old_state_ref.borrow().delegates);
        // TODO: delete `attached_props` and `contact_prop` propref fields on `Katamari`
    }
}

impl Hydrate for Player {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>) {
        self.animation.hydrate(old_state_ref);
        self.camera.hydrate(old_state_ref);
        self.katamari.hydrate(old_state_ref);
    }
}
