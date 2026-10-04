fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_activity_rail_surfaces_use_rust_callbacks_and_toml_projection() {
    let globals = source("src/ui/retained_host/host_contract/globals/ui_context.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring/host_shell/menu.rs");
    let pointer_layout = source("src/ui/retained_host/app/pointer_layout/shell_chrome.rs");
    let activity_rail_sync = pointer_layout
        .split("fn sync_activity_rail_pointer_layout")
        .nth(1)
        .and_then(|source| source.split("fn sync_host_page_pointer_layout").next())
        .expect("activity rail pointer sync function should exist");
    let activity_rail_layout = source(
        "src/ui/retained_host/activity_rail_pointer/build_host_activity_rail_pointer_layout.rs",
    );
    let chrome_projection =
        source("src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs");
    let activity_asset = source("assets/ui/editor/workbench_activity_rail.zui");

    assert!(globals.contains("on_activity_rail_pointer_clicked"));
    assert!(wiring.contains("host_shell.on_activity_rail_pointer_clicked("));
    assert!(activity_rail_sync
        .contains("build_host_activity_rail_pointer_layout_with_workbench_layout_frames("));
    assert!(activity_rail_sync.contains("self.workbench_window_bridge.layout_frames()"));
    assert!(!activity_rail_sync.contains("self.template_bridge.root_shell_frames()"));
    assert!(activity_rail_sync.contains("workbench_layout_frames,"));
    assert!(!pointer_layout.contains("root_shell_frames_with_componentized_drawers();"));
    assert!(activity_rail_layout.contains(".activity_rail_frame"));
    assert!(activity_rail_layout.contains(".drawer_shell_frame(ShellRegionId::Right)"));
    assert!(!activity_rail_layout.contains("resolve_root_activity_rail_frame"));
    assert!(!pointer_layout.contains("workbench_layout_frames.left_region_frame"));
    for required in [
        "activity_rail_nodes",
        "activity_rail_button_frames",
        "activity_rail_active_control_id",
    ] {
        assert!(chrome_projection.contains(required), "missing `{required}`");
    }
    for required in [
        "ActivityRailPanel",
        "ActivityRailButton0",
        "ActivityRailButton1",
    ] {
        assert!(activity_asset.contains(required), "missing `{required}`");
    }
}

#[test]
fn activity_rail_pointer_bridge_uses_route_intent_only() {
    let bridge =
        source("src/ui/retained_host/activity_rail_pointer/host_activity_rail_pointer_bridge.rs");
    let rebuild = source("src/ui/retained_host/activity_rail_pointer/rebuild_surface.rs");
    let insert_strip = source("src/ui/retained_host/activity_rail_pointer/insert_strip.rs");
    let dispatch = source("src/ui/retained_host/activity_rail_pointer/dispatch_event.rs");

    assert!(bridge.contains("route_intents: EditorRouteIntentMap"));
    assert!(insert_strip.contains("EditorRouteIntent::ActivityRail"));
    assert!(insert_strip.contains("route_intents.bind_node"));
    assert!(dispatch.contains("activity_rail_route_for_pointer_dispatch"));
    for forbidden in [
        "targets:",
        "HostActivityRailPointerTarget",
        "handled_by",
        "route.target",
    ] {
        assert!(
            !bridge.contains(forbidden)
                && !rebuild.contains(forbidden)
                && !insert_strip.contains(forbidden)
                && !dispatch.contains(forbidden),
            "activity rail pointer bridge should not keep old hit target marker `{forbidden}`"
        );
    }
}

#[test]
fn activity_rail_pointer_bridge_publishes_retained_geometry_deltas() {
    let sync = source("src/ui/retained_host/activity_rail_pointer/sync.rs");
    let rebuild = source("src/ui/retained_host/activity_rail_pointer/rebuild_surface.rs");

    assert!(sync.contains("surface_delta_for_layout"));
    assert!(rebuild.contains("publish_authored_geometry("));
    assert!(rebuild.contains("rebuild_authored_frames("));
    assert!(!rebuild.contains("surface.rebuild();"));
}

#[test]
fn window_metrics_activity_rail_sync_uses_geometry_only_semantics() {
    let shell_chrome = source("src/ui/retained_host/app/pointer_layout/shell_chrome.rs");
    let recompute = source("src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs");
    let layout = source(
        "src/ui/retained_host/activity_rail_pointer/build_host_activity_rail_pointer_layout.rs",
    );

    assert!(recompute.contains("window_metrics_target"));
    assert!(shell_chrome.contains("sync_activity_rail_pointer_layout_for_target"));
    assert!(shell_chrome.contains("build_host_activity_rail_pointer_geometry_layout("));
    assert!(layout.contains("Arc::clone(&previous.left_tabs)"));
    assert!(layout.contains("Arc::clone(&previous.right_tabs)"));
}

#[test]
fn window_metrics_recompute_reuses_geometry_free_pointer_receipts() {
    let recompute = source("src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs");
    let reuse_gate = recompute
        .split("if !window_metrics_target {")
        .nth(1)
        .and_then(|source| source.split("} else {").next())
        .expect("WindowMetrics semantic pointer receipt reuse gate");

    assert!(reuse_gate.contains("sync_host_page_pointer_layout(model)"));
    assert!(reuse_gate.contains("sync_document_tab_pointer_layout(model)"));
    assert!(reuse_gate.contains("sync_drawer_header_pointer_layout(model)"));
    assert!(recompute.contains("ui.window_metrics.semantic_pointer_receipt_reuse_count"));
}

#[test]
fn window_metrics_recompute_uses_welcome_recent_viewport_fast_path() {
    let pointer_surfaces =
        source("src/ui/retained_host/app/host_lifecycle/recompute/pointer_surfaces.rs");
    let welcome = source("src/ui/retained_host/app/pointer_layout/welcome_recent.rs");

    assert!(pointer_surfaces.contains("sync_welcome_recent_pointer_size()"));
    assert!(pointer_surfaces.contains("ui.window_metrics.welcome_recent_semantic_reuse_count"));
    assert!(welcome.contains("fn sync_welcome_recent_pointer_size"));
    assert!(pointer_surfaces.contains("if window_metrics_target {"));
}
