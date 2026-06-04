use super::support::{marker_invocation, register_marker_event};

use super::super::super::*;

#[test]
fn dynamic_event_registry_rejects_mismatched_payload_schema() {
    let sound = DefaultSoundManager::default();
    register_marker_event(&sound);
    let mut invocation = marker_invocation();
    invocation.payload_schema = "sound.dynamic.other.v1".to_string();

    assert!(sound
        .submit_dynamic_event(invocation)
        .unwrap_err()
        .to_string()
        .contains("expects payload schema"));
}
