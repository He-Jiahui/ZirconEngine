use super::super::source_assertions::assert_source_order;
use super::sources::{runtime_window_events_source, runtime_window_lifecycle_source};

#[test]
fn runtime_entry_forwards_backend_scale_factor_before_logical_scale_factor() {
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_lifecycle_source = runtime_window_lifecycle_source();

    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::ScaleFactorChanged { scale_factor, .. }",
            "self.handle_window_scale_factor_changed(event_loop, scale_factor);",
        ],
        "runtime entry should delegate scale-factor status forwarding to the window lifecycle module",
    );
    assert_source_order(
        runtime_window_lifecycle_source.as_str(),
        &[
            "fn handle_window_scale_factor_changed",
            "ZrRuntimeEventV1::window_backend_scale_factor_changed",
            "self.session.handle_event(backend_event).is_err()",
            "return;",
            "ZrRuntimeEventV1::window_scale_factor_changed",
            "self.session.handle_event(logical_event).is_err()",
        ],
        "runtime entry should forward backend scale-factor changes before logical scale-factor changes",
    );
}
