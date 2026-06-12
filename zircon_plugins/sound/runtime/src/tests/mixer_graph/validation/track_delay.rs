use super::super::super::*;

#[test]
fn mixer_graph_rejects_unbounded_track_delay_before_render() {
    let sound = DefaultSoundManager::default();
    let mut master = SoundTrackDescriptor::master();
    master.controls.delay_frames = 1_000_000;

    assert!(sound
        .add_or_update_track(master)
        .unwrap_err()
        .to_string()
        .contains("history budget"));
}
