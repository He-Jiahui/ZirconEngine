use zircon_runtime_interface::ZrRuntimeViewportHandle;

use super::runtime_event_dispatch_failure;

#[test]
fn runtime_event_dispatch_failure_is_actionable() {
    let failure = runtime_event_dispatch_failure(
        17,
        ZrRuntimeViewportHandle::new(3),
        "runtime rejected event",
    );

    assert_eq!(
        failure.to_string(),
        "runtime startup diagnostic: component=runtime_event_dispatch requested=event_kind=17 viewport=ZrRuntimeViewportHandle(3) cause=runtime event dispatch failed: runtime rejected event recovery=verify the runtime library ABI and event handler, then restart zircon_runtime"
    );
}
