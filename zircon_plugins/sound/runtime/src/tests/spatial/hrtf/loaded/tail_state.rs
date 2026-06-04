use super::super::super::super::*;
use super::super::super::support::test_hrtf_profile;

#[test]
fn loaded_hrtf_profile_keeps_fir_tail_across_render_blocks() {
    let sound = DefaultSoundManager::default();
    sound
        .load_hrtf_profile(test_hrtf_profile("loaded-tail"))
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/hrtf-tail.wav", &[1.0, 0.0]));
    let mut listener = test_listener();
    listener.hrtf_profile = Some("loaded-tail".to_string());
    sound.update_listener(listener).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.0, 1.0]);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.0]);
}
