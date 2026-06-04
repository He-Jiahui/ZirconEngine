use super::super::super::*;

#[test]
fn source_speed_and_muted_controls_match_bevy_playback_settings() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip(
        "res://sound/source-speed-muted.wav",
        &[0.25, 0.5, 0.75],
    ));

    let mut source = SoundSourceDescriptor::clip(clip);
    source.speed = 2.0;
    source.muted = true;
    let source_id = sound.create_source(source.clone()).unwrap();

    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);

    source.id = Some(source_id);
    source.muted = false;
    sound.update_source(source).unwrap();
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.75, 0.75]);

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.speed = 0.0;
    assert!(sound.create_source(invalid).is_err());

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.speed = f32::NAN;
    assert!(sound.create_source(invalid).is_err());
}
