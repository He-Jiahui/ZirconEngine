use std::path::Path;

use super::source_assertions::assert_source_order;

fn runtime_application_handler_source() -> &'static str {
    include_str!("../runtime_entry_app/application_handler/hooks.rs")
}

fn runtime_window_events_source() -> String {
    [
        include_str!("../runtime_entry_app/window_events/mod.rs"),
        include_str!("../runtime_entry_app/window_events/dispatch.rs"),
    ]
    .join("\n")
}

#[test]
fn runtime_entry_keeps_pointer_input_source_visible() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_device_events_source =
        include_str!("../runtime_entry_app/device_events/dispatch.rs");
    let runtime_pointer_input_root_source =
        include_str!("../runtime_entry_app/pointer_input/mod.rs");
    let runtime_pointer_input_button_source =
        include_str!("../runtime_entry_app/pointer_input/button.rs");
    let runtime_pointer_input_cursor_source =
        include_str!("../runtime_entry_app/pointer_input/cursor.rs");
    let runtime_pointer_input_device_source =
        include_str!("../runtime_entry_app/pointer_input/device.rs");
    let runtime_pointer_input_motion_source =
        include_str!("../runtime_entry_app/pointer_input/motion.rs");
    let runtime_pointer_input_wheel_source =
        include_str!("../runtime_entry_app/pointer_input/wheel.rs");
    let runtime_pointer_input_source = [
        runtime_pointer_input_root_source,
        runtime_pointer_input_button_source,
        runtime_pointer_input_cursor_source,
        runtime_pointer_input_device_source,
        runtime_pointer_input_motion_source,
        runtime_pointer_input_wheel_source,
    ]
    .join("\n");
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
        !std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("entry")
            .join("runtime_entry_app")
            .join("pointer_input.rs")
            .exists(),
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

#[test]
fn runtime_entry_keeps_file_drag_drop_source_visible() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_file_drag_drop_root_source =
        include_str!("../runtime_entry_app/file_drag_drop/mod.rs");
    let runtime_file_drag_drop_cancelled_source =
        include_str!("../runtime_entry_app/file_drag_drop/cancelled.rs");
    let runtime_file_drag_drop_dropped_source =
        include_str!("../runtime_entry_app/file_drag_drop/dropped.rs");
    let runtime_file_drag_drop_hovered_source =
        include_str!("../runtime_entry_app/file_drag_drop/hovered.rs");
    let runtime_file_drag_drop_source = [
        runtime_file_drag_drop_root_source,
        runtime_file_drag_drop_cancelled_source,
        runtime_file_drag_drop_dropped_source,
        runtime_file_drag_drop_hovered_source,
    ]
    .join("\n");
    let runtime_window_events_source = runtime_window_events_source();

    assert!(
        runtime_app_source.contains("mod file_drag_drop;"),
        "runtime entry app should keep file drag/drop forwarding in a child module"
    );
    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::DragEntered { paths, .. }",
            "self.handle_files_hovered(event_loop, paths);",
            "WindowEvent::DragDropped { paths, .. }",
            "self.handle_files_dropped(event_loop, paths);",
            "WindowEvent::DragLeft { .. }",
            "self.handle_file_drag_cancelled(event_loop);",
        ],
        "runtime window event handling should delegate file drag/drop forwarding",
    );
    for forbidden in [
        "ZrRuntimeEventV1::file_hovered",
        "ZrRuntimeEventV1::file_dropped",
        "ZrRuntimeEventV1::file_drag_cancelled",
        "path.to_string_lossy()",
    ] {
        assert!(
            !runtime_handler_source.contains(forbidden),
            "runtime ApplicationHandler should not own file drag/drop implementation detail `{forbidden}`"
        );
    }
    for required in [
        "pub(in crate::entry::runtime_entry_app) fn handle_files_hovered",
        "ZrRuntimeEventV1::file_hovered",
        "pub(in crate::entry::runtime_entry_app) fn handle_files_dropped",
        "ZrRuntimeEventV1::file_dropped",
        "pub(in crate::entry::runtime_entry_app) fn handle_file_drag_cancelled",
        "ZrRuntimeEventV1::file_drag_cancelled",
        "path.to_string_lossy().to_string()",
        "byte_slice(path_text.as_str())",
    ] {
        assert!(
            runtime_file_drag_drop_source.contains(required),
            "runtime file drag/drop module should preserve `{required}`"
        );
    }
    for required in ["mod cancelled;", "mod dropped;", "mod hovered;"] {
        assert!(
            runtime_file_drag_drop_root_source.contains(required),
            "runtime file drag/drop root should preserve structural wiring `{required}`"
        );
    }
    assert!(
        !std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("entry")
            .join("runtime_entry_app")
            .join("file_drag_drop.rs")
            .exists(),
        "runtime file drag/drop host should stay folder-backed instead of returning to an umbrella file_drag_drop.rs file"
    );
}

