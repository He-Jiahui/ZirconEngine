use super::super::super::*;

use super::support::register_marker_event;

#[test]
fn dynamic_event_registry_reports_unknown_event_after_unregister() {
    let sound = DefaultSoundManager::default();
    register_marker_event(&sound);

    sound
        .unregister_dynamic_event("sound.dynamic.marker")
        .unwrap();

    assert!(matches!(
        sound
            .unregister_dynamic_event("sound.dynamic.marker")
            .unwrap_err(),
        SoundError::UnknownDynamicEvent { .. }
    ));
}
