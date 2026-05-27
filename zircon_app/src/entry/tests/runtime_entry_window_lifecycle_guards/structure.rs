use super::super::source_assertions::assert_source_order;
use super::sources::{
    runtime_app_source, runtime_entry_app_root, runtime_window_creation_source,
    runtime_window_lifecycle_root_source,
};

#[test]
fn runtime_entry_keeps_window_lifecycle_sources_folder_backed() {
    let runtime_app_source = runtime_app_source();
    let runtime_window_creation_source = runtime_window_creation_source();
    let runtime_window_lifecycle_root_source = runtime_window_lifecycle_root_source();
    let root = runtime_entry_app_root();

    assert!(
        runtime_app_source.contains("mod window_lifecycle;"),
        "runtime entry app should keep window lifecycle/status handling in a child module"
    );
    assert!(
        runtime_app_source.contains("mod window_events;"),
        "runtime entry app should keep concrete window-event dispatch in a child module"
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
        !root.join("window_lifecycle.rs").exists(),
        "runtime window lifecycle should stay folder-backed instead of returning to an umbrella window_lifecycle.rs file"
    );
    assert_source_order(
        runtime_window_creation_source,
        &[
            "fn create_primary_window_surface",
            "self.window_descriptor.primary_window.is_none()",
            "return;",
            "let window_attributes = runtime_window_attributes(&self.window_descriptor, event_loop);",
        ],
        "runtime entry should honor no-primary-window host policy before creating winit attributes",
    );
}
