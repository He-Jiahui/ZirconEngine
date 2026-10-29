use zircon_runtime::core::framework::sound::{
    SoundPlaybackId, SoundSourceFinishReason, SoundSourceManager,
};

use super::*;

fn clip() -> SoundAsset {
    SoundAsset {
        uri: AssetUri::parse("res://sound/scene-fixture.wav").unwrap(),
        sample_rate_hz: 8,
        channel_count: 1,
        channel_layout: AudioChannelLayout::mono(),
        samples: vec![0.25; 8],
    }
}

#[test]
fn scene_source_natural_completion_keeps_tombstone_without_consuming_finished_events() {
    let manager = DefaultSoundManager::default();
    let clip = manager.insert_clip_for_test(clip());
    let descriptor = SoundSourceDescriptor::clip(clip);
    let source = manager.sync_scene_source(None, descriptor.clone()).unwrap();
    // Feed the actual completion reconciler as a callback would, without
    // requiring a real device or a second production backend.
    {
        let mut state = lock_recover(&manager.state);
        let playback = SoundPlaybackId::new(1);
        state.sources.get_mut(&source.id).unwrap().kira_playback = Some(playback);
        state.reconcile_kira_completions(vec![playback]);
    }
    let next = manager
        .sync_scene_source(Some(&source), descriptor)
        .unwrap();
    assert_eq!(next.id, source.id);
    assert!(lock_recover(&manager.state).sources.is_empty());
    let finished = manager.drain_finished_sources().unwrap();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].reason, SoundSourceFinishReason::Completed);
    manager.release_scene_source(next.id).unwrap();
    assert!(lock_recover(&manager.scene_sources).is_empty());
}

#[test]
fn scene_source_scalar_controls_preserve_cursor() {
    let manager = DefaultSoundManager::default();
    let clip = manager.insert_clip_for_test(clip());
    let mut descriptor = SoundSourceDescriptor::clip(clip);
    let source = manager.sync_scene_source(None, descriptor.clone()).unwrap();
    {
        let mut state = lock_recover(&manager.state);
        let voice = state.sources.get_mut(&source.id).unwrap();
        voice.cursor_frame = 4;
        voice.cursor_position = 4.0;
    }
    descriptor.gain = 0.5;
    descriptor.muted = true;
    descriptor.speed = 1.5;
    let next = manager
        .sync_scene_source(Some(&source), descriptor)
        .unwrap();
    assert_eq!(next.id, source.id);
    assert_eq!(manager.source_status(next.id).unwrap().cursor_frame, 4);
    manager.release_scene_source(next.id).unwrap();
    assert!(lock_recover(&manager.scene_sources).is_empty());
}

#[test]
fn scene_source_failed_release_retains_session_ownership_for_retry() {
    let manager = DefaultSoundManager::default();
    let clip = manager.insert_clip_for_test(clip());
    let source = manager
        .sync_scene_source(None, SoundSourceDescriptor::clip(clip))
        .unwrap();
    lock_recover(&manager.state)
        .kira
        .mark_provider_retiring_for_test();
    assert!(manager.release_scene_source(source.id).is_err());
    assert!(lock_recover(&manager.scene_sources).contains(&source.id));
    assert!(manager.source_status(source.id).is_ok());
    lock_recover(&manager.state).kira.deactivate();
    manager.release_scene_sources().unwrap();
    assert!(lock_recover(&manager.scene_sources).is_empty());
    assert!(manager.source_status(source.id).is_err());
}
use zircon_runtime::asset::{AssetUri, SoundAsset};
use zircon_runtime::core::framework::audio::AudioChannelLayout;
