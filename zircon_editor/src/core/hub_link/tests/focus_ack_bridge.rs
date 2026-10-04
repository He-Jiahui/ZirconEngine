use zircon_runtime_interface::hub_protocol::{HubEditorFocusSignalV1, HubSessionToken};

use super::HubFocusAcknowledgementBridge;

#[test]
fn bridge_preserves_multiple_requests_until_the_native_focus_owner_acknowledges_them() {
    let bridge = HubFocusAcknowledgementBridge::new("E:/Projects/My Game");
    let first = HubEditorFocusSignalV1::new(HubSessionToken::new(), "913-42", 1, 1, u64::MAX)
        .expect("first request");
    let second = HubEditorFocusSignalV1::new(HubSessionToken::new(), "913-42", 1, 2, u64::MAX)
        .expect("second request");

    assert!(bridge.enqueue(first).expect("queue first request"));
    assert!(bridge.enqueue(second).expect("queue second request"));

    let pending = bridge
        .pending
        .lock()
        .expect("test queue lock must be available");
    assert_eq!(pending.len(), 2);
    assert_eq!(pending[0].sequence, 1);
    assert_eq!(pending[1].sequence, 2);
}

#[test]
fn retired_bridge_refuses_to_acknowledge_the_previous_session_generation() {
    let bridge = HubFocusAcknowledgementBridge::new("E:/Projects/My Game");

    bridge.retire().expect("an empty bridge can retire");

    assert!(!bridge.accepting_for_test());
}
