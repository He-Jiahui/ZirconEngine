use std::path::{Path, PathBuf};

pub(super) fn runtime_event_loop_policy_source() -> String {
    [
        include_str!("../../runtime_entry_app/event_loop_policy/mod.rs"),
        include_str!("../../runtime_entry_app/event_loop_policy/control_flow.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_config_source() -> String {
    [
        include_str!("../../runtime_entry_app/config/mod.rs"),
        include_str!("../../runtime_entry_app/config/app_config.rs"),
    ]
    .join("\n")
}

pub(super) fn runtime_application_handler_source() -> &'static str {
    include_str!("../../runtime_entry_app/application_handler/hooks.rs")
}

pub(super) fn runtime_window_events_source() -> String {
    [
        include_str!("../../runtime_entry_app/window_events/mod.rs"),
        include_str!("../../runtime_entry_app/window_events/dispatch.rs"),
    ]
    .join("\n")
}

pub(super) fn entry_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry")
}

pub(super) fn runtime_entry_app_root() -> PathBuf {
    entry_root().join("runtime_entry_app")
}
