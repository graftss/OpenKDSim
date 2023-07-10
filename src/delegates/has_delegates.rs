use gl_matrix::common::Vec3;

use crate::{constants::UNITY_TO_SIM_SCALE, delegates::sound_id::SoundId};

use super::{vfx_id::VfxId, DelegatesRef};

pub trait HasDelegates {
    fn get_delegates_ref(&self) -> Option<&DelegatesRef>;
    fn set_delegates_ref(&mut self, delegates_ref: &DelegatesRef);

    // Convenience wrapper around playing SFX.
    fn play_sound_fx(&self, sound_id: SoundId, volume: f32, pan: i32) {
        if let Some(delegates_ref) = self.get_delegates_ref() {
            if let Some(play_sound_fx) = delegates_ref.borrow().play_sound_fx {
                play_sound_fx(Into::<u16>::into(sound_id) as i32, volume, pan);
            }
        }
    }

    // Convenience wrapper around playing VFX.
    fn play_vfx(
        &self,
        vfx_id: VfxId,
        pos: &Vec3,
        dir: &Vec3,
        scale: f32,
        attach_id: i32,
        player: u8,
    ) {
        if let Some(delegates_ref) = self.get_delegates_ref() {
            if let Some(play_visual_fx) = delegates_ref.borrow().play_visual_fx {
                play_visual_fx(
                    Into::<u16>::into(vfx_id) as i32,
                    pos[0] / UNITY_TO_SIM_SCALE,
                    pos[1] / UNITY_TO_SIM_SCALE,
                    pos[2] / UNITY_TO_SIM_SCALE,
                    dir[0] / UNITY_TO_SIM_SCALE,
                    dir[1] / UNITY_TO_SIM_SCALE,
                    dir[2] / UNITY_TO_SIM_SCALE,
                    scale / UNITY_TO_SIM_SCALE,
                    attach_id,
                    player as i32,
                );
            }
        }
    }
}
