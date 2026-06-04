use super::super::super::*;

#[test]
fn automation_binding_rejects_invalid_timeline_track_paths() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .bind_automation(SoundAutomationBinding {
            id: SoundAutomationBindingId::new(20),
            timeline_track_path: " ".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("gain"),
        })
        .unwrap_err()
        .to_string()
        .contains("timeline track path"));
    assert!(sound
        .bind_automation(SoundAutomationBinding {
            id: SoundAutomationBindingId::new(23),
            timeline_track_path: "Root/Master:gain".to_string(),
            target: SoundAutomationTarget::Track(SoundTrackId::master()),
            parameter: SoundParameterId::new("gain"),
        })
        .unwrap_err()
        .to_string()
        .contains("AnimationTrackPath-style"));
}
