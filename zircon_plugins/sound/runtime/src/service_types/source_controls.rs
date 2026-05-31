use zircon_runtime::core::framework::sound::{SoundError, SoundSourceId};

use crate::automation::values::ensure_finite_value;

use super::playback_validation::validate_playback_speed;
use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn pause_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.playing = false;
        Ok(())
    }

    pub(super) fn resume_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.playing = true;
        Ok(())
    }

    pub(super) fn toggle_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.playing = !voice.descriptor.playing;
        Ok(())
    }

    pub(super) fn set_source_gain_impl(
        &self,
        source: SoundSourceId,
        gain: f32,
    ) -> Result<(), SoundError> {
        ensure_finite_value("source gain", gain)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.gain = gain;
        Ok(())
    }

    pub(super) fn set_source_speed_impl(
        &self,
        source: SoundSourceId,
        speed: f32,
    ) -> Result<(), SoundError> {
        let speed = validate_playback_speed(speed)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.speed = speed;
        Ok(())
    }

    pub(super) fn mute_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.muted = true;
        Ok(())
    }

    pub(super) fn unmute_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.muted = false;
        Ok(())
    }

    pub(super) fn toggle_mute_source_impl(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.muted = !voice.descriptor.muted;
        Ok(())
    }
}
