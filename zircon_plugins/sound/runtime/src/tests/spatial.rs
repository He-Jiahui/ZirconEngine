use super::*;

#[test]
fn hrtf_profile_applies_ear_delay_for_lateral_source() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/hrtf-preview.wav", &[1.0]));
    let mut listener = test_listener();
    listener.hrtf_profile = Some("preview".to_string());
    listener.left_ear_offset = [-0.08, 0.0, 0.0];
    listener.right_ear_offset = [0.08, 0.0, 0.0];
    sound.update_listener(listener).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [0.5, 0.0, 1.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(16).unwrap();
    assert_sample_near(mix.samples[0], 0.0);
    assert!(mix.samples[1] > 0.9);
    let delayed_left_peak = mix
        .samples
        .chunks_exact(2)
        .skip(1)
        .map(|frame| frame[0].abs())
        .fold(0.0_f32, f32::max);
    assert!(delayed_left_peak > 0.25);
}
