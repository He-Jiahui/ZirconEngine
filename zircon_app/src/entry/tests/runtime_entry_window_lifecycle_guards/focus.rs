use super::super::source_assertions::assert_source_order;
use super::sources::{runtime_window_events_source, runtime_window_lifecycle_source};

#[test]
fn runtime_entry_translates_focus_changes_to_lifecycle_events() {
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_lifecycle_source = runtime_window_lifecycle_source();

    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::Focused(focused)",
            "self.handle_window_focus_changed(event_loop, focused);",
        ],
        "runtime entry should delegate focus lifecycle forwarding to the window lifecycle module",
    );
    assert_source_order(
        runtime_window_lifecycle_source.as_str(),
        &[
            "fn handle_window_focus_changed",
            "let state = if focused",
            "ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1",
            "ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1",
            "ZrRuntimeEventV1::lifecycle",
            "self.session.handle_event(event).is_err()",
        ],
        "runtime entry should translate focus changes into runtime foreground/background lifecycle events",
    );
}
