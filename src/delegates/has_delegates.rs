use crate::fx_ids::SoundId;

use super::DelegatesRef;

pub trait HasDelegates {
    fn get_delegates_ref(&self) -> Option<&DelegatesRef>;
    fn set_delegates_ref(&mut self, delegates_ref: &DelegatesRef);

    // Convenience wrapper around playing SFX.
    fn play_sound_fx(&self, sound_id: SoundId, volume: f32, pan: i32) {
        if let Some(delegates_ref) = self.get_delegates_ref() {
            if let Some(play_sound_fx) = delegates_ref.borrow().play_sound_fx {
                play_sound_fx(sound_id.into(), volume, pan);
            }
        }
    }
}
