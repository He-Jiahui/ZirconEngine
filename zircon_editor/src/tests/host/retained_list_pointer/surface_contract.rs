fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_list_surfaces_route_through_pane_surface_host_context() {
    let globals = source("src/ui/retained_host/host_contract/globals/pane_context/callbacks.rs");
    let wiring = [
        "src/ui/retained_host/app/callback_wiring/pane_surface/hierarchy.rs",
        "src/ui/retained_host/app/callback_wiring/pane_surface/assets/tree.rs",
        "src/ui/retained_host/app/callback_wiring/pane_surface/assets/content.rs",
        "src/ui/retained_host/app/callback_wiring/pane_surface/assets/references.rs",
        "src/ui/retained_host/app/callback_wiring/pane_surface/welcome.rs",
    ]
    .into_iter()
    .map(source)
    .collect::<Vec<_>>()
    .join("\n");
    let pointer_layout = [
        "src/ui/retained_host/app/pointer_layout/asset_surfaces/sync.rs",
        "src/ui/retained_host/app/pointer_layout/welcome_recent.rs",
    ]
    .into_iter()
    .map(source)
    .collect::<Vec<_>>()
    .join("\n");

    for required in [
        "on_hierarchy_pointer_clicked",
        "on_asset_tree_pointer_clicked",
        "on_asset_tree_pointer_event",
        "on_asset_content_pointer_clicked",
        "on_asset_reference_pointer_clicked",
        "on_welcome_recent_pointer_clicked",
    ] {
        assert!(
            globals.contains(required),
            "host globals missing `{required}`"
        );
    }
    for required in [
        "pane_surface_host.on_hierarchy_pointer_clicked(",
        "pane_surface_host.on_asset_tree_pointer_clicked(",
        "pane_surface_host.on_asset_tree_pointer_event(",
        "pane_surface_host.on_asset_content_pointer_clicked(",
        "pane_surface_host.on_asset_reference_pointer_clicked(",
        "pane_surface_host.on_welcome_recent_pointer_clicked(",
    ] {
        assert!(
            wiring.contains(required),
            "callback wiring missing `{required}`"
        );
    }
    assert!(pointer_layout.contains("sync_asset_pointer_layouts"));
    assert!(pointer_layout.contains("sync_welcome_recent_pointer_layout"));
}

#[test]
fn hierarchy_pointer_bridge_uses_direct_arithmetic_routing_only() {
    let bridge = source("src/ui/retained_host/hierarchy_pointer/hierarchy_pointer_bridge.rs");
    let route = source("src/ui/retained_host/hierarchy_pointer/route_at_point.rs");

    for forbidden in [
        "UiSurface",
        "UiPointerDispatcher",
        "EditorRouteIntentMap",
        "route_intents",
    ] {
        assert!(
            !bridge.contains(forbidden),
            "hierarchy pointer bridge should not keep mirror hit marker `{forbidden}`"
        );
    }
    assert!(route.contains(".floor() as usize"));
    assert!(route.contains("self.layout.item_count"));
    assert!(!route.contains("node_ids"));
}

#[test]
fn welcome_recent_pointer_bridge_uses_direct_arithmetic_routing_only() {
    let bridge =
        source("src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge.rs");
    let route = source(
        "src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_project_route.rs",
    );

    for forbidden in ["UiSurface", "UiPointerDispatcher", "EditorRouteIntentMap"] {
        assert!(
            !bridge.contains(forbidden),
            "welcome recent pointer bridge should not keep mirror marker `{forbidden}`"
        );
    }
    assert!(route.contains(".floor() as usize"));
    assert!(route.contains("recent_project_paths.len()"));
    assert!(route.contains("action_target_for_route"));
    assert!(!route.contains("path.clone()"));
}
