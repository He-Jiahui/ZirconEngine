use super::support::{retired_flat_module, src_root};

#[test]
fn retired_dynamic_event_flat_files_stay_removed() {
    let src = src_root();
    for retired_flat_file in [
        retired_flat_module(&src, "", "dynamic_events"),
        retired_flat_module(&src, "", "dynamic_event_abi"),
        retired_flat_module(&src, "service_types", "dynamic_events"),
        retired_flat_module(&src, "service_types", "dynamic_event_executors"),
    ] {
        assert!(
            !retired_flat_file.exists(),
            "{} must stay retired; dynamic event behavior belongs in folder-backed modules",
            retired_flat_file.display()
        );
    }
}
