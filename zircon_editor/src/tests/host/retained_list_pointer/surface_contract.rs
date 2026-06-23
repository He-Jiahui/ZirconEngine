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
fn hierarchy_pointer_bridge_uses_route_intent_only() {
    let bridge = source("src/ui/retained_host/hierarchy_pointer/hierarchy_pointer_bridge.rs");
    let rebuild = source("src/ui/retained_host/hierarchy_pointer/rebuild_surface.rs");
    let dispatch = source("src/ui/retained_host/hierarchy_pointer/dispatch_event.rs");

    assert!(bridge.contains("route_intents: EditorRouteIntentMap"));
    assert!(rebuild.contains("EditorRouteIntent::Hierarchy"));
    assert!(rebuild.contains("route_intents.bind_node"));
    assert!(dispatch.contains("hierarchy_route_for_pointer_dispatch"));
    for forbidden in [
        "targets:",
        "HierarchyPointerTarget",
        "handled_by",
        "route.target",
    ] {
        assert!(
            !bridge.contains(forbidden)
                && !rebuild.contains(forbidden)
                && !dispatch.contains(forbidden),
            "hierarchy pointer bridge should not keep old hit target marker `{forbidden}`"
        );
    }
}

#[test]
fn welcome_recent_pointer_bridge_uses_route_intent_only() {
    let bridge =
        source("src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge.rs");
    let rebuild = source(
        "src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_rebuild_surface.rs",
    );
    let dispatch = source(
        "src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_dispatch_event.rs",
    );

    assert!(bridge.contains("route_intents: EditorRouteIntentMap"));
    assert!(rebuild.contains("EditorRouteIntent::WelcomeRecent"));
    assert!(rebuild.contains("route_intents.bind_node"));
    assert!(dispatch.contains("welcome_recent_route_for_pointer_dispatch"));
    assert!(rebuild.contains("WelcomeRecentPointerRouteIntent"));
    for forbidden in [
        "targets:",
        "WelcomeRecentPointerTarget",
        "handled_by",
        "route.target",
    ] {
        assert!(
            !bridge.contains(forbidden)
                && !rebuild.contains(forbidden)
                && !dispatch.contains(forbidden),
            "welcome recent pointer bridge should not keep old hit target marker `{forbidden}`"
        );
    }
}
