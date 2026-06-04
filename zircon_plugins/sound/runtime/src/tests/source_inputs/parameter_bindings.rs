use super::super::*;

#[test]
fn audio_source_parameter_bindings_follow_synth_parameters() {
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

    let mut invalid_source = SoundSourceDescriptor {
        input: SoundSourceInput::Silence,
        ..SoundSourceDescriptor::clip(SoundClipId::new(999))
    };
    invalid_source
        .parameter_bindings
        .push(SoundSourceParameterBinding {
            source_parameter: SoundParameterId::new("not_a_source_parameter"),
            synth_parameter: SoundParameterId::new("synth.invalid"),
        });
    assert!(sound
        .create_source(invalid_source)
        .unwrap_err()
        .to_string()
        .contains("unsupported source parameter binding"));
    assert!(sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::SynthParameter {
                parameter: SoundParameterId::new(" "),
                default_value: 0.0,
            },
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap_err()
        .to_string()
        .contains("synth source input"));
    assert!(sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::SynthParameter {
                parameter: SoundParameterId::new("synth.bad_default"),
                default_value: f32::NAN,
            },
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap_err()
        .to_string()
        .contains("finite default"));
}
