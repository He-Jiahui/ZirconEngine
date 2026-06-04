use super::super::*;

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
