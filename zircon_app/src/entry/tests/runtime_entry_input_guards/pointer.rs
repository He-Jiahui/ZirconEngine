use super::super::source_assertions::assert_source_order;
use super::sources::{
    runtime_app_source, runtime_application_handler_source, runtime_device_events_source,
    runtime_entry_app_path, runtime_pointer_input_root_source, runtime_pointer_input_source,
    runtime_window_events_source,
};

#[test]
fn runtime_entry_keeps_pointer_input_source_visible() {
    let runtime_app_source = runtime_app_source();
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_device_events_source = runtime_device_events_source();
    let runtime_pointer_input_root_source = runtime_pointer_input_root_source();
    let runtime_pointer_input_source = runtime_pointer_input_source();
    let runtime_window_events_source = runtime_window_events_source();

    assert!(
        runtime_app_source.contains("mod pointer_input;"),
        "runtime entry app should keep pointer and mouse input forwarding in a child module"
    );
    assert!(
        runtime_app_source.contains("mod device_events;"),
        "runtime entry app should keep raw device-event dispatch in a child module"
    );
    for (event, helper) in [
        (
            "WindowEvent::PointerEntered",
            "self.handle_pointer_entered(event_loop);",
        ),
        (
            "WindowEvent::PointerLeft { position, kind, .. }",
            "self.handle_pointer_left(event_loop, position, kind);",
        ),
        (
            "WindowEvent::PointerMoved",
            "self.handle_pointer_moved(event_loop, position, source);",
        ),
        (
            "WindowEvent::PointerButton",
            "self.handle_pointer_button(event_loop, state, button, position);",
        ),
        (
            "WindowEvent::MouseWheel { delta, .. }",
            "self.handle_mouse_wheel(event_loop, delta);",
        ),
    ] {
        assert_source_order(
            runtime_window_events_source.as_str(),
            &[event, helper],
            "runtime window event handling should delegate pointer and mouse event forwarding",
        );
    }
    assert_source_order(
        runtime_device_events_source,
        &[
            "fn handle_device_event",
            "DeviceEvent",
            "self.handle_pointer_device_event(event_loop, event);",
        ],
        "runtime device event handling should delegate raw pointer motion forwarding",
    );
    for forbidden in [
        "ZrRuntimeEventV1::cursor_entered",
        "ZrRuntimeEventV1::cursor_left",
        "ZrRuntimeEventV1::pointer_moved",
        "ZrRuntimeEventV1::mouse_button",
        "ZrRuntimeEventV1::mouse_wheel_delta",
        "ZrRuntimeEventV1::mouse_motion",
        "pointer_source_touch_id",
        "touch_button_phase",
    ] {
        assert!(
            !runtime_handler_source.contains(forbidden),
            "runtime ApplicationHandler should not own pointer/mouse implementation detail `{forbidden}`"
        );
    }
    for required in [
        "mod button;",
        "mod cursor;",
        "mod device;",
        "mod motion;",
        "mod wheel;",
    ] {
        assert!(
            runtime_pointer_input_root_source.contains(required),
            "runtime pointer input root should preserve structural wiring `{required}`"
        );
    }
    assert!(
        !runtime_entry_app_path("pointer_input.rs").exists(),
        "runtime pointer input should stay folder-backed instead of returning to an umbrella pointer_input.rs file"
    );
    for required in [
        "pub(in crate::entry::runtime_entry_app) fn handle_pointer_entered",
        "ZrRuntimeEventV1::cursor_entered",
        "pub(in crate::entry::runtime_entry_app) fn handle_pointer_left",
        "ZrRuntimeEventV1::cursor_left",
        "pointer_kind_touch_id(kind)",
        "ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1",
        "pub(in crate::entry::runtime_entry_app) fn handle_pointer_moved",
        "pointer_source_touch_id(&source)",
        "ZR_RUNTIME_TOUCH_PHASE_MOVED_V1",
        "ZrRuntimeEventV1::pointer_moved",
        "pub(in crate::entry::runtime_entry_app) fn handle_pointer_button",
        "touch_button_phase(&button, state)",
        "mouse_button(button)",
        "button_state(state)",
        "ZrRuntimeEventV1::mouse_button",
        "pub(in crate::entry::runtime_entry_app) fn handle_mouse_wheel",
        "mouse_wheel_delta(delta)",
        "ZrRuntimeEventV1::mouse_wheel_delta",
        "pub(in crate::entry::runtime_entry_app) fn handle_pointer_device_event",
        "DeviceEvent::PointerMotion",
        "ZrRuntimeEventV1::mouse_motion",
    ] {
        assert!(
            runtime_pointer_input_source.contains(required),
            "runtime pointer input module should preserve `{required}`"
        );
    }
}
