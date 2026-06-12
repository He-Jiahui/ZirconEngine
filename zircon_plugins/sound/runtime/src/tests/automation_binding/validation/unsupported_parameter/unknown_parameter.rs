use super::super::super::super::*;

#[test]
fn automation_binding_reports_unsupported_parameters() {
    let sound = DefaultSoundManager::default();
    let unsupported_binding = SoundAutomationBindingId::new(21);

    sound
        .bind_automation(SoundAutomationBinding {
            id: unsupported_binding,
            timeline_track_path: "Root/Master:sound.master.unknown".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("unknown_parameter"),
        })
        .unwrap();

    assert!(sound
        .apply_automation_value(unsupported_binding, 1.0)
        .unwrap_err()
        .to_string()
        .contains("unsupported sound automation parameter"));
}
