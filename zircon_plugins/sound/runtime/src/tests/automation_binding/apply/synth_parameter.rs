use super::super::super::*;

#[test]
fn automation_binding_applies_values_to_synth_parameter_targets() {
    let sound = DefaultSoundManager::default();
    let synth_parameter = SoundParameterId::new("synth.amp");
    let source = SoundSourceDescriptor {
        input: SoundSourceInput::SynthParameter {
            parameter: synth_parameter.clone(),
            default_value: 0.0,
        },
        ..SoundSourceDescriptor::clip(SoundClipId::new(999))
    };
    sound.create_source(source).unwrap();

    let synth_binding = SoundAutomationBindingId::new(10);
    sound
        .bind_automation(SoundAutomationBinding {
            id: synth_binding,
            timeline_track_path: "Root/Synth:sound.synth.amp".to_string(),
            target: SoundAutomationTarget::SynthParameter(synth_parameter.clone()),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();
    sound.apply_automation_value(synth_binding, 0.4).unwrap();

    assert_sample_near(sound.parameter_value(&synth_parameter).unwrap(), 0.4);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.4, 0.4]);
}

#[test]
fn automation_binding_rejects_non_finite_synth_parameter_values() {
    let sound = DefaultSoundManager::default();
    let synth_parameter = SoundParameterId::new("synth.invalid");
    let binding = SoundAutomationBindingId::new(11);

    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Root/Synth:sound.synth.invalid".to_string(),
            target: SoundAutomationTarget::SynthParameter(synth_parameter),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();

    assert!(sound
        .apply_automation_value(binding, f32::NAN)
        .unwrap_err()
        .to_string()
        .contains("finite"));
}
