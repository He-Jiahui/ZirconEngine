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

pub(super) fn runtime_application_handler_source() -> &'static str {
    include_str!("../../runtime_entry_app/application_handler/hooks.rs")
}

pub(super) fn runtime_device_events_root_source() -> &'static str {
    include_str!("../../runtime_entry_app/device_events/mod.rs")
}

pub(super) fn runtime_device_events_dispatch_source() -> &'static str {
    include_str!("../../runtime_entry_app/device_events/dispatch.rs")
}

pub(super) fn runtime_pointer_device_source() -> &'static str {
    include_str!("../../runtime_entry_app/pointer_input/device.rs")
}
