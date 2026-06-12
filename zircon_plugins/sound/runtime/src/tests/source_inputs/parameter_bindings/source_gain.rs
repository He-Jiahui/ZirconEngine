use super::super::super::*;

#[test]
fn audio_source_gain_binding_follows_synth_parameter_changes() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/bound-source.wav", &[1.0, 1.0]));
    let gain_parameter = SoundParameterId::new("synth.source_gain");
    sound.set_parameter(gain_parameter.clone(), 0.25).unwrap();
    let mut source = SoundSourceDescriptor::clip(clip);
    source.parameter_bindings.push(SoundSourceParameterBinding {
        source_parameter: SoundParameterId::new("gain"),
        synth_parameter: gain_parameter.clone(),
    });
    sound.create_source(source).unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.25, 0.25]);
    sound.set_parameter(gain_parameter, 0.5).unwrap();
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.5]);
}
