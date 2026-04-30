fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_drag_capture_surface_uses_rust_owned_pointer_event_contract() {
    let globals = source("src/ui/slint_host/host_contract/globals.rs");
    let wiring = source("src/ui/slint_host/app/callback_wiring.rs");
    let docking = source("src/ui/slint_host/app/workspace_docking.rs");

    assert!(globals.contains("on_host_drag_pointer_event"));
    assert!(wiring.contains("host_shell.on_host_drag_pointer_event("));
    for required in [
        "pub(super) fn host_drag_pointer_event",
        "sync_drag_target_group",
        "dispatch_drag_drop_from_pointer",
        "HOST_POINTER_UP",
    ] {
        assert!(docking.contains(required), "workspace docking missing `{required}`");
    }
    for legacy in ["on_drop_tab", "on_update_drag_target"] {
        assert!(!wiring.contains(legacy), "drag wiring should not keep `{legacy}`");
    }
}
