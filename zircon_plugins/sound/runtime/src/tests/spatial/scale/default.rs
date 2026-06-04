use super::super::super::*;

#[test]
fn default_spatial_scale_controls_listener_source_distance() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/spatial-scale.wav", &[1.0]));
    sound.update_listener(test_listener()).unwrap();
    assert_eq!(sound.default_spatial_scale().unwrap(), 1.0);

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [2.0, 0.0, 0.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        min_distance: 1.0,
        max_distance: 5.0,
        attenuation: SoundAttenuationMode::Linear,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(1).unwrap();
    assert_sample_near(mix.samples[0], 0.0);
    assert_sample_near(mix.samples[1], 0.75);

    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/spatial-scale-half.wav", &[1.0]));
    sound.update_listener(test_listener()).unwrap();
    sound.set_default_spatial_scale(0.5).unwrap();
    assert_eq!(sound.default_spatial_scale().unwrap(), 0.5);
    assert!(sound.set_default_spatial_scale(f32::NAN).is_err());
    assert!(sound.set_default_spatial_scale(-0.1).is_err());

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [2.0, 0.0, 0.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        min_distance: 1.0,
        max_distance: 5.0,
        attenuation: SoundAttenuationMode::Linear,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(1).unwrap();
    assert_sample_near(mix.samples[0], 0.0);
    assert_sample_near(mix.samples[1], 1.0);
}
