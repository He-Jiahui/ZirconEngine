use super::super::super::*;

#[test]
fn automation_curve_sample_updates_bound_synth_parameter() {
    let sound = DefaultSoundManager::default();
    let parameter = SoundParameterId::new("synth.curve_amp");
    let binding = SoundAutomationBindingId::new(101);
    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Root/Synth:sound.synth.curve_amp".to_string(),
            target: SoundAutomationTarget::SynthParameter(parameter.clone()),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();
    sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::SynthParameter {
                parameter: parameter.clone(),
                default_value: 0.0,
            },
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    let curve = SoundAutomationCurve::from_keyframes([
        SoundAutomationKeyframe::linear(0.0, 0.2),
        SoundAutomationKeyframe::smooth_step(1.0, 0.8),
        SoundAutomationKeyframe::linear(2.0, 0.4),
    ]);

    let value = sound
        .apply_automation_curve_sample(binding, &curve, 0.5)
        .unwrap();

    assert_sample_near(value, 0.5);
    assert_sample_near(sound.parameter_value(&parameter).unwrap(), 0.5);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.5]);
}
