use super::super::source_assertions::assert_source_order;
use super::sources::{
    runtime_app_source, runtime_application_handler_source, runtime_entry_app_path,
    runtime_ime_input_root_source, runtime_ime_input_source, runtime_window_events_source,
};

#[test]
fn runtime_entry_keeps_ime_input_source_visible() {
    let runtime_app_source = runtime_app_source();
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_ime_input_root_source = runtime_ime_input_root_source();
    let runtime_ime_input_source = runtime_ime_input_source();
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
        !runtime_entry_app_path("ime_input.rs").exists(),
        "runtime IME input host should stay folder-backed instead of returning to an umbrella ime_input.rs file"
    );
}
