use super::super::super::super::*;
use super::super::super::support::test_hrtf_profile;

#[test]
fn loaded_hrtf_profile_applies_deterministic_kernels() {
    let sound = DefaultSoundManager::default();
    sound
        .load_hrtf_profile(test_hrtf_profile("loaded"))
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/hrtf-loaded.wav", &[1.0, 0.0]));
    let mut listener = test_listener();
    listener.hrtf_profile = Some("loaded".to_string());
    sound.update_listener(listener).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(2).unwrap();
    assert_sample_near(mix.samples[0], 0.0);
    assert_sample_near(mix.samples[1], 1.0);
    assert_sample_near(mix.samples[2], 0.5);
    assert_sample_near(mix.samples[3], 0.0);
}
