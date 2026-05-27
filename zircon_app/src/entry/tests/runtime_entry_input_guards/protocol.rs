use super::sources::{
    runtime_app_source, runtime_application_handler_source, runtime_converter_root_source,
    runtime_entry_app_path, runtime_event_translation_source, runtime_file_drag_drop_root_source,
    runtime_frame_loop_source, runtime_gamepad_root_source, runtime_gamepad_source,
    runtime_host_requests_ime_root_source, runtime_host_requests_root_source,
    runtime_ime_input_root_source, runtime_keyboard_input_root_source,
    runtime_pointer_input_root_source, runtime_window_lifecycle_root_source,
};

#[test]
fn runtime_input_protocol_crosses_through_runtime_interface_events() {
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_app_source = runtime_app_source();
    let runtime_converter_root_source = runtime_converter_root_source();
    let runtime_file_drag_drop_root_source = runtime_file_drag_drop_root_source();
    let runtime_frame_loop_source = runtime_frame_loop_source();
    let runtime_host_requests_root_source = runtime_host_requests_root_source();
    let runtime_host_requests_ime_root_source = runtime_host_requests_ime_root_source();
    let runtime_ime_input_root_source = runtime_ime_input_root_source();
    let runtime_keyboard_input_root_source = runtime_keyboard_input_root_source();
    let runtime_pointer_input_root_source = runtime_pointer_input_root_source();
    let runtime_window_lifecycle_root_source = runtime_window_lifecycle_root_source();
    let runtime_gamepad_root_source = runtime_gamepad_root_source();
    let runtime_gamepad_source = runtime_gamepad_source();
    let runtime_event_translation_source = runtime_event_translation_source();

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
        !runtime_entry_app_path("file_drag_drop.rs").exists(),
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
    for required in ["GamepadRumble", "apply_runtime_gamepad_rumble_request"] {
        assert!(
            runtime_event_translation_source.contains(required),
            "runtime host-request routing should preserve gamepad rumble request marker `{required}`"
        );
    }
    for required in [
        "mod rumble;",
        "EffectBuilder::new",
        "BaseEffectType::Strong",
        "BaseEffectType::Weak",
        "clear_gamepad_rumble_effects",
        "clear_gamepad_rumble_effects_for_gamepad",
        "runtime_gamepad_rumble_force_feedback_not_supported",
        "EventType::Disconnected",
    ] {
        assert!(
            runtime_gamepad_source.contains(required),
            "runtime gamepad host should preserve rumble backend marker `{required}`"
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
        !runtime_entry_app_path("host_requests.rs").exists(),
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
        !runtime_entry_app_path("ime_input.rs").exists(),
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
        !runtime_entry_app_path("keyboard_input.rs").exists(),
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
        !runtime_entry_app_path("pointer_input.rs").exists(),
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
        !runtime_entry_app_path("window_lifecycle.rs").exists(),
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
        "send_raw_button",
    ] {
        assert!(
            runtime_gamepad_source.contains(required),
            "runtime gamepad host should preserve `{required}` translation"
        );
    }
    assert!(
        !runtime_gamepad_source.contains("value >= 0.5"),
        "runtime gamepad host should forward raw analog button values and leave thresholds to runtime input state"
    );
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
        !runtime_entry_app_path("gamepad.rs").exists(),
        "runtime gamepad host should stay folder-backed instead of returning to an umbrella gamepad.rs file"
    );
    assert!(
        !runtime_handler_source.contains("zircon_runtime::input"),
        "runtime window event handling should not import runtime implementation input types"
    );
}
