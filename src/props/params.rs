use serde::{Deserialize, Serialize};

use crate::mission::{config::MissionConfig, GameType};

#[derive(Debug, Serialize, Deserialize)]
pub struct PropParams {
    /// See `destroy_prop_diam_ratio_clearprops`. This ratio applies to normal mission.
    /// offset: 0x9eac8
    pub destroy_diam_ratio_normal: f32,

    /// The largest a prop can be (represented as a ratio of the katamari's volume) such that
    /// it will destroy itself once the katamari moves far enough away from it to have 0 alpha.
    /// This ratio in particular applies to `GameMode::ClearProp` missions.
    /// offset: 0x9eacc
    pub destroy_diam_ratio_clearprops: f32,

    /// See `destroy_prop_diam_ratio_clearprop`. This ratio is smaller, and applies to
    /// missions which have the `keep_smaller_props_alive` flag on.
    /// offset: 0x9ead0
    pub destroy_diam_ratio_reduced: f32,

    /// The ratio of vertical velocity a prop keeps after bouncing off of a surface while airborne.
    /// offset: 0x155230
    pub vertical_vel_decay_after_bounce: f32,

    /// The ratio of lateral velocity a prop keeps after bouncing off of a surface while airborne.
    /// offset: 0x155234
    pub lateral_vel_decay_after_bounce: f32,

    /// The number of ticks a prop spends spinning on the ground after finishing airborne bouncing.
    /// offset: 0x155240
    pub spin_after_landing_ticks: i32,

    /// When a katamari and prop collide, this is the minimum angle difference in their movement
    /// directions which can cause the prop to wobble.
    /// offset: 0x155244
    pub min_hit_angle_for_wobble_deg: f32,
}

impl Default for PropParams {
    fn default() -> Self {
        Self {
            vertical_vel_decay_after_bounce: 0.46,
            lateral_vel_decay_after_bounce: 0.46,
            spin_after_landing_ticks: 60,
            min_hit_angle_for_wobble_deg: 70.0,
            destroy_diam_ratio_normal: 0.234,
            destroy_diam_ratio_reduced: 0.1,
            destroy_diam_ratio_clearprops: 0.145,
        }
    }
}

impl PropParams {
    // Computes how small props need to be relative to the katamari
    // before they're destroyed as they become invisible.
    pub fn compute_destroy_invis_diam_ratio(&self, mission_config: &MissionConfig) -> f32 {
        if mission_config.game_type == GameType::ClearProps {
            self.destroy_diam_ratio_clearprops
        } else if mission_config.keep_smaller_props_alive {
            self.destroy_diam_ratio_reduced
        } else {
            self.destroy_diam_ratio_normal
        }
    }
}
