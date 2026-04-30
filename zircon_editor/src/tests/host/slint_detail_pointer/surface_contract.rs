fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_detail_scroll_surfaces_keep_scroll_authority_in_rust() {
    let globals = source("src/ui/slint_host/host_contract/globals.rs");
    let pointer_layout = source("src/ui/slint_host/app/pointer_layout.rs");

    for required in [
        "set_hierarchy_scroll_px",
        "set_console_scroll_px",
        "set_inspector_scroll_px",
        "set_browser_asset_details_scroll_px",
        "on_hierarchy_pointer_scrolled",
        "on_console_pointer_scrolled",
        "on_inspector_pointer_scrolled",
        "on_browser_asset_details_pointer_scrolled",
    ] {
        assert!(globals.contains(required), "host contract missing `{required}`");
    }
    for required in [
        "sync_detail_pointer_layouts",
        "sync_console_pointer_layout",
        "sync_inspector_pointer_layout",
        "sync_browser_asset_details_pointer_layout",
    ] {
        assert!(pointer_layout.contains(required), "pointer layout missing `{required}`");
    }
}
