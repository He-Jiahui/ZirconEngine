use super::super::super::*;

#[test]
fn source_spatial_scale_overrides_default_spatial_scale() {
    let sound = DefaultSoundManager::default();
    let clip =
        sound.insert_clip_for_test(test_clip("res://sound/source-spatial-scale.wav", &[1.0]));
    sound.update_listener(test_listener()).unwrap();
    sound.set_default_spatial_scale(0.5).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [2.0, 0.0, 0.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        spatial_scale: Some(1.0),
        min_distance: 1.0,
        max_distance: 5.0,
        attenuation: SoundAttenuationMode::Linear,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(1).unwrap();
    assert_sample_near(mix.samples[0], 0.0);
    assert_sample_near(mix.samples[1], 0.75);

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.spatial.spatial_scale = Some(f32::NAN);
    assert!(sound.create_source(invalid).is_err());

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.spatial.spatial_scale = Some(-0.1);
    assert!(sound.create_source(invalid).is_err());
}
