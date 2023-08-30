use serde::{Deserialize, Serialize};

use crate::props::prop::Prop;

use super::actions::MotionAction;

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
    pub fn update_name_index_motion(&mut self, motion: Option<&mut MotionAction>) {
        match self.get_name_index_motion() {
            NameIndexMotion::Normal => {
                // offset: 0x39850
                if let Some(_move_type) = self.get_move_type() {
                    // TODO: (*(code *)(&callback3_generic_moving_states)[prop->pstActionState])()
                    if let Some(motion) = motion {
                        // motion.should_do_alt_motion();
                        self.update_motion_action(motion);
                    }
                }
            }
        }

        // TODO
        // if prop->playerFlags[global_player].isKatDistanceAware...
        // props_update_subroutine(prop)
    }
}
