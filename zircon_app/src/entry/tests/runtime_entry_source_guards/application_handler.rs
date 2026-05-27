use super::super::source_assertions::assert_source_order;
use super::sources::{entry_root, runtime_application_handler_source};

#[test]
fn runtime_entry_application_handler_stays_folder_backed_hook_surface() {
    let runtime_app_source = include_str!("../../runtime_entry_app/mod.rs");
    let application_handler_root_source =
        include_str!("../../runtime_entry_app/application_handler/mod.rs");
    let runtime_handler_source = runtime_application_handler_source();
    let root = entry_root();

    assert!(
        runtime_app_source.contains("mod application_handler;"),
        "runtime entry app should keep the winit ApplicationHandler implementation in a child module"
    );
    assert!(
        application_handler_root_source.contains("mod hooks;"),
        "runtime application-handler root should stay structural and delegate trait hooks"
    );
    assert!(
        !root.join("runtime_entry_app/application_handler.rs")
            .exists(),
        "runtime application handler should stay folder-backed instead of returning to an umbrella application_handler.rs file"
    );
    assert_source_order(
        runtime_handler_source,
        &[
            "impl ApplicationHandler for RuntimeEntryApp",
            "fn can_create_surfaces",
            "self.create_primary_window_surface(event_loop);",
            "fn window_event",
            "self.handle_window_event(event_loop, event);",
            "fn about_to_wait",
            "self.pump_frame_loop(event_loop);",
            "fn device_event",
            "self.handle_device_event(event_loop, event);",
        ],
        "runtime ApplicationHandler hooks should remain a narrow profile-and-delegate surface",
    );
}
