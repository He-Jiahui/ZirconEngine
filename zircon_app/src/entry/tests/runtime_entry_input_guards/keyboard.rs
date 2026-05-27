use super::super::source_assertions::assert_source_order;
use super::sources::{
    runtime_app_source, runtime_application_handler_source, runtime_entry_app_path,
    runtime_keyboard_input_root_source, runtime_keyboard_input_source,
    runtime_window_events_source,
};

#[test]
fn runtime_entry_keeps_keyboard_input_source_visible() {
    let runtime_app_source = runtime_app_source();
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_keyboard_input_root_source = runtime_keyboard_input_root_source();
    let runtime_keyboard_input_source = runtime_keyboard_input_source();
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
        !runtime_entry_app_path("keyboard_input.rs").exists(),
        "runtime keyboard input host should stay folder-backed instead of returning to an umbrella keyboard_input.rs file"
    );
}
