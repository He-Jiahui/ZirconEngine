use kira::backend::mock::MockBackend;
use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundPlaybackCompletionAction, SoundPlaybackFinishReason, SoundPlaybackId,
    SoundSourceDescriptor, SoundSourceFinishReason, SoundSourceId, SoundTrackId,
};

use crate::engine::{ActivePlayback, SoundEngineState, SourceVoice};
use crate::kira_bridge::KiraEngine;
use crate::SoundConfig;

use super::support::{mock_settings, silent_stereo_clip};

#[test]
fn deactivate_returns_detached_playback_ids_without_ghost_handles() {
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine.activate(mock_settings(48_000)).unwrap();
    let playback = SoundPlaybackId::new(41);
    engine
        .play(playback, SoundTrackId::master(), silent_stereo_clip(48_000))
        .unwrap();

    let detached = engine.deactivate();

    assert_eq!(detached, vec![playback]);
    assert!(!engine.is_active());
    assert!(!engine.contains_playback(playback));
}

#[test]
fn mock_backend_natural_completion_is_drained_once() {
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine.activate(mock_settings(48_000)).unwrap();
    let playback = SoundPlaybackId::new(42);
    engine
        .play(playback, SoundTrackId::master(), silent_stereo_clip(48_000))
        .unwrap();
    engine
        .with_backend_mut(|backend| {
            backend.on_start_processing();
            backend.process();
            backend.on_start_processing();
            backend.process();
        })
        .unwrap();

    assert_eq!(engine.drain_finished_playbacks(), vec![playback]);
    assert!(engine.drain_finished_playbacks().is_empty());
    assert!(!engine.contains_playback(playback));
}

#[test]
fn completed_kira_ids_emit_typed_playback_and_source_events() {
    let mut state = SoundEngineState::new(&SoundConfig::default());
    let playback = SoundPlaybackId::new(51);
    let source_playback = SoundPlaybackId::new(52);
    let source = SoundSourceId::new(9);
    state
        .playbacks
        .insert(playback, active_playback(SoundClipId::new(3)));
    let mut voice = SourceVoice::new(SoundSourceDescriptor::clip(SoundClipId::new(4)));
    voice.kira_playback = Some(source_playback);
    state.sources.insert(source, voice);

    state.reconcile_kira_completions(vec![playback, source_playback]);

    assert_eq!(state.finished_playbacks.len(), 1);
    assert_eq!(
        state.finished_playbacks[0].reason,
        SoundPlaybackFinishReason::Completed
    );
    assert_eq!(state.finished_sources.len(), 1);
    assert_eq!(state.finished_sources[0].source, source);
    assert_eq!(
        state.finished_sources[0].reason,
        SoundSourceFinishReason::Completed
    );
    assert!(!state.playbacks.contains_key(&playback));
    assert!(!state.sources.contains_key(&source));
}

#[test]
fn deactivated_kira_ids_finish_playbacks_and_preserve_sources_for_rebuild() {
    let mut state = SoundEngineState::new(&SoundConfig::default());
    let playback = SoundPlaybackId::new(61);
    let source_playback = SoundPlaybackId::new(62);
    let source = SoundSourceId::new(10);
    state
        .playbacks
        .insert(playback, active_playback(SoundClipId::new(5)));
    let mut voice = SourceVoice::new(SoundSourceDescriptor::clip(SoundClipId::new(6)));
    voice.kira_playback = Some(source_playback);
    state.sources.insert(source, voice);

    state.reconcile_kira_deactivation(vec![playback, source_playback]);

    assert_eq!(state.finished_playbacks.len(), 1);
    assert_eq!(
        state.finished_playbacks[0].reason,
        SoundPlaybackFinishReason::Stopped
    );
    assert!(!state.playbacks.contains_key(&playback));
    assert_eq!(state.sources[&source].kira_playback, None);
    assert!(state.sources[&source].descriptor.playing);
    assert!(state.finished_sources.is_empty());
}

fn active_playback(clip: SoundClipId) -> ActivePlayback {
    ActivePlayback {
        clip,
        cursor_frame: 0,
        cursor_position: 0.0,
        gain: 1.0,
        speed: 1.0,
        looped: false,
        completion_action: SoundPlaybackCompletionAction::None,
        paused: false,
        muted: false,
        range_start_frame: 0,
        range_end_frame: None,
        output_track: SoundTrackId::master(),
        pan: 0.0,
    }
}
