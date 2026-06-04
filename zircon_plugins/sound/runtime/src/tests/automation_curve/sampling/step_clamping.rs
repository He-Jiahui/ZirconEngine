use super::super::super::*;

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
