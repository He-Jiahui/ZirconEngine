use super::super::source_assertions::assert_source_order;
use super::sources::{runtime_window_events_source, runtime_window_lifecycle_source};

#[test]
fn runtime_entry_notifies_runtime_before_applying_close_policy() {
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_lifecycle_source = runtime_window_lifecycle_source();

    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::CloseRequested",
            "self.handle_window_close_requested(event_loop);",
        ],
        "runtime entry should delegate close-request policy handling to the window lifecycle module",
    );
    assert_source_order(
        runtime_window_lifecycle_source.as_str(),
        &[
            "fn handle_window_close_requested",
            "ZrRuntimeEventV1::window_close_requested",
            "self.session.handle_event(event).is_err()",
            "event_loop.exit();",
            "return;",
            "self.window_lifecycle_policy.should_close_on_request()",
            "self.close_primary_window_after_request();",
            ".should_exit_after_primary_close()",
            "event_loop.exit();",
        ],
        "runtime entry should notify the runtime about close requests before applying the configurable close policy",
    );
}
