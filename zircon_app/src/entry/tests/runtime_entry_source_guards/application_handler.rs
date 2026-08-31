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
        !root
            .join("runtime_entry_app/application_handler.rs")
            .exists(),
        "runtime application handler should stay folder-backed instead of returning to an umbrella application_handler.rs file"
    );
    assert_source_order(
        runtime_handler_source,
        &[
            "impl ApplicationHandler for RuntimeEntryApp",
            "fn resumed",
            "zircon_runtime::profile_scope!(\"app\", \"runtime_entry\", \"resumed\");",
            "self.handle_application_resumed(event_loop);",
            "fn can_create_surfaces",
            "zircon_runtime::profile_scope!(\"app\", \"runtime_entry\", \"can_create_surfaces\");",
            "self.handle_surface_availability(event_loop);",
            "fn suspended",
            "zircon_runtime::profile_scope!(\"app\", \"runtime_entry\", \"suspended\");",
            "self.handle_application_suspended(event_loop);",
            "fn destroy_surfaces",
            "zircon_runtime::profile_scope!(\"app\", \"runtime_entry\", \"destroy_surfaces\");",
            "self.handle_surface_destruction(event_loop);",
            "fn exiting",
            "zircon_runtime::profile_scope!(\"app\", \"runtime_entry\", \"exiting\");",
            "self.handle_application_exit(event_loop);",
            "fn proxy_wake_up",
            "self.request_runtime_frame();",
            "fn window_event",
            "self.handle_window_event(event_loop, event);",
            "fn about_to_wait",
            "self.application_lifecycle.allows_frame_pump()",
            "self.pump_frame_loop(event_loop);",
            "fn device_event",
            "self.handle_device_event(event_loop, event);",
        ],
        "runtime ApplicationHandler hooks should remain a narrow profile-and-delegate surface",
    );
    assert!(
        runtime_app_source.contains("mod application_lifecycle;"),
        "runtime entry app should keep lifecycle state outside the ApplicationHandler hook module"
    );
    let resumed_hook = runtime_handler_source
        .split("fn resumed")
        .nth(1)
        .and_then(|source| source.split("fn can_create_surfaces").next())
        .expect("ApplicationHandler should retain resumed before can_create_surfaces");
    assert!(
        !resumed_hook.contains("create_primary_window_surface"),
        "winit requires render surfaces to be created from can_create_surfaces, not resumed"
    );
    assert_eq!(
        runtime_handler_source
            .matches("if !self.failure_state.is_recorded() {")
            .count(),
        4,
        "after a terminal callback failure, proxy, window, frame-loop, and device hooks must not submit new work"
    );
}
