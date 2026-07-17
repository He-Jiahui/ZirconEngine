use kira::backend::mock::MockBackend;
use zircon_runtime::core::framework::sound::{
    SoundPlaybackId, SoundSourceId, SoundSourceManager, SoundTrackId,
};

use crate::kira_bridge::KiraEngine;

use super::support::{
    graph_with_music_track, inactive_source_fixture, mock_settings, silent_stereo_clip,
};

#[test]
fn mock_backend_reconfigure_clears_previous_graph_state() {
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine.activate(mock_settings(48_000)).unwrap();
    engine.sync_graph(&graph_with_music_track(48_000)).unwrap();
    assert!(engine.contains_track(SoundTrackId::new(2)));

    engine.activate(mock_settings(24_000)).unwrap();

    assert!(engine.is_active());
    assert_eq!(engine.track_count(), 1);
    assert!(!engine.contains_track(SoundTrackId::new(2)));
}

#[test]
fn mock_backend_reconfigure_accepts_new_format_graph() {
    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine.activate(mock_settings(48_000)).unwrap();
    engine.sync_graph(&graph_with_music_track(48_000)).unwrap();
    engine.activate(mock_settings(24_000)).unwrap();
    engine.sync_graph(&graph_with_music_track(24_000)).unwrap();
    let playback = SoundPlaybackId::new(9);

    engine
        .play(playback, SoundTrackId::new(2), silent_stereo_clip(24_000))
        .unwrap();

    assert!(engine.contains_track(SoundTrackId::new(2)));
    assert!(engine.contains_playback(playback));
}

#[test]
fn inactive_source_controls_validate_without_synthetic_output_backend() {
    let (sound, source) = inactive_source_fixture();

    assert!(sound.seek_source_seconds(source, -0.1).is_err());
    assert!(sound.set_source_gain(source, f32::NAN).is_err());
    assert!(sound.set_source_speed(source, 0.0).is_err());
    assert!(sound.unmute_source(SoundSourceId::new(999_999)).is_err());
}
