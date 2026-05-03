use super::*;

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

#[test]
fn automation_curve_supports_step_and_endpoint_clamping() {
    let sound = DefaultSoundManager::default();
    let parameter = SoundParameterId::new("synth.stepped");
    let binding = SoundAutomationBindingId::new(102);
    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Root/Synth:sound.synth.stepped".to_string(),
            target: SoundAutomationTarget::SynthParameter(parameter.clone()),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();
    let curve = SoundAutomationCurve::from_keyframes([
        SoundAutomationKeyframe::step(1.0, 0.25),
        SoundAutomationKeyframe::linear(2.0, 0.75),
    ]);

    assert_sample_near(
        sound
            .apply_automation_curve_sample(binding, &curve, 0.0)
            .unwrap(),
        0.25,
    );
    assert_sample_near(
        sound
            .apply_automation_curve_sample(binding, &curve, 1.5)
            .unwrap(),
        0.25,
    );
    assert_sample_near(
        sound
            .apply_automation_curve_sample(binding, &curve, 3.0)
            .unwrap(),
        0.75,
    );
}

#[test]
fn automation_curve_rejects_invalid_curve_data_cleanly() {
    let sound = DefaultSoundManager::default();
    let binding = SoundAutomationBindingId::new(103);
    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Root/Master:sound.master.gain".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("gain"),
        })
        .unwrap();

    let empty = SoundAutomationCurve::from_keyframes(Vec::<SoundAutomationKeyframe>::new());
    assert!(sound
        .apply_automation_curve_sample(binding, &empty, 0.0)
        .unwrap_err()
        .to_string()
        .contains("at least one keyframe"));

    let unsorted = SoundAutomationCurve::from_keyframes([
        SoundAutomationKeyframe::linear(1.0, 0.5),
        SoundAutomationKeyframe::linear(1.0, 0.75),
    ]);
    assert!(sound
        .apply_automation_curve_sample(binding, &unsorted, 0.0)
        .unwrap_err()
        .to_string()
        .contains("strictly increasing"));

    let non_finite =
        SoundAutomationCurve::from_keyframes([SoundAutomationKeyframe::linear(0.0, f32::NAN)]);
    assert!(sound
        .apply_automation_curve_sample(binding, &non_finite, 0.0)
        .unwrap_err()
        .to_string()
        .contains("finite"));
}