#[test]
fn runtime_entry_keeps_keyboard_input_source_visible() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_keyboard_input_root_source =
        include_str!("../runtime_entry_app/keyboard_input/mod.rs");
    let runtime_keyboard_input_event_source =
        include_str!("../runtime_entry_app/keyboard_input/event.rs");
    let runtime_keyboard_input_payload_source =
        include_str!("../runtime_entry_app/keyboard_input/payload.rs");
    let runtime_keyboard_input_source = [
        runtime_keyboard_input_root_source,
        runtime_keyboard_input_event_source,
        runtime_keyboard_input_payload_source,
    ]
    .join("\n");
    let runtime_window_events_source = runtime_window_events_source();

    assert!(
        runtime_app_source.contains("mod keyboard_input;"),
        "runtime entry app should keep keyboard forwarding in a child module"
    );
    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::KeyboardInput { event, .. }",
            "self.handle_keyboard_input(event_loop, event);",
        ],
        "runtime window event handling should delegate keyboard forwarding",
    );
    for forbidden in [
        "ZrRuntimeEventV1::keyboard",
        "ZrByteSlice",
        "key_action(event.state)",
        "physical_key_code(&event.physical_key)",
    ] {
        assert!(
            !runtime_handler_source.contains(forbidden),
            "runtime ApplicationHandler should not own keyboard implementation detail `{forbidden}`"
        );
    }
    for required in [
        "pub(in crate::entry::runtime_entry_app) fn handle_keyboard_input",
        "key_action(event.state)",
        "ZrByteSlice",
        "text.as_bytes().as_ptr()",
        "physical_key_code(&event.physical_key)",
        "ZrRuntimeEventV1::keyboard",
    ] {
        assert!(
            runtime_keyboard_input_source.contains(required),
            "runtime keyboard input module should preserve `{required}`"
        );
    }
    for required in ["mod event;", "mod payload;"] {
        assert!(
            runtime_keyboard_input_root_source.contains(required),
            "runtime keyboard input root should preserve structural wiring `{required}`"
        );
    }
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("entry")
            .join("runtime_entry_app")
            .join("keyboard_input.rs")
            .exists(),
        "runtime keyboard input host should stay folder-backed instead of returning to an umbrella keyboard_input.rs file"
    );
}

