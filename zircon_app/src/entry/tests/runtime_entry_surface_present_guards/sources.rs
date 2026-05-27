use std::path::{Path, PathBuf};

pub(super) fn entry_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry")
}

pub(super) fn runtime_app_source() -> &'static str {
    include_str!("../../runtime_entry_app/mod.rs")
}

pub(super) fn runtime_application_handler_source() -> &'static str {
    include_str!("../../runtime_entry_app/application_handler/hooks.rs")
}

pub(super) fn runtime_frame_loop_source() -> &'static str {
    include_str!("../../runtime_entry_app/frame_loop.rs")
}

pub(super) fn runtime_window_creation_source() -> &'static str {
    include_str!("../../runtime_entry_app/window_creation.rs")
}

pub(super) fn runtime_session_source() -> &'static str {
    include_str!("../../runtime_library/runtime_session.rs")
}

pub(super) fn runtime_surface_present_source() -> String {
    [
        include_str!("../../runtime_entry_app/surface_present/mod.rs"),
        include_str!("../../runtime_entry_app/surface_present/binding.rs"),
        include_str!("../../runtime_entry_app/surface_present/lifecycle.rs"),
        include_str!("../../runtime_entry_app/surface_present/fallback.rs"),
        include_str!("../../runtime_entry_app/surface_present/redraw.rs"),
        include_str!("../../runtime_entry_app/surface_present/resize.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_window_surface_source() -> String {
    [
        include_str!("../../runtime_entry_app/window_surface/mod.rs"),
        include_str!("../../runtime_entry_app/window_surface/native_target.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_window_events_source() -> String {
    [
        include_str!("../../runtime_entry_app/window_events/mod.rs"),
        include_str!("../../runtime_entry_app/window_events/dispatch.rs"),
    ]
    .join("\n")
}
