fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_list_surfaces_route_through_pane_surface_host_context() {
    let globals = source("src/ui/slint_host/host_contract/globals.rs");
    let wiring = source("src/ui/slint_host/app/callback_wiring.rs");
    let pointer_layout = source("src/ui/slint_host/app/pointer_layout.rs");

    for required in [
        "on_hierarchy_pointer_clicked",
        "on_asset_tree_pointer_clicked",
        "on_asset_content_pointer_clicked",
        "on_asset_reference_pointer_clicked",
        "on_welcome_recent_pointer_clicked",
    ] {
        assert!(globals.contains(required), "host globals missing `{required}`");
    }
    for required in [
        "pane_surface_host.on_hierarchy_pointer_clicked(",
        "pane_surface_host.on_asset_tree_pointer_clicked(",
        "pane_surface_host.on_asset_content_pointer_clicked(",
        "pane_surface_host.on_asset_reference_pointer_clicked(",
        "pane_surface_host.on_welcome_recent_pointer_clicked(",
    ] {
        assert!(wiring.contains(required), "callback wiring missing `{required}`");
    }
    assert!(pointer_layout.contains("sync_asset_pointer_layouts"));
    assert!(pointer_layout.contains("sync_welcome_recent_pointer_layout"));
}
