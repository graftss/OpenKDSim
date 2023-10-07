use std::cell::RefCell;

use crate::{
    delegates::has_delegates::HasDelegates,
    gamestate::GameState,
    mission::{config::MissionConfig, stage::StageConfig, state::MissionState},
    player::{
        animation::Animation,
        camera::{Camera, CameraState},
        katamari::Katamari,
        prince::Prince,
        Player,
    },
    props::{config::NamePropConfig, prop::Prop, PropsState},
};

// TODO_REFACTOR: this seemed like a reasonable idea but it ends up being pretty
// pointless, with most of the work just being to pass along delegate refs.
// it should be replaced with something that feels nicer, whenever the requirements
// of savestates change and this demonstrates more friction.
/// The `Hydrate` trait is used to perform extra initialization of a `GameState`
/// after it's been deserialized. This is used to initialize "redundant" struct
/// fields that are `skip`ped by `serde` serialization (for example, pointers to
/// props on the `Katamari`), as well as pass along the values of delegates from
/// the old state.
/// # Arguments
/// `old_state_ref`: a reference to the previous game state, before a new state was loaded.
pub trait Hydrate {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>);
}

impl Hydrate for GameState {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>) {
        let old_state = old_state_ref.borrow();
        self.delegates = old_state.delegates.clone();
        self.mono_data = old_state.mono_data.clone();
        self.mission_state.hydrate(old_state_ref);

        // NOTE: props need to be hydrated before player, since the katamari uses props to
        // hydrate itself
        self.props.hydrate(old_state_ref);

        for player in self.players.iter_mut() {
            player.hydrate(old_state_ref);
            player.katamari.hydrate_prop_refs(&self.props);
            player.katamari.initialize_collision_rays();
        }
    }
}

impl Hydrate for PropsState {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>) {
        let old_state = old_state_ref.borrow();
        self.set_delegates_ref(&old_state.delegates);
        self.raycasts = Some(old_state.raycast.clone());
        self.config = old_state.props.config;

        for prop_ref in self.props.iter() {
            prop_ref.borrow_mut().hydrate(old_state_ref);
        }

        self.hydrate_prop_links();
    }
}

impl Hydrate for Prop {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>) {
        let name_idx = self.get_name_idx();

        // rebuild mono data
        let prop_mono_data = &old_state_ref.borrow().mono_data.props[name_idx as usize];
        let config = NamePropConfig::get(name_idx);
        self.init_mono_data_fields(prop_mono_data, config);
    }
}

impl Hydrate for MissionState {
    fn hydrate(&mut self, _old_state_ref: &RefCell<GameState>) {
        MissionConfig::get(&mut self.mission_config, self.mission as u8);
        StageConfig::get(&mut self.stage_config, self.stage.into());
    }
}

impl Hydrate for CameraState {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>) {
        let delegates = &old_state_ref.borrow().delegates;
        self.set_delegates_ref(delegates);
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
        let delegates = &old_state_ref.borrow().delegates;
        self.set_delegates_ref(delegates);
        self.raycasts.set_delegates_ref(delegates);
        // TODO: delete `attached_props` and `contact_prop` propref fields on `Katamari`
    }
}

impl Hydrate for Prince {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>) {
        self.set_delegates_ref(&old_state_ref.borrow().delegates);
    }
}

impl Hydrate for Player {
    fn hydrate(&mut self, old_state_ref: &RefCell<GameState>) {
        self.animation.hydrate(old_state_ref);
        self.camera.hydrate(old_state_ref);
        self.katamari.hydrate(old_state_ref);
        self.prince.hydrate(old_state_ref);
    }
}
