//! 查询时回收 Kira 已结束句柄，分别记录一次性播放和自然完成的持续声源；设备停机只记录一次性播放的停止，并解除持续声源的后端绑定供重启。
use std::sync::Arc;

use kira::sound::static_sound::StaticSoundData;
use zircon_runtime::asset::SoundAsset;
use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundError, SoundPlaybackCompletionAction, SoundPlaybackFinishReason,
    SoundPlaybackFinished, SoundPlaybackId, SoundSourceFinishReason, SoundSourceFinished,
    SoundSourceId, SoundSourceInput, SoundTrackId,
};

use super::SoundEngineState;

#[derive(Clone, Debug)]
pub(crate) struct LoadedClip {
    pub(crate) asset: Arc<SoundAsset>,
    pub(crate) kira_data: StaticSoundData,
}

pub(crate) struct PreparedKiraDeactivation {
    finished_playbacks: Vec<SoundPlaybackFinished>,
}

impl LoadedClip {
    pub(crate) fn new(asset: SoundAsset) -> Result<Self, SoundError> {
        let kira_data = crate::kira_bridge::cached_static_sound_data(&asset)?;
        Ok(Self {
            asset: Arc::new(asset),
            kira_data,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ActivePlayback {
    pub(crate) clip: SoundClipId,
    pub(crate) cursor_frame: usize,
    pub(crate) cursor_position: f64,
    pub(crate) gain: f32,
    pub(crate) speed: f32,
    pub(crate) looped: bool,
    pub(crate) completion_action: SoundPlaybackCompletionAction,
    pub(crate) paused: bool,
    pub(crate) muted: bool,
    pub(crate) range_start_frame: usize,
    pub(crate) range_end_frame: Option<usize>,
    pub(crate) output_track: SoundTrackId,
    pub(crate) pan: f32,
}

impl SoundEngineState {
    pub(crate) fn prepare_kira_deactivation(
        &mut self,
        detached: &[SoundPlaybackId],
    ) -> Result<PreparedKiraDeactivation, SoundError> {
        let finished_count = detached
            .iter()
            .filter(|playback| self.playbacks.contains_key(playback))
            .count();
        let mut finished_playbacks = Vec::new();
        finished_playbacks
            .try_reserve_exact(finished_count)
            .map_err(|error| SoundError::BackendUnavailable {
                detail: format!("unable to reserve retired playback records: {error}"),
            })?;
        // Reconciliation extends the owner journal after the visible Kira swap. Reserve that
        // destination before publication so the post-swap path cannot grow a Vec or fail.
        self.finished_playbacks
            .try_reserve_exact(finished_count)
            .map_err(|error| SoundError::BackendUnavailable {
                detail: format!("unable to reserve retired playback journal: {error}"),
            })?;
        for playback in detached {
            if let Some(active) = self.playbacks.get(playback) {
                finished_playbacks.push(playback_finished(
                    *playback,
                    active.clone(),
                    SoundPlaybackFinishReason::Stopped,
                ));
            }
        }
        Ok(PreparedKiraDeactivation { finished_playbacks })
    }

    pub(crate) fn reconcile_prepared_kira_deactivation(
        &mut self,
        detached: Vec<SoundPlaybackId>,
        prepared: PreparedKiraDeactivation,
    ) {
        for playback in detached {
            if self.playbacks.remove(&playback).is_some() {
                continue;
            }
            if let Some(source) = self.source_for_kira_playback(playback) {
                if let Some(voice) = self.sources.get_mut(&source) {
                    voice.kira_playback = None;
                }
            }
        }
        self.finished_playbacks.extend(prepared.finished_playbacks);
    }

    pub(crate) fn poll_kira_completions(&mut self) {
        let completed = self.kira.drain_finished_playbacks();
        self.reconcile_kira_completions(completed);
    }

    pub(crate) fn deactivate_kira(&mut self) {
        self.poll_kira_completions();
        let detached = self.kira.deactivate();
        self.reconcile_kira_deactivation(detached);
    }

    pub(crate) fn reconcile_kira_completions(&mut self, completed: Vec<SoundPlaybackId>) {
        for playback in completed {
            if let Some(active) = self.playbacks.remove(&playback) {
                self.finished_playbacks.push(playback_finished(
                    playback,
                    active,
                    SoundPlaybackFinishReason::Completed,
                ));
                continue;
            }
            if let Some(source) = self.source_for_kira_playback(playback) {
                if let Some(voice) = self.sources.remove(&source) {
                    self.finished_sources.push(source_finished(
                        source,
                        voice,
                        SoundSourceFinishReason::Completed,
                    ));
                }
            }
        }
    }

    pub(crate) fn reconcile_kira_deactivation(&mut self, detached: Vec<SoundPlaybackId>) {
        for playback in detached {
            if let Some(active) = self.playbacks.remove(&playback) {
                self.finished_playbacks.push(playback_finished(
                    playback,
                    active,
                    SoundPlaybackFinishReason::Stopped,
                ));
                continue;
            }
            if let Some(source) = self.source_for_kira_playback(playback) {
                if let Some(voice) = self.sources.get_mut(&source) {
                    voice.kira_playback = None;
                }
            }
        }
    }

    fn source_for_kira_playback(&self, playback: SoundPlaybackId) -> Option<SoundSourceId> {
        self.sources
            .iter()
            .find_map(|(source, voice)| (voice.kira_playback == Some(playback)).then_some(*source))
    }
}

fn playback_finished(
    playback: SoundPlaybackId,
    active: ActivePlayback,
    reason: SoundPlaybackFinishReason,
) -> SoundPlaybackFinished {
    SoundPlaybackFinished {
        playback,
        clip: active.clip,
        reason,
        completion_action: active.completion_action,
        output_track: active.output_track,
    }
}

fn source_finished(
    source: SoundSourceId,
    voice: super::SourceVoice,
    reason: SoundSourceFinishReason,
) -> SoundSourceFinished {
    let descriptor = voice.descriptor;
    let input = descriptor.input;
    let clip = match input {
        SoundSourceInput::Clip(clip) => Some(clip),
        SoundSourceInput::External(_)
        | SoundSourceInput::SynthParameter { .. }
        | SoundSourceInput::Silence => None,
    };
    SoundSourceFinished {
        source,
        input,
        clip,
        reason,
        completion_action: descriptor.completion_action,
        output_track: descriptor.output_track,
    }
}
