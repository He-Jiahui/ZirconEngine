use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId};

use crate::automation::values::ensure_finite_value;

use super::playback_validation::validate_playback_speed;
use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn pause_playback_impl(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.paused = true;
        Ok(())
    }

    pub(super) fn resume_playback_impl(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.paused = false;
        Ok(())
    }

    pub(super) fn toggle_playback_impl(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.paused = !active.paused;
        Ok(())
    }

    pub(super) fn set_playback_gain_impl(
        &self,
        playback: SoundPlaybackId,
        gain: f32,
    ) -> Result<(), SoundError> {
        ensure_finite_value("playback gain", gain)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.gain = gain;
        Ok(())
    }

    pub(super) fn set_playback_speed_impl(
        &self,
        playback: SoundPlaybackId,
        speed: f32,
    ) -> Result<(), SoundError> {
        let speed = validate_playback_speed(speed)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.speed = speed;
        Ok(())
    }

    pub(super) fn seek_playback_seconds_impl(
        &self,
        playback: SoundPlaybackId,
        seconds: f32,
    ) -> Result<(), SoundError> {
        ensure_finite_value("playback seek seconds", seconds)?;
        if seconds < 0.0 {
            return Err(SoundError::InvalidParameter(
                "playback seek seconds must be non-negative".to_string(),
            ));
        }
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        let active_clip = active.clip;
        let range_start_frame = active.range_start_frame;
        let range_end_frame = active.range_end_frame;
        let clip = state
            .clips
            .get(&active_clip)
            .ok_or(SoundError::UnknownClip { clip: active_clip })?;
        let sample_rate = clip.asset.sample_rate_hz.max(1) as f32;
        let frame_count = clip.asset.frame_count();
        let requested_frame = (seconds * sample_rate).round() as usize;
        let range_end = range_end_frame.unwrap_or(frame_count).min(frame_count);
        let clamped_frame = requested_frame.max(range_start_frame).min(range_end);
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.cursor_frame = clamped_frame;
        active.cursor_position = clamped_frame as f64;
        Ok(())
    }

    pub(super) fn mute_playback_impl(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.muted = true;
        Ok(())
    }

    pub(super) fn unmute_playback_impl(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.muted = false;
        Ok(())
    }

    pub(super) fn toggle_mute_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.muted = !active.muted;
        Ok(())
    }
}
