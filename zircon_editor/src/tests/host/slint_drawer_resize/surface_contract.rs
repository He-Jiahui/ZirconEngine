fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_resize_surface_uses_rust_owned_pointer_event_contract() {
    let globals = source("src/ui/slint_host/host_contract/globals.rs");
    let wiring = source("src/ui/slint_host/app/callback_wiring.rs");
    let docking = source("src/ui/slint_host/app/workspace_docking.rs");
    let resize_surface = source("src/ui/slint_host/shell_pointer/resize_surface.rs");
    let shell_pointer_bridge = source("src/ui/slint_host/shell_pointer/bridge.rs");

    assert!(globals.contains("on_host_resize_pointer_event"));
    assert!(wiring.contains("host_shell.on_host_resize_pointer_event("));
    assert!(resize_surface.contains("resolve_root_left_splitter_frame"));
    assert!(!resize_surface.contains(".splitter_frame("));
    assert!(shell_pointer_bridge.contains("update_resize_surface("));
    assert!(shell_pointer_bridge.contains("shared_root_frames,"));
    assert!(!shell_pointer_bridge.contains(
        "update_resize_surface(\n            &mut self.resize_surface,\n            root_size,\n            geometry,"
    ));
    for required in [
        "pub(super) fn host_resize_pointer_event",
        "begin_drawer_resize_capture",
        "update_drawer_resize_capture",
        "finish_drawer_resize_capture",
    ] {
        assert!(
            docking.contains(required),
            "workspace docking missing `{required}`"
        );
    }
    for legacy in [
        "on_begin_drawer_resize",
        "on_update_drawer_resize",
        "on_finish_drawer_resize",
    ] {
        assert!(
            !wiring.contains(legacy),
            "resize wiring should not keep `{legacy}`"
        );
    }
}
