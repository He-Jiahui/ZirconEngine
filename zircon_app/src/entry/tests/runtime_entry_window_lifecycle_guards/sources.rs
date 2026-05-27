use std::path::{Path, PathBuf};

pub(super) fn runtime_entry_app_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry")
        .join("runtime_entry_app")
}

pub(super) fn runtime_app_source() -> &'static str {
    include_str!("../../runtime_entry_app/mod.rs")
}

pub(super) fn runtime_window_creation_source() -> &'static str {
    include_str!("../../runtime_entry_app/window_creation.rs")
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

pub(super) fn runtime_window_events_source() -> String {
    [
        include_str!("../../runtime_entry_app/window_events/mod.rs"),
        include_str!("../../runtime_entry_app/window_events/dispatch.rs"),
    ]
    .join("\n")
}
