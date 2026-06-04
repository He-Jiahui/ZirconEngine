use super::super::super::*;

#[test]
fn playback_settings_reject_non_finite_initial_mix_parameters() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/playback-invalid.wav", &[0.5]));

    assert!(sound
        .play_clip(clip, SoundPlaybackSettings::ONCE.with_gain(f32::NAN),)
        .is_err());
    assert!(sound
        .play_clip(clip, SoundPlaybackSettings::ONCE.with_gain(f32::INFINITY),)
        .is_err());
    assert!(sound
        .play_clip(clip, SoundPlaybackSettings::ONCE.with_pan(f32::NAN),)
        .is_err());
    assert!(sound
        .play_clip(
            clip,
            SoundPlaybackSettings::ONCE.with_pan(f32::NEG_INFINITY),
        )
        .is_err());
}
