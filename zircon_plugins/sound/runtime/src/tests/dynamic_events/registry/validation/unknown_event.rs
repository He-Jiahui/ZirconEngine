use super::super::super::*;

use super::support::marker_invocation;

#[test]
fn dynamic_event_registry_rejects_unknown_invocation_event() {
    let sound = DefaultSoundManager::default();
    let mut invocation = marker_invocation();
    invocation.event_id = "sound.dynamic.missing".to_string();

    assert!(matches!(
        sound.submit_dynamic_event(invocation).unwrap_err(),
        SoundError::UnknownDynamicEvent { .. }
    ));
}