#[test]
fn runtime_entry_keeps_ime_input_source_visible() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_ime_input_root_source = include_str!("../runtime_entry_app/ime_input/mod.rs");
    let runtime_ime_input_composition_source =
        include_str!("../runtime_entry_app/ime_input/composition.rs");
    let runtime_ime_input_deletion_source =
        include_str!("../runtime_entry_app/ime_input/deletion.rs");
    let runtime_ime_input_lifecycle_source =
        include_str!("../runtime_entry_app/ime_input/lifecycle.rs");
    let runtime_ime_input_routing_source =
        include_str!("../runtime_entry_app/ime_input/routing.rs");
    let runtime_ime_input_source = [
        runtime_ime_input_root_source,
        runtime_ime_input_composition_source,
        runtime_ime_input_deletion_source,
        runtime_ime_input_lifecycle_source,
        runtime_ime_input_routing_source,
    ]
    .join("\n");
    let runtime_window_events_source = runtime_window_events_source();

    assert!(
        runtime_app_source.contains("mod ime_input;"),
        "runtime entry app should keep IME forwarding in a child module"
    );
    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::Ime(ime)",
            "self.handle_ime_input(event_loop, ime);",
        ],
        "runtime window event handling should delegate IME forwarding",
    );
    for forbidden in [
        "ZrRuntimeEventV1::ime_enabled",
        "ZrRuntimeEventV1::ime_disabled",
        "ZrRuntimeEventV1::ime_preedit",
        "ZrRuntimeEventV1::ime_commit",
        "ZrRuntimeEventV1::ime_delete_surrounding",
        "ZR_RUNTIME_IME_CURSOR_HIDDEN_V1",
        "usize_to_u32(before_bytes)",
    ] {
        assert!(
            !runtime_handler_source.contains(forbidden),
            "runtime ApplicationHandler should not own IME implementation detail `{forbidden}`"
        );
    }
    for required in [
        "pub(in crate::entry::runtime_entry_app) fn handle_ime_input",
        "Ime::Enabled",
        "ZrRuntimeEventV1::ime_enabled",
        "Ime::Disabled",
        "ZrRuntimeEventV1::ime_disabled",
        "Ime::Preedit",
        "ZR_RUNTIME_IME_CURSOR_HIDDEN_V1",
        "ZrRuntimeEventV1::ime_preedit",
        "Ime::Commit",
        "ZrRuntimeEventV1::ime_commit",
        "Ime::DeleteSurrounding",
        "usize_to_u32(before_bytes)",
        "ZrRuntimeEventV1::ime_delete_surrounding",
    ] {
        assert!(
            runtime_ime_input_source.contains(required),
            "runtime IME input module should preserve `{required}`"
        );
    }
    for required in [
        "mod composition;",
        "mod deletion;",
        "mod lifecycle;",
        "mod routing;",
    ] {
        assert!(
            runtime_ime_input_root_source.contains(required),
            "runtime IME input root should preserve structural wiring `{required}`"
        );
    }
    assert!(
        !std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("entry")
            .join("runtime_entry_app")
            .join("ime_input.rs")
            .exists(),
        "runtime IME input host should stay folder-backed instead of returning to an umbrella ime_input.rs file"
    );
}

