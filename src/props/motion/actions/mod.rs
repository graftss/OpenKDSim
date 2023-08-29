use serde::{Deserialize, Serialize};

use crate::props::prop::Prop;

use self::sway::SwayAction;

mod path;
mod sway;

pub trait ActionUpdate {
    fn update(&mut self, prop: &mut Prop);
    fn should_do_alt_motion(&self) -> bool;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MotionAction {
    // TODO: in `pmot_misc_init` the prop decides if it's `stationary` based on its root prop's link action
    // this needs to happen for all misc motions
    MiscSway(SwayAction),
    Unimplemented(u16),
}

impl MotionAction {
    pub fn parse_id(action_id: u16) -> Self {
        match action_id {
            0x16 => Self::MiscSway(SwayAction::default()),
            _ => Self::Unimplemented(action_id),
        }
    }

    fn update(&mut self, prop: &mut Prop) {
        match self {
            MotionAction::MiscSway(sway) => sway.update(prop),
            MotionAction::Unimplemented(_) => {}
        }
    }
}

impl Prop {
    pub fn update_motion_action(&mut self) {
        if let Some(motion) = self.motion.clone() {
            let next_motion = match motion {
                MotionAction::MiscSway(mut sway) => {
                    sway.update(self);
                    MotionAction::MiscSway(sway)
                }
                MotionAction::Unimplemented(_) => motion,
            };

            self.motion = Some(next_motion);
        }
    }
}
