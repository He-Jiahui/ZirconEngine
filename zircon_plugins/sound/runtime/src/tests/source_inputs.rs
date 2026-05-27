use super::*;

#[test]
fn external_audio_source_block_routes_other_component_audio() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("particles.wind-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 1,
                samples: vec![0.25, 0.5],
            },
        )
        .unwrap();
    let source_id = sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(handle.clone()),
            gain: 0.5,
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    let first_mix = sound.render_mix(2).unwrap();
    let second_mix = sound.render_mix(1).unwrap();

    assert_eq!(source_id.raw(), 1);
    assert_samples_near(&first_mix.samples, &[0.125, 0.125, 0.25, 0.25]);
    assert_samples_near(&second_mix.samples, &[0.0, 0.0]);
    assert!(sound.source_empty(source_id).unwrap());
    let finished = sound.drain_finished_sources().unwrap();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].source, source_id);
    assert_eq!(finished[0].input, SoundSourceInput::External(handle));
    assert_eq!(finished[0].clip, None);
    assert_eq!(finished[0].reason, SoundSourceFinishReason::Completed);
}

#[test]
fn external_audio_source_lifecycle_reports_invalid_and_missing_blocks() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("navigation.surface-noise");
    let empty_handle = ExternalAudioSourceHandle::new(" ");

    assert!(sound
        .submit_external_source_block(
            empty_handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 1,
                samples: vec![0.0],
            },
        )
        .unwrap_err()
        .to_string()
        .contains("external source handle"));
    assert!(sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 0,
                samples: vec![0.0],
            },
        )
        .unwrap_err()
        .to_string()
        .contains("channel count"));
    assert!(sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 1,
                samples: vec![f32::NAN],
            },
        )
        .unwrap_err()
        .to_string()
        .contains("finite"));
    assert!(matches!(
        sound.clear_external_source(&handle).unwrap_err(),
        SoundError::UnknownExternalSource { .. }
    ));

    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 1,
                samples: vec![0.75],
            },
        )
        .unwrap();
    sound.clear_external_source(&handle).unwrap();
    assert!(sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(empty_handle),
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap_err()
        .to_string()
        .contains("external source handle"));
    sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(handle),
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.0, 0.0]);
}

#[test]
fn clip_and_external_inputs_resample_to_mixer_rate() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip_with_rate(
        "res://sound/resampled.wav",
        24_000,
        &[0.25, 0.5],
    ));
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    assert_samples_near(
        &sound.render_mix(4).unwrap().samples,
        &[0.25, 0.25, 0.375, 0.375, 0.5, 0.5, 0.5, 0.5],
    );

    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("synth.low-rate");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 24_000,
                channel_count: 1,
                samples: vec![0.5, 1.0],
            },
        )
        .unwrap();
    sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(handle),
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    assert_samples_near(
        &sound.render_mix(4).unwrap().samples,
        &[0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0, 1.0],
    );
}

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
