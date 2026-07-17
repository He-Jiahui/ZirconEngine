use zircon_runtime::core::framework::sound::{
    SoundError, SoundPlaybackFinished, SoundPlaybackId, SoundPlaybackStatus,
};

use super::DefaultSoundManager;
use crate::poison_recovery::lock_recover;

impl DefaultSoundManager {
    pub(super) fn playback_empty_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<bool, SoundError> {
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
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
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        let active = state
            .playbacks
            .get(&playback)
            .cloned()
            .ok_or(SoundError::UnknownPlayback { playback })?;
        let sample_rate_hz = state
            .clips
            .get(&active.clip)
            .map(|clip| clip.asset.sample_rate_hz)
            .unwrap_or_default();
        let kira_position = state.kira.playback_position(playback).ok();
        let (cursor_frame, cursor_seconds) = kira_position
            .map(|position| {
                absolute_position_from_kira_slice(
                    position,
                    active.range_start_frame,
                    active.range_end_frame,
                    sample_rate_hz,
                )
            })
            .unwrap_or_else(|| {
                if sample_rate_hz == 0 {
                    (active.cursor_frame, 0.0)
                } else {
                    (
                        active.cursor_frame,
                        active.cursor_position as f32 / sample_rate_hz as f32,
                    )
                }
            });
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
            cursor_frame,
            cursor_seconds,
            output_track: active.output_track,
        })
    }

    pub(super) fn drain_finished_playbacks_impl(
        &self,
    ) -> Result<Vec<SoundPlaybackFinished>, SoundError> {
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        Ok(state.finished_playbacks.drain(..).collect())
    }
}

pub(crate) fn absolute_position_from_kira_slice(
    kira_position_seconds: f64,
    range_start_frame: usize,
    range_end_frame: Option<usize>,
    sample_rate_hz: u32,
) -> (usize, f32) {
    if sample_rate_hz == 0 {
        return (range_start_frame, 0.0);
    }
    let relative_frames = (kira_position_seconds.max(0.0) * sample_rate_hz as f64).round() as usize;
    let absolute_frame = range_start_frame
        .saturating_add(relative_frames)
        .min(range_end_frame.unwrap_or(usize::MAX));
    (
        absolute_frame,
        absolute_frame as f32 / sample_rate_hz as f32,
    )
}
