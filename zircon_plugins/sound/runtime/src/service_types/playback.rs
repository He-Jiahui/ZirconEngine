use std::sync::Arc;

#[cfg(test)]
use zircon_runtime::asset::SoundAsset;
use zircon_runtime::asset::{AssetUri, ProjectAssetManager, PROJECT_ASSET_MANAGER_NAME};
use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundClipInfo, SoundError, SoundPlaybackFinishReason, SoundPlaybackFinished,
    SoundPlaybackId, SoundPlaybackSettings,
};

use crate::automation::ensure_finite_value;
use crate::engine::{ActivePlayback, LoadedClip};

use super::playback_validation::{
    playback_range_for_settings, validate_playback_settings, validate_playback_speed,
};
use super::DefaultSoundManager;

impl DefaultSoundManager {
    fn project_asset_manager(&self) -> Result<Arc<ProjectAssetManager>, SoundError> {
        let core = self
            .core
            .as_ref()
            .ok_or_else(|| SoundError::BackendUnavailable {
                detail: "sound manager is not attached to a CoreRuntime".to_string(),
            })?;
        core.resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)
            .map_err(|error| SoundError::BackendUnavailable {
                detail: error.to_string(),
            })
    }

    #[cfg(test)]
    pub(crate) fn insert_clip_for_test(&self, asset: SoundAsset) -> SoundClipId {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        state.next_clip_id += 1;
        let clip_id = SoundClipId::new(state.next_clip_id);
        state.clips.insert(clip_id, LoadedClip { asset });
        clip_id
    }

    pub(super) fn load_clip_impl(&self, locator: &str) -> Result<SoundClipId, SoundError> {
        let uri = AssetUri::parse(locator).map_err(|_| SoundError::InvalidLocator {
            locator: locator.to_string(),
        })?;
        let asset_manager = self.project_asset_manager()?;
        let asset_id =
            asset_manager
                .resolve_asset_id(&uri)
                .ok_or_else(|| SoundError::InvalidLocator {
                    locator: locator.to_string(),
                })?;
        let asset = asset_manager
            .load_sound_asset(asset_id)
            .map_err(|error| SoundError::Decode(error.to_string()))?;

        let mut state = self.state.lock().expect("sound state mutex poisoned");
        if let Some(existing) = state.clip_ids_by_locator.get(locator).copied() {
            return Ok(existing);
        }

        state.next_clip_id += 1;
        let clip_id = SoundClipId::new(state.next_clip_id);
        state
            .clip_ids_by_locator
            .insert(locator.to_string(), clip_id);
        state.clips.insert(clip_id, LoadedClip { asset });
        Ok(clip_id)
    }

    pub(super) fn clip_info_impl(&self, clip: SoundClipId) -> Result<SoundClipInfo, SoundError> {
        let state = self.state.lock().expect("sound state mutex poisoned");
        let clip = state
            .clips
            .get(&clip)
            .ok_or(SoundError::UnknownClip { clip })?;
        Ok(SoundClipInfo {
            locator: clip.asset.uri.to_string(),
            sample_rate_hz: clip.asset.sample_rate_hz,
            channel_count: clip.asset.channel_count,
            frame_count: clip.asset.frame_count(),
            duration_seconds: clip.asset.duration_seconds(),
        })
    }

    pub(super) fn play_clip_impl(
        &self,
        clip: SoundClipId,
        settings: SoundPlaybackSettings,
    ) -> Result<SoundPlaybackId, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let loaded_clip = state
            .clips
            .get(&clip)
            .ok_or(SoundError::UnknownClip { clip })?;
        validate_playback_settings(&settings)?;
        let playback_range = playback_range_for_settings(loaded_clip, &settings)?;
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
        state.playbacks.insert(
            playback_id,
            ActivePlayback {
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
            },
        );
        Ok(playback_id)
    }

    pub(super) fn stop_playback_impl(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
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
