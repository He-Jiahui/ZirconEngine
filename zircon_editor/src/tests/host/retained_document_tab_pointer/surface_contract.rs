fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_document_tab_surfaces_use_rust_owned_pointer_callbacks() {
    let globals = source("src/ui/retained_host/host_contract/globals.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring.rs");
    let pointer_layout = source("src/ui/retained_host/app/pointer_layout.rs");
    let document_tab_sync = pointer_layout
        .split("pub(super) fn sync_document_tab_pointer_layout")
        .nth(1)
        .and_then(|source| {
            source
                .split(
                    "pub(super) fn sync_drawer_header_pointer_layout_with_workbench_layout_frames",
                )
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
