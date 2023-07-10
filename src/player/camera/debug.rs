use crate::macros::debug_log;

use super::CameraState;

impl CameraState {
    pub fn debug_log_r1_jump_state(&self) {
        debug_log!("r1 jump state:");
        debug_log!(
            "  ctrl pt idx={}, ctrl pt={:?}",
            self.kat_offset_ctrl_pt_idx,
            self.kat_offset_ctrl_pt
        );
        debug_log!(
            "  init_pos={:?}, target={:?}",
            self.r1_jump_init_pos,
            self.r1_jump_target
        );
        debug_log!(
            "  translation={:?}, last_translation={:?}",
            self.r1_jump_translation,
            self.r1_jump_last_translation
        );
        debug_log!(
            "  state={:?}, counter={}, duration={}, peak_height={}, height_ratio={}",
            self.r1_jump_state,
            self.r1_jump_counter,
            self.r1_jump_duration,
            self.r1_jump_peak_height,
            self.r1_jump_height_ratio
        );
    }
}
