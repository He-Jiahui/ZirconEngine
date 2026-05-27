use zircon_runtime::core::framework::sound::{
    SoundError, SoundPlaybackFinished, SoundPlaybackId, SoundPlaybackStatus,
};

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn playback_empty_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<bool, SoundError> {
        let state = self.state.lock().expect("sound state mutex poisoned");
        if state.playbacks.contains_key(&playback) {
            return Ok(false);
        }
        if state
            .finished_playbacks
            .iter()
            .any(|finished| finished.playback == playback)
        {
            return Ok(true);
        }
        Err(SoundError::UnknownPlayback { playback })
    }

    pub(super) fn playback_status_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<SoundPlaybackStatus, SoundError> {
        let state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        let cursor_seconds = state
            .clips
            .get(&active.clip)
            .map(|clip| {
                if clip.asset.sample_rate_hz == 0 {
                    0.0
                } else {
                    active.cursor_position as f32 / clip.asset.sample_rate_hz as f32
                }
            })
            .unwrap_or_default();
        Ok(SoundPlaybackStatus {
            playback,
            clip: active.clip,
            paused: active.paused,
            muted: active.muted,
            looped: active.looped,
            completion_action: active.completion_action,
            gain: active.gain,
            speed: active.speed,
            range_start_frame: active.range_start_frame,
            range_end_frame: active.range_end_frame,
            cursor_frame: active.cursor_frame,
            cursor_seconds,
            output_track: active.output_track,
        })
    }

    pub(super) fn drain_finished_playbacks_impl(
        &self,
    ) -> Result<Vec<SoundPlaybackFinished>, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        Ok(state.finished_playbacks.drain(..).collect())
    }
}
