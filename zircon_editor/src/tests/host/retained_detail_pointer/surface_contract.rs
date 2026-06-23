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
fn detail_scroll_pointer_bridge_uses_route_intent_only() {
    let bridge = source("src/ui/retained_host/detail_pointer/scroll_surface_pointer_bridge.rs");
    let rebuild = source("src/ui/retained_host/detail_pointer/rebuild_surface.rs");
    let scroll = source("src/ui/retained_host/detail_pointer/handle_scroll.rs");

    assert!(bridge.contains("route_intents: EditorRouteIntentMap"));
    assert!(rebuild.contains("EditorRouteIntent::Detail"));
    assert!(rebuild.contains("route_intents.bind_node"));
    assert!(scroll.contains("detail_route_for_pointer_dispatch"));
    for forbidden in ["map_route", "handled_by", "route.target"] {
        assert!(
            !bridge.contains(forbidden)
                && !rebuild.contains(forbidden)
                && !scroll.contains(forbidden),
            "detail scroll pointer bridge should not keep old route marker `{forbidden}`"
        );
    }
}
