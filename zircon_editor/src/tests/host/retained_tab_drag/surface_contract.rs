fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_drag_capture_surface_uses_rust_owned_pointer_event_contract() {
    let globals = source("src/ui/retained_host/host_contract/globals/ui_context.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring/host_shell/drag_resize.rs");
    let docking = source("src/ui/retained_host/app/workspace_docking.rs");

    assert!(globals.contains("on_host_drag_pointer_event"));
    assert!(wiring.contains("host_shell.on_host_drag_pointer_event("));
    for required in [
        "pub(super) fn host_drag_pointer_event",
        "sync_drag_target_group",
        "dispatch_drag_drop_from_pointer",
        "HOST_POINTER_UP",
    ] {
        assert!(
            docking.contains(required),
            "workspace docking missing `{required}`"
        );
    }
    for legacy in ["on_drop_tab", "on_update_drag_target"] {
        assert!(
            !wiring.contains(legacy),
            "drag wiring should not keep `{legacy}`"
        );
    }
}

#[test]
fn host_document_tab_drop_uses_componentized_workbench_layout_frames() {
    let docking = source("src/ui/retained_host/app/workspace_docking.rs");
    let docking_route = source("src/ui/retained_host/app/workspace_docking/drag_drop/route.rs");
    let tab_drag = source("src/ui/retained_host/tab_drag.rs");
    let route_resolution = source("src/ui/retained_host/tab_drag/route_resolution.rs");
    let strip_hitbox = source("src/ui/retained_host/tab_drag/strip_hitbox.rs");

    assert!(docking_route.contains("resolve_host_tab_drop_route_with_workbench_layout_frames("));
    assert!(docking_route.contains("self.workbench_window_bridge.layout_frames()"));
    assert!(!docking.contains("self.template_bridge.root_shell_frames()"));
    assert!(!docking_route.contains("self.template_bridge.root_shell_frames()"));
    assert!(!docking.contains("root_shell_frames_with_componentized_drawers();"));
    assert!(!docking_route.contains("root_shell_frames_with_componentized_drawers();"));
    assert!(tab_drag.contains("resolve_host_tab_drop_route_with_workbench_layout_frames"));
    assert!(route_resolution
        .contains("componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames"));
    assert!(strip_hitbox.contains("precise_drop_target_with_workbench_layout_frames"));
    assert!(strip_hitbox.contains("document_tabs_frame"));
    assert!(strip_hitbox.contains("document_region_frame"));
    assert!(strip_hitbox.contains("center_band_frame"));
    assert!(!strip_hitbox.contains("resolve_root_document_region_frame"));
    assert!(!strip_hitbox.contains("resolve_root_center_band_frame"));
}

#[test]
fn host_shell_drag_uses_componentized_workbench_layout_frames() {
    let componentized_window = source(
        "src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs",
    );
    let recompute_viewport =
        source("src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs");
    let shell_template_bridges =
        source("src/ui/retained_host/app/host_lifecycle/recompute/shell/template_bridges.rs");
    let helpers = source("src/ui/retained_host/app/helpers.rs");
    let drawer_layout =
        source("src/ui/retained_host/callback_dispatch/template_bridge/workbench/drawer_layout.rs");
    let shell_pointer_bridge = source("src/ui/retained_host/shell_pointer/bridge.rs");
    let drag_surface = source("src/ui/retained_host/shell_pointer/drag_surface.rs");

    assert!(componentized_window.contains("layout_frames(&self)"));
    assert!(componentized_window.contains("BuiltinWorkbenchWindowLayoutFrames"));
    assert!(componentized_window.contains("EditorWorkbenchTemplateControlIds::VIEWPORT"));
    assert!(shell_template_bridges.contains("self.workbench_window_bridge.layout_frames()"));
    assert!(!shell_template_bridges.contains("root_shell_frames_with_componentized_drawers();"));
    assert!(!helpers.contains("root_shell_frames_with_componentized_drawers"));
    assert!(!drawer_layout.contains("merge_drawer_frames_into_root_shell_frames"));
    assert!(recompute_viewport.contains("update_layout_with_workbench_layout_frames("));
    let normalized_lifecycle = recompute_viewport.replace("\r\n", "\n");
    assert!(!normalized_lifecycle.contains(
        "Some(&root_shell_frames),\n                    componentized_workbench_layout_frames"
    ));
    assert!(shell_pointer_bridge
        .contains("componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames"));
    assert!(drag_surface
        .contains("componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames"));
    assert!(drag_surface.contains("center_band_frame"));
    assert!(drag_surface.contains("status_bar_frame"));
    assert!(drag_surface.contains("document_region_frame"));
    assert!(!drag_surface.contains("shared_root_frames"));
    assert!(!drag_surface.contains("resolve_root_center_band_frame"));
    assert!(!drag_surface.contains("resolve_root_document_region_frame"));
    assert!(!drag_surface.contains("resolve_root_status_bar_frame"));
    assert!(!drag_surface.contains("resolve_direct_document_host_frame"));
}

#[test]
fn tab_drag_bridge_uses_shell_pointer_route_intent_resolution() {
    let tab_drag_bridge = source("src/ui/retained_host/tab_drag/bridge.rs");
    let shell_pointer_bridge = source("src/ui/retained_host/shell_pointer/bridge.rs");
    let drag_surface = source("src/ui/retained_host/shell_pointer/drag_surface.rs");

    assert!(tab_drag_bridge.contains("HostShellPointerBridge"));
    assert!(tab_drag_bridge.contains("self.shell_pointer.drag_target_at(point)"));
    assert!(shell_pointer_bridge.contains("drag_route_intents: EditorRouteIntentMap"));
    assert!(drag_surface.contains("EditorRouteIntent::ShellPointer"));
    assert!(!tab_drag_bridge.contains("dispatch_pointer_event("));
    assert!(!tab_drag_bridge.contains("HitTable"));
    assert!(!tab_drag_bridge.contains("hit_table"));
}
