use super::sources::{
    runtime_app_source, runtime_device_events_root_source, runtime_entry_app_root,
};

#[test]
fn runtime_entry_device_event_sources_stay_folder_backed() {
    let runtime_app_source = runtime_app_source();
    let runtime_device_events_root_source = runtime_device_events_root_source();
    let root = runtime_entry_app_root();

    assert!(
        runtime_app_source.contains("mod device_events;"),
        "runtime entry app should keep raw device-event dispatch in a child module"
    );
    assert!(
        runtime_device_events_root_source.contains("mod dispatch;"),
        "runtime device-events root should stay structural and delegate dispatch behavior"
    );
    assert!(
        !root.join("device_events.rs").exists(),
        "runtime device events should stay folder-backed instead of returning to an umbrella device_events.rs file"
    );
}
