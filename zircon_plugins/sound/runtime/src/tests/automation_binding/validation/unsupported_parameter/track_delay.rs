use super::super::super::super::*;

#[test]
fn automation_binding_rejects_unbounded_track_delay_parameters() {
    let sound = DefaultSoundManager::default();
    let binding = SoundAutomationBindingId::new(22);

    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Root/Master:sound.master.delay_frames".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("delay_frames"),
        })
        .unwrap();

    assert!(sound
        .apply_automation_value(binding, 1_000_000.0)
        .unwrap_err()
        .to_string()
        .contains("history budget"));

    assert!(sound
        .apply_automation_value(binding, f32::NAN)
        .unwrap_err()
        .to_string()
        .contains("non-negative frame count"));
}
