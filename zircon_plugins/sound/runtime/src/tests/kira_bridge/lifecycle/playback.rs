use kira::backend::mock::MockBackend;
use zircon_runtime::core::framework::sound::{SoundPlaybackId, SoundTrackId};

use crate::kira_bridge::KiraEngine;

use super::support::{graph_with_music_track, mock_settings, silent_stereo_clip};

#[test]
fn playback_lifecycle_round_trips_through_kira() {
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine.activate(mock_settings(48_000)).unwrap();
    engine.sync_graph(&graph_with_music_track(48_000)).unwrap();
    let playback = SoundPlaybackId::new(7);

    engine
        .play(playback, SoundTrackId::new(2), silent_stereo_clip(48_000))
        .unwrap();
    assert!(engine.contains_playback(playback));
    engine.pause(playback).unwrap();
    engine.seek_to(playback, 0.0).unwrap();
    engine.resume(playback).unwrap();
    engine.stop(playback).unwrap();
    assert!(!engine.contains_playback(playback));
}
