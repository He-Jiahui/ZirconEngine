use super::super::source_assertions::assert_source_order;
use super::sources::{
    runtime_application_handler_source, runtime_device_events_dispatch_source,
    runtime_pointer_device_source,
};

#[test]
fn runtime_entry_device_event_dispatch_delegates_to_pointer_device_input() {
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_device_events_dispatch_source = runtime_device_events_dispatch_source();
    let runtime_pointer_device_source = runtime_pointer_device_source();

    assert_source_order(
        runtime_handler_source,
        &[
            "fn device_event",
            "zircon_runtime::profile_scope!(\"app\", \"runtime_entry\", \"device_event\");",
            "self.handle_device_event(event_loop, event);",
        ],
        "ApplicationHandler::device_event should only profile and delegate raw device dispatch",
    );
    assert!(
        !runtime_handler_source.contains("self.handle_pointer_device_event(event_loop, event);"),
        "ApplicationHandler should not call pointer-device forwarding directly"
    );
    assert_source_order(
        runtime_device_events_dispatch_source,
        &[
            "fn handle_device_event",
            "DeviceEvent",
            "self.handle_pointer_device_event(event_loop, event);",
        ],
        "device-event dispatcher should route raw device events to the pointer-input module",
    );
    assert_source_order(
        runtime_pointer_device_source,
        &[
            "fn handle_pointer_device_event",
            "DeviceEvent::PointerMotion",
            "ZrRuntimeEventV1::mouse_motion",
        ],
        "pointer-input module should keep raw pointer-motion runtime event construction",
    );
}
