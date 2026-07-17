use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundError, SoundPlaybackFinishReason, SoundPlaybackFinished, SoundPlaybackId,
    SoundPlaybackSettings,
};

use crate::engine::ActivePlayback;
use crate::kira_bridge::static_sound_data;
use crate::poison_recovery::lock_recover;

use super::playback_validation::{
    playback_range_for_settings, validate_playback_settings, validate_playback_speed,
};
use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn play_clip_impl(
        &self,
        clip: SoundClipId,
        settings: SoundPlaybackSettings,
    ) -> Result<SoundPlaybackId, SoundError> {
        validate_playback_settings(&settings)?;
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        let (playback_range, data) = {
            let loaded_clip = state
                .clips
                .get(&clip)
                .ok_or(SoundError::UnknownClip { clip })?;
            let playback_range = playback_range_for_settings(loaded_clip, &settings)?;
            let data = static_sound_data(
                &loaded_clip.kira_data,
                &settings,
                playback_range.start_frame,
                playback_range.end_frame,
            );
            (playback_range, data)
        };
        if !state
            .graph
            .tracks
            .iter()
            .any(|track| track.id == settings.output_track)
        {
            return Err(SoundError::UnknownTrack {
                track: settings.output_track,
            });
        }
        state.next_playback_id += 1;
        let playback_id = SoundPlaybackId::new(state.next_playback_id);
        let active = ActivePlayback {
            clip,
            cursor_frame: playback_range.start_frame,
            cursor_position: playback_range.start_frame as f64,
            gain: settings.gain,
            speed: validate_playback_speed(settings.speed)?,
            looped: settings.looped,
            completion_action: settings.completion_action,
            paused: settings.paused,
            muted: settings.muted,
            range_start_frame: playback_range.start_frame,
            range_end_frame: playback_range.end_frame,
            output_track: settings.output_track,
            pan: settings.pan,
        };
        state.kira.play(playback_id, settings.output_track, data)?;
        if settings.paused {
            state.kira.pause(playback_id)?;
        }
        state.playbacks.insert(playback_id, active);
        Ok(playback_id)
    }

    pub(super) fn stop_playback_impl(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        state.kira.stop(playback)?;
        let active = state
            .playbacks
            .remove(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        state.finished_playbacks.push(SoundPlaybackFinished {
            playback,
            clip: active.clip,
            reason: SoundPlaybackFinishReason::Stopped,
            completion_action: active.completion_action,
            output_track: active.output_track,
        });
        Ok(())
    }
}
