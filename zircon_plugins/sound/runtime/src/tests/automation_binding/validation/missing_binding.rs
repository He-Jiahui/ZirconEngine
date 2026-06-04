use super::super::super::*;

#[test]
fn automation_binding_reports_missing_binding_application() {
    let sound = DefaultSoundManager::default();

    let missing = sound
        .apply_automation_value(SoundAutomationBindingId::new(999), 0.1)
        .unwrap_err();

    assert!(matches!(
        missing,
        SoundError::UnknownAutomationBinding { .. }
    ));
}
