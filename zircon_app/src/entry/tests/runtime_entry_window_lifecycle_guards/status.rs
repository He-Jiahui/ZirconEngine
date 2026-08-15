use super::super::source_assertions::assert_source_order;
use super::sources::{runtime_window_events_source, runtime_window_lifecycle_source};

#[test]
fn runtime_entry_delegates_window_status_forwarding_to_lifecycle_module() {
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_lifecycle_source = runtime_window_lifecycle_source();

    for (event, handler_call, helper, constructor) in [
        (
            "WindowEvent::Destroyed",
            "self.handle_window_destroyed(event_loop);",
            "fn handle_window_destroyed",
            "ZrRuntimeEventV1::window_destroyed",
        ),
        (
            "WindowEvent::Moved(position)",
            "self.handle_window_moved(event_loop, position);",
            "fn handle_window_moved",
            "ZrRuntimeEventV1::window_moved",
        ),
        (
            "WindowEvent::Occluded(occluded)",
            "self.handle_window_occluded(event_loop, occluded);",
            "fn handle_window_occluded",
            "ZrRuntimeEventV1::window_occluded",
        ),
        (
            "WindowEvent::ThemeChanged(theme)",
            "self.handle_window_theme_changed(event_loop, theme);",
            "fn handle_window_theme_changed",
            "ZrRuntimeEventV1::window_theme_changed",
        ),
    ] {
        assert_source_order(
            runtime_window_events_source.as_str(),
            &[event, handler_call],
            "runtime entry should delegate window status event forwarding to the window lifecycle module",
        );
        assert_source_order(
            runtime_window_lifecycle_source.as_str(),
            &[
                helper,
                constructor,
                "self.dispatch_runtime_event(event_loop, event);",
            ],
            "runtime entry should keep window status event forwarding source-visible",
        );
    }

    assert_source_order(
        runtime_window_lifecycle_source.as_str(),
        &[
            "fn handle_window_occluded",
            "if self.frame_cadence.set_window_occluded(occluded)",
            "self.request_runtime_frame();",
            "ZrRuntimeEventV1::window_occluded",
            "self.dispatch_runtime_event(event_loop, event);",
        ],
        "occlusion must update the cadence owner before the runtime status event is dispatched",
    );
}
