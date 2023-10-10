use serde::{Deserialize, Serialize};

use crate::{
    collision::raycast_state::RaycastRef,
    props::prop::{Prop, PropAnimationType, PropMotionFlags},
};

use super::MotionAction;

#[derive(Debug, Serialize, Deserialize)]
enum ZoneTriggerState {
    Init,
    WaitForTrigger,
}

impl Default for ZoneTriggerState {
    fn default() -> Self {
        Self::Init
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ZoneTrigger {
    /// offset: 0x32 (on base motion state)
    do_alt_action: bool,

    /// The state of the motion's state machine.
    /// offset: 0x0
    state: ZoneTriggerState,

    /// The zone associated to the prop's motion.
    /// offset: 0x1
    zone: Option<u8>,

    /// (??) True if the prop has behavior 0x25.
    /// offset: 0x2
    is_behavior_0x25: bool,

    /// (??) Possibly unused.
    /// offset: 0x3
    field_0x3: u8,

    /// If the katamari has at least this much volume, the prop may be scared by it
    /// (depending on its behavior).
    /// offset: 0x4
    scary_kat_vol_m3: f32,

    /// (??) Possibly unused.
    /// offset: 0x8
    field_0x8: u64,

    /// (??) Possibly unused.
    /// offset: 0x10
    field_0x10: u64,

    /// (??) Possibly unused.
    /// offset: 0x18
    field_0x18: u64,
}

impl MotionAction for ZoneTrigger {
    fn should_do_alt_action(&self) -> bool {
        self.do_alt_action
    }

    fn get_zone(&self) -> Option<u8> {
        self.zone
    }
}

impl ZoneTrigger {
    /// The main update behavior for the `ZoneTrigger` action.
    /// offset: 0x3c230
    pub fn update(&mut self, prop: &mut Prop, raycast_ref: RaycastRef) {
        match self.state {
            ZoneTriggerState::Init => self.update_state_init(prop, raycast_ref),
            ZoneTriggerState::WaitForTrigger => self.update_state_wait_for_trigger(prop),
        }
    }

    /// offset: 0x3c250
    fn update_state_init(&mut self, prop: &mut Prop, raycast_ref: RaycastRef) {
        self.do_alt_action = false;

        self.zone = raycast_ref.borrow_mut().find_zone_below_point(
            &prop.pos,
            prop.get_radius(),
            &prop.get_unattached_transform(),
        );
        assert!(self.zone.is_some());

        // TODO_PARAM (`PROP_ATTACH_VOL_RATIO`)
        self.scary_kat_vol_m3 = prop.get_compare_vol_m3() / 0.1;
        prop.stationary = true;

        match prop.get_behavior() {
            Some(0x25) => {
                prop.animation_type = PropAnimationType::MovingForward;
                self.is_behavior_0x25 = true;
            }
            Some(_) => {
                prop.animation_type = PropAnimationType::Waiting;
                self.is_behavior_0x25 = false;
            }
            _ => unreachable!(),
        }

        self.state = ZoneTriggerState::WaitForTrigger;
        prop.motion_flags.insert(PropMotionFlags::WaitForTrigger);
    }

    /// offset: 0x3c2e0
    fn update_state_wait_for_trigger(&mut self, prop: &mut Prop) {
        if prop.alt_motion_action.is_none() {
            return;
        }

        // TODO_HIGH: call alt motion trigger based on behavior type
        // up next: just add all alt motion triggers, indexed by behavior
        let should_do_alt_motion = false;

        if should_do_alt_motion {
            self.do_alt_action = true;
            self.state = ZoneTriggerState::Init;
            self.zone = None;
            self.is_behavior_0x25 = false;
            self.field_0x3 = 0;
            self.scary_kat_vol_m3 = 0.0;
            self.field_0x8 = 0;
            self.field_0x10 = 0;
            self.field_0x18 = 0;
        }
    }
}
