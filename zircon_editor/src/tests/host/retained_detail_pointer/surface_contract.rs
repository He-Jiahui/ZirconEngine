fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_detail_scroll_surfaces_keep_scroll_authority_in_rust() {
    let globals = [
        "src/ui/retained_host/host_contract/globals/pane_context/callbacks.rs",
        "src/ui/retained_host/host_contract/globals/pane_context/setters/interaction.rs",
    ]
    .into_iter()
    .map(source)
    .collect::<Vec<_>>()
    .join("\n");
    let pointer_layout = [
        "src/ui/retained_host/app/pointer_layout/hierarchy.rs",
        "src/ui/retained_host/app/pointer_layout/detail_scrolls.rs",
    ]
    .into_iter()
    .map(source)
    .collect::<Vec<_>>()
    .join("\n");

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
        assert!(
            globals.contains(required),
            "host contract missing `{required}`"
        );
    }
    for required in [
        "sync_detail_pointer_layouts",
        "sync_console_pointer_layout",
        "sync_inspector_pointer_layout",
        "sync_browser_asset_details_pointer_layout",
    ] {
        assert!(
            pointer_layout.contains(required),
            "pointer layout missing `{required}`"
        );
    }
}

#[test]
fn detail_scroll_pointer_bridge_uses_direct_scalar_routing_only() {
    let bridge = source("src/ui/retained_host/detail_pointer/scroll_surface_pointer_bridge.rs");
    let scroll = source("src/ui/retained_host/detail_pointer/handle_scroll.rs");

    for forbidden in [
        "UiSurface",
        "UiPointerDispatcher",
        "EditorRouteIntentMap",
        "route_intents",
    ] {
        assert!(
            !bridge.contains(forbidden),
            "detail scroll pointer bridge should not keep mirror marker `{forbidden}`"
        );
    }
    assert!(scroll.contains("viewport_frame(&self.layout).contains_point(point)"));
    assert!(scroll.contains("self.clamp_scroll_offset()"));
    assert!(scroll.contains("changed:"));
}

#[test]
fn detail_scroll_offset_has_no_pointer_surface_to_recreate() {
    let scroll = source("src/ui/retained_host/detail_pointer/handle_scroll.rs");

    assert!(!scroll.contains("self.rebuild_surface()"));
    assert!(!scroll.contains("dispatch_pointer_event"));
}

#[test]
fn asset_details_extent_does_not_allocate_a_temporary_section_list() {
    let extent = source("src/ui/retained_host/detail_pointer/asset_details_content_extent.rs");

    assert!(
        !extent.contains("vec!["),
        "fixed asset-details sections should be summed without a temporary heap allocation"
    );
}
