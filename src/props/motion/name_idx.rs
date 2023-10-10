use serde::{Deserialize, Serialize};

use crate::{
    collision::raycast_state::RaycastRef, global::GlobalState, mission::state::MissionState,
    props::prop::Prop,
};

use super::{
    actions::{MotionActionState},
    global_path::GlobalPathState,
};

/// Motion common to all props with the same name index.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NameIndexMotion {
    Normal,
}

impl Default for NameIndexMotion {
    fn default() -> Self {
        Self::Normal
    }
}

impl Prop {
    pub fn update_name_index_motion(
        &mut self,
        motion: Option<&mut MotionActionState>,
        gps: &GlobalPathState,
        mission_state: &MissionState,
        global_state: &mut GlobalState,
        raycasts: RaycastRef,
    ) {
        match self.get_name_index_motion() {
            NameIndexMotion::Normal => {
                // offset: 0x39850
                if let Some(_move_type) = self.get_move_type() {
                    // TODO: (*(code *)(&callback3_generic_moving_states)[prop->pstActionState])()
                    if let Some(motion) = motion {
                        // motion.should_do_alt_motion();
                        motion.update(self, gps, mission_state, global_state, raycasts);
                    }
                }
            }
        }

        // TODO (in `props_update_nonending`)
        // if prop->playerFlags[global_player].isKatDistanceAware...
        // props_update_subroutine(prop)
    }
}
