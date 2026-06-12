use super::super::super::*;

#[test]
fn clip_input_resamples_to_mixer_rate() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip_with_rate(
        "res://sound/resampled.wav",
        24_000,
        &[0.25, 0.5],
    ));
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    assert_samples_near(
        &sound.render_mix(4).unwrap().samples,
        &[0.25, 0.25, 0.375, 0.375, 0.5, 0.5, 0.5, 0.5],
    );
}
