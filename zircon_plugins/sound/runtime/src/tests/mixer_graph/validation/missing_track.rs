use super::super::super::*;

#[test]
fn mixer_graph_rejects_playback_to_missing_output_track() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/cycle.wav", &[1.0]));

    let missing = sound
        .play_clip(
            clip,
            SoundPlaybackSettings {
                output_track: SoundTrackId::new(99),
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap_err();
    assert!(missing.to_string().contains("unknown track"));
}
