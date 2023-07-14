use crate::gamestate::GameState;

/// The `Hydrate` trait is used to perform extra initialization of a `GameState`
/// after it's been deserialized. This is used to initialize "redundant" struct
/// fields that are `skip`ped by `serde` serialization (for example, pointers to
/// props on the `Katamari`), as well as pass along the values of delegates from
/// the old state.
/// # Arguments
/// `old_state`: the original game state, unrelated to the deserialized, loaded state.
/// `new_state`: the deserialized, *unhydrated* state that is being loaded.
pub trait Hydrate {
    fn hydrate(&mut self, old_state: &GameState, new_state: &GameState);
}

impl Hydrate for GameState {
    fn hydrate(&mut self, old_state: &GameState, _new_state: &GameState) {
        self.delegates = old_state.delegates.clone();
        self.mono_data = old_state.mono_data.clone();
    }
}
