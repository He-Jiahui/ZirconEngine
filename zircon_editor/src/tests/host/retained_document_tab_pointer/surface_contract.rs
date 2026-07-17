fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_document_tab_surfaces_use_rust_owned_pointer_callbacks() {
    let globals = source("src/ui/retained_host/host_contract/globals.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring/host_shell/chrome.rs");
    let pointer_layout = source("src/ui/retained_host/app/pointer_layout/shell_chrome.rs");
    let document_tab_sync = pointer_layout
        .split("fn sync_document_tab_pointer_layout")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn sync_drawer_header_pointer_layout_with_workbench_layout_frames")
                .next()
        })
        .expect("document tab pointer sync function should exist");

    for required in [
        "on_document_tab_pointer_clicked",
        "on_document_tab_close_pointer_clicked",
        "document_tab_pointer_clicked",
        "document_tab_close_pointer_clicked",
    ] {
        assert!(
            globals.contains(required) || wiring.contains(required),
            "missing `{required}`"
        );
    }
    assert!(document_tab_sync.contains("self.workbench_window_bridge.layout_frames()"));
    assert!(document_tab_sync
        .contains("build_host_document_tab_pointer_layout_with_workbench_layout_frames("));
    assert!(!document_tab_sync.contains("self.template_bridge.root_shell_frames()"));
}

#[test]
fn document_tab_pointer_bridge_uses_route_intent_only() {
    let bridge =
        source("src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge.rs");
    let rebuild = source(
        "src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge_rebuild_surface.rs",
    );
    let dispatch = source(
        "src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge_dispatch_event.rs",
    );

    assert!(bridge.contains("route_intents: EditorRouteIntentMap"));
    assert!(rebuild.contains("EditorRouteIntent::DocumentTab"));
    assert!(rebuild.contains("route_intents.bind_node"));
    assert!(dispatch.contains("document_tab_route_for_pointer_dispatch"));
    for forbidden in [
        "targets:",
        "HostDocumentTabPointerTarget",
        "handled_by",
        "route.target",
    ] {
        assert!(
            !bridge.contains(forbidden)
                && !rebuild.contains(forbidden)
                && !dispatch.contains(forbidden),
            "document tab pointer bridge should not keep old hit target marker `{forbidden}`"
        );
    }
}

#[test]
fn document_tab_pointer_rebuild_borrows_measured_frames() {
    let rebuild = source(
        "src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge_rebuild_surface.rs",
    );

    assert!(
        !rebuild.contains(".cloned()"),
        "surface rebuild must borrow measured frames instead of cloning the complete frame vector"
    );
}

#[test]
fn repeated_document_tab_measurement_is_a_no_op() {
    let update = source(
        "src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge_update_measured_frame.rs",
    );

    assert!(
        update.contains("frames[item_index] == Some(measured_frame)"),
        "repeated tab pointer callbacks must not rebuild an unchanged measured frame"
    );
}
