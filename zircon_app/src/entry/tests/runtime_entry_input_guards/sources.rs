use std::path::{Path, PathBuf};

pub(super) fn runtime_app_source() -> &'static str {
    include_str!("../../runtime_entry_app/mod.rs")
}

pub(super) fn runtime_application_handler_source() -> &'static str {
    include_str!("../../runtime_entry_app/application_handler/hooks.rs")
}

pub(super) fn runtime_entry_app_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry")
        .join("runtime_entry_app")
        .join(relative)
}

pub(super) fn runtime_window_events_source() -> String {
    [
        include_str!("../../runtime_entry_app/window_events/mod.rs"),
        include_str!("../../runtime_entry_app/window_events/dispatch.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_device_events_source() -> &'static str {
    include_str!("../../runtime_entry_app/device_events/dispatch.rs")
}

pub(super) fn runtime_frame_loop_source() -> &'static str {
    include_str!("../../runtime_entry_app/frame_loop.rs")
}

pub(super) fn runtime_converter_root_source() -> &'static str {
    include_str!("../../runtime_entry_app/converters/mod.rs")
}

pub(super) fn runtime_converters_source() -> String {
    [
        runtime_converter_root_source(),
        include_str!("../../runtime_entry_app/converters/abi.rs"),
        include_str!("../../runtime_entry_app/converters/keyboard.rs"),
        include_str!("../../runtime_entry_app/converters/pointer.rs"),
        include_str!("../../runtime_entry_app/converters/window.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_file_drag_drop_root_source() -> &'static str {
    include_str!("../../runtime_entry_app/file_drag_drop/mod.rs")
}

pub(super) fn runtime_file_drag_drop_source() -> String {
    [
        runtime_file_drag_drop_root_source(),
        include_str!("../../runtime_entry_app/file_drag_drop/cancelled.rs"),
        include_str!("../../runtime_entry_app/file_drag_drop/dropped.rs"),
        include_str!("../../runtime_entry_app/file_drag_drop/hovered.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_host_requests_root_source() -> &'static str {
    include_str!("../../runtime_entry_app/host_requests/mod.rs")
}

pub(super) fn runtime_host_requests_ime_root_source() -> &'static str {
    include_str!("../../runtime_entry_app/host_requests/ime/mod.rs")
}

pub(super) fn runtime_host_requests_source() -> String {
    [
        runtime_host_requests_root_source(),
        include_str!("../../runtime_entry_app/host_requests/drain.rs"),
        include_str!("../../runtime_entry_app/host_requests/routing.rs"),
        runtime_host_requests_ime_root_source(),
        include_str!("../../runtime_entry_app/host_requests/ime/enable.rs"),
        include_str!("../../runtime_entry_app/host_requests/ime/geometry.rs"),
        include_str!("../../runtime_entry_app/host_requests/ime/request.rs"),
        include_str!("../../runtime_entry_app/host_requests/ime/surrounding_text.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_ime_input_root_source() -> &'static str {
    include_str!("../../runtime_entry_app/ime_input/mod.rs")
}

pub(super) fn runtime_ime_input_source() -> String {
    [
        runtime_ime_input_root_source(),
        include_str!("../../runtime_entry_app/ime_input/composition.rs"),
        include_str!("../../runtime_entry_app/ime_input/deletion.rs"),
        include_str!("../../runtime_entry_app/ime_input/lifecycle.rs"),
        include_str!("../../runtime_entry_app/ime_input/routing.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_keyboard_input_root_source() -> &'static str {
    include_str!("../../runtime_entry_app/keyboard_input/mod.rs")
}

pub(super) fn runtime_keyboard_input_source() -> String {
    [
        runtime_keyboard_input_root_source(),
        include_str!("../../runtime_entry_app/keyboard_input/event.rs"),
        include_str!("../../runtime_entry_app/keyboard_input/payload.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_pointer_input_root_source() -> &'static str {
    include_str!("../../runtime_entry_app/pointer_input/mod.rs")
}

pub(super) fn runtime_pointer_input_source() -> String {
    [
        runtime_pointer_input_root_source(),
        include_str!("../../runtime_entry_app/pointer_input/button.rs"),
        include_str!("../../runtime_entry_app/pointer_input/cursor.rs"),
        include_str!("../../runtime_entry_app/pointer_input/device.rs"),
        include_str!("../../runtime_entry_app/pointer_input/motion.rs"),
        include_str!("../../runtime_entry_app/pointer_input/wheel.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_window_lifecycle_root_source() -> &'static str {
    include_str!("../../runtime_entry_app/window_lifecycle/mod.rs")
}

pub(super) fn runtime_window_lifecycle_source() -> String {
    [
        runtime_window_lifecycle_root_source(),
        include_str!("../../runtime_entry_app/window_lifecycle/close.rs"),
        include_str!("../../runtime_entry_app/window_lifecycle/focus.rs"),
        include_str!("../../runtime_entry_app/window_lifecycle/scale_factor.rs"),
        include_str!("../../runtime_entry_app/window_lifecycle/status.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_gamepad_root_source() -> &'static str {
    include_str!("../../runtime_entry_app/gamepad/mod.rs")
}

pub(super) fn runtime_gamepad_source() -> String {
    [
        runtime_gamepad_root_source(),
        include_str!("../../runtime_entry_app/gamepad/codes.rs"),
        include_str!("../../runtime_entry_app/gamepad/events.rs"),
        include_str!("../../runtime_entry_app/gamepad/host.rs"),
        include_str!("../../runtime_entry_app/gamepad/polling.rs"),
        include_str!("../../runtime_entry_app/gamepad/rumble.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_event_translation_source() -> String {
    let converters = runtime_converters_source();
    let file_drag_drop = runtime_file_drag_drop_source();
    let host_requests = runtime_host_requests_source();
    let ime_input = runtime_ime_input_source();
    let keyboard_input = runtime_keyboard_input_source();
    let pointer_input = runtime_pointer_input_source();
    let window_events = runtime_window_events_source();
    let window_lifecycle = runtime_window_lifecycle_source();

    [
        runtime_application_handler_source(),
        converters.as_str(),
        runtime_device_events_source(),
        file_drag_drop.as_str(),
        runtime_frame_loop_source(),
        host_requests.as_str(),
        ime_input.as_str(),
        keyboard_input.as_str(),
        pointer_input.as_str(),
        window_events.as_str(),
        window_lifecycle.as_str(),
    ]
    .join("\n")
}
