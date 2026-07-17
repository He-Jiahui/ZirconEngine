use kira::{
    backend::mock::MockBackend,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    AudioManagerSettings, Frame,
};
use std::sync::Arc;
use zircon_runtime::core::framework::sound::{
    SoundMixerGraph, SoundPlaybackId, SoundTrackDescriptor, SoundTrackId, SoundTrackSend,
};

use crate::kira_bridge::KiraEngine;

use super::support::mock_settings;

#[test]
fn engine_starts_and_stops_with_mock_backend() {
    let mut engine = KiraEngine::<MockBackend>::inactive();
    assert!(!engine.is_active());

    engine.activate(mock_settings(48_000)).unwrap();
    assert!(engine.is_active());

    assert!(engine.deactivate().is_empty());
    assert!(!engine.is_active());
}

#[test]
fn configured_track_and_voice_limits_are_enforced_before_backend_exhaustion() {
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine
        .activate_with_limits(mock_settings(48_000), 2, 1)
        .unwrap();
    let mut graph = SoundMixerGraph::default_stereo(48_000);
    graph
        .tracks
        .push(SoundTrackDescriptor::child(SoundTrackId::new(2), "Music"));
    engine.sync_graph(&graph).unwrap();

    let mut oversized = graph.clone();
    oversized
        .tracks
        .push(SoundTrackDescriptor::child(SoundTrackId::new(3), "SFX"));
    assert!(engine.sync_graph(&oversized).is_err());
    assert!(engine.contains_track(SoundTrackId::new(2)));
    assert!(!engine.contains_track(SoundTrackId::new(3)));

    let clip = || StaticSoundData {
        sample_rate: 48_000,
        frames: Arc::from([Frame::ZERO; 8]),
        settings: StaticSoundSettings::default(),
        slice: None,
    };
    engine
        .play(SoundPlaybackId::new(1), SoundTrackId::new(2), clip())
        .unwrap();
    assert!(engine
        .play(SoundPlaybackId::new(2), SoundTrackId::new(2), clip())
        .is_err());
}

#[test]
fn backend_allocation_failure_preserves_the_previously_installed_graph() {
    let mut settings = mock_settings(48_000);
    settings.capacities.sub_track_capacity = 1;
    settings.capacities.send_track_capacity = 0;
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine.activate(settings).unwrap();
    engine.set_logical_capacities_for_test(2, 8);

    let mut before = SoundMixerGraph::default_stereo(48_000);
    before
        .tracks
        .push(SoundTrackDescriptor::child(SoundTrackId::new(2), "Music"));
    engine.sync_graph(&before).unwrap();

    let mut after = before.clone();
    after.tracks[1].sends.push(SoundTrackSend {
        target: SoundTrackId::master(),
        gain: 0.5,
        pre_effects: false,
    });
    assert!(engine.sync_graph(&after).is_err());
    assert_eq!(engine.installed_graph_for_test(), Some(&before));
    assert!(engine.contains_track(SoundTrackId::new(2)));
}
