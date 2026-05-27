use super::super::source_assertions::assert_source_order;
use super::sources::{
    runtime_app_source, runtime_application_handler_source, runtime_entry_app_path,
    runtime_file_drag_drop_root_source, runtime_file_drag_drop_source,
    runtime_window_events_source,
};

#[test]
fn runtime_entry_keeps_file_drag_drop_source_visible() {
    let runtime_app_source = runtime_app_source();
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_file_drag_drop_root_source = runtime_file_drag_drop_root_source();
    let runtime_file_drag_drop_source = runtime_file_drag_drop_source();
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
        !runtime_entry_app_path("file_drag_drop.rs").exists(),
        "runtime file drag/drop host should stay folder-backed instead of returning to an umbrella file_drag_drop.rs file"
    );
}
