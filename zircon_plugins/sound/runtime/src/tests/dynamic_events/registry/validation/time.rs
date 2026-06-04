use super::support::{marker_invocation, register_marker_event};

use super::super::super::*;

#[test]
fn dynamic_event_registry_rejects_non_finite_invocation_time() {
    let sound = DefaultSoundManager::default();
    register_marker_event(&sound);
    let mut invocation = marker_invocation();
    invocation.time_seconds = f32::NAN;

    assert!(sound
        .submit_dynamic_event(invocation)
        .unwrap_err()
        .to_string()
        .contains("time"));
}
