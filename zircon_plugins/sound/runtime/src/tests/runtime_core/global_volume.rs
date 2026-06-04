use super::super::*;

#[test]
fn global_volume_gain_scales_final_mix_and_rejects_invalid_values() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/global-volume.wav", &[1.0]));
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    assert_eq!(sound.global_volume_gain().unwrap(), 1.0);
    sound.set_global_volume_gain(0.25).unwrap();
    let mix = sound.render_mix(1).unwrap();

    assert_eq!(sound.global_volume_gain().unwrap(), 0.25);
    assert_eq!(mix.samples, vec![0.25, 0.25]);
    assert!(sound.set_global_volume_gain(f32::NAN).is_err());
    assert!(sound.set_global_volume_gain(-0.1).is_err());
}
