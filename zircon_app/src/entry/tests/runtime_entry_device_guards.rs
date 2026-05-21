use super::source_assertions::assert_source_order;

#[test]
fn runtime_entry_device_event_dispatch_stays_in_child_module() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_handler_source = include_str!("../runtime_entry_app/application_handler/hooks.rs");
    let runtime_device_events_root_source =
        include_str!("../runtime_entry_app/device_events/mod.rs");
    let runtime_device_events_dispatch_source =
        include_str!("../runtime_entry_app/device_events/dispatch.rs");
    let runtime_pointer_input_source = include_str!("../runtime_entry_app/pointer_input/device.rs");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry");

    assert!(
        runtime_app_source.contains("mod device_events;"),
        "runtime entry app should keep raw device-event dispatch in a child module"
    );
    assert!(
        runtime_device_events_root_source.contains("mod dispatch;"),
        "runtime device-events root should stay structural and delegate dispatch behavior"
    );
    assert!(
        !root.join("runtime_entry_app/device_events.rs").exists(),
        "runtime device events should stay folder-backed instead of returning to an umbrella device_events.rs file"
    );
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
        runtime_pointer_input_source,
        &[
            "fn handle_pointer_device_event",
            "DeviceEvent::PointerMotion",
            "ZrRuntimeEventV1::mouse_motion",
        ],
        "pointer-input module should keep raw pointer-motion runtime event construction",
    );
}
