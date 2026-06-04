use super::super::super::*;
use super::super::support::test_hrtf_profile;

#[test]
fn loaded_hrtf_profile_state_survives_parameter_driven_playing() {
    let sound = DefaultSoundManager::default();
    sound
        .load_hrtf_profile(test_hrtf_profile("parameter-loaded"))
        .unwrap();
    let playing_parameter = SoundParameterId::new("synth.hrtf_playing");
    sound.set_parameter(playing_parameter.clone(), 1.0).unwrap();
    let clip = sound.insert_clip_for_test(test_clip(
        "res://sound/hrtf-parameter-playing.wav",
        &[1.0, 0.0],
    ));
    let mut listener = test_listener();
    listener.hrtf_profile = Some("parameter-loaded".to_string());
    sound.update_listener(listener).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.playing = false;
    source.parameter_bindings.push(SoundSourceParameterBinding {
        source_parameter: SoundParameterId::new("playing"),
        synth_parameter: playing_parameter,
    });
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.0, 1.0]);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.0]);
}