#[test]
fn runtime_input_protocol_crosses_through_runtime_interface_events() {
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_converter_root_source = include_str!("../runtime_entry_app/converters/mod.rs");
    let runtime_converter_abi_source = include_str!("../runtime_entry_app/converters/abi.rs");
    let runtime_converter_keyboard_source =
        include_str!("../runtime_entry_app/converters/keyboard.rs");
    let runtime_converter_pointer_source =
        include_str!("../runtime_entry_app/converters/pointer.rs");
    let runtime_converter_window_source = include_str!("../runtime_entry_app/converters/window.rs");
    let runtime_converters_source = [
        runtime_converter_root_source,
        runtime_converter_abi_source,
        runtime_converter_keyboard_source,
        runtime_converter_pointer_source,
        runtime_converter_window_source,
    ]
    .join("\n");
    let runtime_file_drag_drop_root_source =
        include_str!("../runtime_entry_app/file_drag_drop/mod.rs");
    let runtime_file_drag_drop_cancelled_source =
        include_str!("../runtime_entry_app/file_drag_drop/cancelled.rs");
    let runtime_file_drag_drop_dropped_source =
        include_str!("../runtime_entry_app/file_drag_drop/dropped.rs");
    let runtime_file_drag_drop_hovered_source =
        include_str!("../runtime_entry_app/file_drag_drop/hovered.rs");
    let runtime_file_drag_drop_source = [
        runtime_file_drag_drop_root_source,
        runtime_file_drag_drop_cancelled_source,
        runtime_file_drag_drop_dropped_source,
        runtime_file_drag_drop_hovered_source,
    ]
    .join("\n");
    let runtime_device_events_source =
        include_str!("../runtime_entry_app/device_events/dispatch.rs");
    let runtime_frame_loop_source = include_str!("../runtime_entry_app/frame_loop.rs");
    let runtime_host_requests_root_source =
        include_str!("../runtime_entry_app/host_requests/mod.rs");
    let runtime_host_requests_drain_source =
        include_str!("../runtime_entry_app/host_requests/drain.rs");
    let runtime_host_requests_routing_source =
        include_str!("../runtime_entry_app/host_requests/routing.rs");
    let runtime_host_requests_ime_root_source =
        include_str!("../runtime_entry_app/host_requests/ime/mod.rs");
    let runtime_host_requests_ime_enable_source =
        include_str!("../runtime_entry_app/host_requests/ime/enable.rs");
    let runtime_host_requests_ime_geometry_source =
        include_str!("../runtime_entry_app/host_requests/ime/geometry.rs");
    let runtime_host_requests_ime_request_source =
        include_str!("../runtime_entry_app/host_requests/ime/request.rs");
    let runtime_host_requests_ime_surrounding_text_source =
        include_str!("../runtime_entry_app/host_requests/ime/surrounding_text.rs");
    let runtime_host_requests_source = [
        runtime_host_requests_root_source,
        runtime_host_requests_drain_source,
        runtime_host_requests_routing_source,
        runtime_host_requests_ime_root_source,
        runtime_host_requests_ime_enable_source,
        runtime_host_requests_ime_geometry_source,
        runtime_host_requests_ime_request_source,
        runtime_host_requests_ime_surrounding_text_source,
    ]
    .join("\n");
    let runtime_ime_input_root_source = include_str!("../runtime_entry_app/ime_input/mod.rs");
    let runtime_ime_input_composition_source =
        include_str!("../runtime_entry_app/ime_input/composition.rs");
    let runtime_ime_input_deletion_source =
        include_str!("../runtime_entry_app/ime_input/deletion.rs");
    let runtime_ime_input_lifecycle_source =
        include_str!("../runtime_entry_app/ime_input/lifecycle.rs");
    let runtime_ime_input_routing_source =
        include_str!("../runtime_entry_app/ime_input/routing.rs");
    let runtime_ime_input_source = [
        runtime_ime_input_root_source,
        runtime_ime_input_composition_source,
        runtime_ime_input_deletion_source,
        runtime_ime_input_lifecycle_source,
        runtime_ime_input_routing_source,
    ]
    .join("\n");
    let runtime_keyboard_input_root_source =
        include_str!("../runtime_entry_app/keyboard_input/mod.rs");
    let runtime_keyboard_input_event_source =
        include_str!("../runtime_entry_app/keyboard_input/event.rs");
    let runtime_keyboard_input_payload_source =
        include_str!("../runtime_entry_app/keyboard_input/payload.rs");
    let runtime_keyboard_input_source = [
        runtime_keyboard_input_root_source,
        runtime_keyboard_input_event_source,
        runtime_keyboard_input_payload_source,
    ]
    .join("\n");
    let runtime_pointer_input_root_source =
        include_str!("../runtime_entry_app/pointer_input/mod.rs");
    let runtime_pointer_input_button_source =
        include_str!("../runtime_entry_app/pointer_input/button.rs");
    let runtime_pointer_input_cursor_source =
        include_str!("../runtime_entry_app/pointer_input/cursor.rs");
    let runtime_pointer_input_device_source =
        include_str!("../runtime_entry_app/pointer_input/device.rs");
    let runtime_pointer_input_motion_source =
        include_str!("../runtime_entry_app/pointer_input/motion.rs");
    let runtime_pointer_input_wheel_source =
        include_str!("../runtime_entry_app/pointer_input/wheel.rs");
    let runtime_pointer_input_source = [
        runtime_pointer_input_root_source,
        runtime_pointer_input_button_source,
        runtime_pointer_input_cursor_source,
        runtime_pointer_input_device_source,
        runtime_pointer_input_motion_source,
        runtime_pointer_input_wheel_source,
    ]
    .join("\n");
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_lifecycle_root_source =
        include_str!("../runtime_entry_app/window_lifecycle/mod.rs");
    let runtime_window_lifecycle_close_source =
        include_str!("../runtime_entry_app/window_lifecycle/close.rs");
    let runtime_window_lifecycle_focus_source =
        include_str!("../runtime_entry_app/window_lifecycle/focus.rs");
    let runtime_window_lifecycle_scale_factor_source =
        include_str!("../runtime_entry_app/window_lifecycle/scale_factor.rs");
    let runtime_window_lifecycle_status_source =
        include_str!("../runtime_entry_app/window_lifecycle/status.rs");
    let runtime_window_lifecycle_source = [
        runtime_window_lifecycle_root_source,
        runtime_window_lifecycle_close_source,
        runtime_window_lifecycle_focus_source,
        runtime_window_lifecycle_scale_factor_source,
        runtime_window_lifecycle_status_source,
    ]
    .join("\n");
    let runtime_gamepad_root_source = include_str!("../runtime_entry_app/gamepad/mod.rs");
    let runtime_gamepad_codes_source = include_str!("../runtime_entry_app/gamepad/codes.rs");
    let runtime_gamepad_events_source = include_str!("../runtime_entry_app/gamepad/events.rs");
    let runtime_gamepad_host_source = include_str!("../runtime_entry_app/gamepad/host.rs");
    let runtime_gamepad_polling_source = include_str!("../runtime_entry_app/gamepad/polling.rs");
    let runtime_gamepad_source = [
        runtime_gamepad_root_source,
        runtime_gamepad_codes_source,
        runtime_gamepad_events_source,
        runtime_gamepad_host_source,
        runtime_gamepad_polling_source,
    ]
    .join("\n");
    let runtime_event_translation_source = [
        runtime_handler_source,
        runtime_converters_source.as_str(),
        runtime_device_events_source,
        runtime_file_drag_drop_source.as_str(),
        runtime_frame_loop_source,
        runtime_host_requests_source.as_str(),
        runtime_ime_input_source.as_str(),
        runtime_keyboard_input_source.as_str(),
        runtime_pointer_input_source.as_str(),
        runtime_window_events_source.as_str(),
        runtime_window_lifecycle_source.as_str(),
    ]
    .join("\n");

    assert!(
        runtime_event_translation_source.contains("ZrRuntimeEventV1"),
        "runtime window event handling should delegate input forwarding through runtime interface events"
    );
    for required in [
        "WindowEvent::KeyboardInput",
        "ZrRuntimeEventV1::keyboard",
        "WindowEvent::Focused",
        "ZrRuntimeEventV1::lifecycle",
        "WindowEvent::CloseRequested",
        "WindowEvent::Destroyed",
        "WindowEvent::Moved",
        "WindowEvent::Occluded",
        "WindowEvent::ThemeChanged",
        "WindowEvent::ScaleFactorChanged",
        "ZrRuntimeEventV1::window_close_requested",
        "ZrRuntimeEventV1::window_destroyed",
        "ZrRuntimeEventV1::window_moved",
        "ZrRuntimeEventV1::window_occluded",
        "ZrRuntimeEventV1::window_theme_changed",
        "ZrRuntimeEventV1::window_backend_scale_factor_changed",
        "ZrRuntimeEventV1::window_scale_factor_changed",
        "WindowEvent::PointerEntered",
        "WindowEvent::PointerLeft",
        "ZrRuntimeEventV1::cursor_entered",
        "ZrRuntimeEventV1::cursor_left",
        "WindowEvent::DragEntered",
        "WindowEvent::DragDropped",
        "WindowEvent::DragLeft",
        "ZrRuntimeEventV1::file_hovered",
        "ZrRuntimeEventV1::file_dropped",
        "ZrRuntimeEventV1::file_drag_cancelled",
        "PointerSource::Touch",
        "PointerKind::Touch",
        "ZrRuntimeEventV1::touch",
        "DeviceEvent::PointerMotion",
        "ZrRuntimeEventV1::mouse_motion",
        "WindowEvent::MouseWheel",
        "MouseScrollDelta::LineDelta",
        "MouseScrollDelta::PixelDelta",
        "ZrRuntimeEventV1::mouse_wheel_delta",
        "ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1",
        "ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1",
        "WindowEvent::Ime",
        "Ime::Preedit",
        "Ime::DeleteSurrounding",
        "ZrRuntimeEventV1::ime_preedit",
        "ZrRuntimeEventV1::ime_commit",
        "ZrRuntimeEventV1::ime_delete_surrounding",
        "ZrRuntimeEventV1::ime_enabled",
        "ZrRuntimeEventV1::ime_disabled",
        "drain_host_requests",
        "ZrRuntimeHostRequestV1",
        "request_ime_update",
        "ImeRequest::Enable",
        "ImeRequest::Update",
        "ImeRequest::Disable",
        "ImeCapabilities::new",
        "ImeSurroundingText::new",
    ] {
        assert!(
            runtime_event_translation_source.contains(required),
            "runtime window event handling should preserve `{required}` translation"
        );
    }
    assert!(
        runtime_app_source.contains("mod converters;"),
        "runtime entry app should keep winit event conversion in a child module"
    );
    assert!(
        runtime_app_source.contains("mod device_events;"),
        "runtime entry app should keep raw device-event dispatch in a child module"
    );
    for required in [
        "mod abi;",
        "mod keyboard;",
        "mod pointer;",
        "mod window;",
        "pub(super) use abi::{byte_slice, usize_to_u32};",
        "pub(super) use keyboard::{key_action, physical_key_code};",
        "pub(super) use pointer::{",
        "pub(super) use window::window_theme;",
    ] {
        assert!(
            runtime_converter_root_source.contains(required),
            "runtime converter root should preserve `{required}`"
        );
    }
    assert!(
        runtime_app_source.contains("mod file_drag_drop;"),
        "runtime entry app should keep file drag/drop forwarding in a child module"
    );
    for required in ["mod cancelled;", "mod dropped;", "mod hovered;"] {
        assert!(
            runtime_file_drag_drop_root_source.contains(required),
            "runtime file drag/drop root should preserve structural wiring `{required}`"
        );
    }
    assert!(
        !std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("entry")
            .join("runtime_entry_app")
            .join("file_drag_drop.rs")
            .exists(),
        "runtime file drag/drop host should stay folder-backed instead of returning to an umbrella file_drag_drop.rs file"
    );
    assert!(
        runtime_app_source.contains("mod frame_loop;"),
        "runtime entry app should keep about-to-wait frame pumping in a child module"
    );
    assert!(
        runtime_app_source.contains("mod host_requests;"),
        "runtime entry app should keep runtime host-request application in a child module"
    );
    for required in ["mod drain;", "mod ime;", "mod routing;"] {
        assert!(
            runtime_host_requests_root_source.contains(required),
            "runtime host-request root should preserve structural wiring `{required}`"
        );
    }
    for required in [
        "mod enable;",
        "mod geometry;",
        "mod request;",
        "mod surrounding_text;",
        "pub(super) use request::apply_runtime_ime_host_request;",
    ] {
        assert!(
            runtime_host_requests_ime_root_source.contains(required),
            "runtime IME host-request root should preserve structural wiring `{required}`"
        );
    }
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("entry")
            .join("runtime_entry_app")
            .join("host_requests.rs")
            .exists(),
        "runtime host requests should stay folder-backed instead of returning to an umbrella host_requests.rs file"
    );
    assert!(
        runtime_app_source.contains("mod ime_input;"),
        "runtime entry app should keep IME forwarding in a child module"
    );
    for required in [
        "mod composition;",
        "mod deletion;",
        "mod lifecycle;",
        "mod routing;",
    ] {
        assert!(
            runtime_ime_input_root_source.contains(required),
            "runtime IME input root should preserve structural wiring `{required}`"
        );
    }
    assert!(
        !std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("entry")
            .join("runtime_entry_app")
            .join("ime_input.rs")
            .exists(),
        "runtime IME input host should stay folder-backed instead of returning to an umbrella ime_input.rs file"
    );
    assert!(
        runtime_app_source.contains("mod keyboard_input;"),
        "runtime entry app should keep keyboard forwarding in a child module"
    );
    for required in ["mod event;", "mod payload;"] {
        assert!(
            runtime_keyboard_input_root_source.contains(required),
            "runtime keyboard input root should preserve structural wiring `{required}`"
        );
    }
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("entry")
            .join("runtime_entry_app")
            .join("keyboard_input.rs")
            .exists(),
        "runtime keyboard input host should stay folder-backed instead of returning to an umbrella keyboard_input.rs file"
    );
    assert!(
        runtime_app_source.contains("mod pointer_input;"),
        "runtime entry app should keep pointer and mouse input forwarding in a child module"
    );
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
        !std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("entry")
            .join("runtime_entry_app")
            .join("pointer_input.rs")
            .exists(),
        "runtime pointer input host should stay folder-backed instead of returning to an umbrella pointer_input.rs file"
    );
    assert!(
        runtime_app_source.contains("mod window_events;"),
        "runtime entry app should keep concrete winit window-event dispatch in a child module"
    );
    assert!(
        runtime_app_source.contains("mod window_lifecycle;"),
        "runtime entry app should keep window lifecycle/status forwarding in a child module"
    );
    for required in [
        "mod close;",
        "mod focus;",
        "mod scale_factor;",
        "mod status;",
    ] {
        assert!(
            runtime_window_lifecycle_root_source.contains(required),
            "runtime window lifecycle root should preserve structural wiring `{required}`"
        );
    }
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("entry")
            .join("runtime_entry_app")
            .join("window_lifecycle.rs")
            .exists(),
        "runtime window lifecycle host should stay folder-backed instead of returning to an umbrella window_lifecycle.rs file"
    );
    assert!(
        !runtime_handler_source.contains("fn apply_runtime_ime_host_request"),
        "runtime window event handling should not own native IME host-request application"
    );
    for required in [
        "mod gamepad;",
        "gamepads: Option<gilrs::Gilrs>",
        "self.poll_gamepads(event_loop)",
    ] {
        assert!(
            runtime_app_source.contains(required)
                || runtime_handler_source.contains(required)
                || runtime_frame_loop_source.contains(required),
            "runtime entry app should preserve gamepad host wiring `{required}`"
        );
    }
    for required in [
        "GilrsBuilder::new",
        "pub(in crate::entry::runtime_entry_app) fn poll_gamepads",
        "EventType::ButtonChanged",
        "EventType::AxisChanged",
        "ZrRuntimeEventV1::gamepad_connection_with_ids",
        "ZrRuntimeEventV1::gamepad_button",
        "ZrRuntimeEventV1::gamepad_axis",
    ] {
        assert!(
            runtime_gamepad_source.contains(required),
            "runtime gamepad host should preserve `{required}` translation"
        );
    }
    for required in [
        "mod codes;",
        "mod events;",
        "mod host;",
        "mod polling;",
        "pub(super) use host::create_gilrs;",
    ] {
        assert!(
            runtime_gamepad_root_source.contains(required),
            "runtime gamepad root should preserve structural wiring `{required}`"
        );
    }
    assert!(
        !std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("entry")
            .join("runtime_entry_app")
            .join("gamepad.rs")
            .exists(),
        "runtime gamepad host should stay folder-backed instead of returning to an umbrella gamepad.rs file"
    );
    assert!(
        !runtime_handler_source.contains("zircon_runtime::input"),
        "runtime window event handling should not import runtime implementation input types"
    );
}
