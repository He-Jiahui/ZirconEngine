fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_resize_surface_uses_rust_owned_pointer_event_contract() {
    let globals = source("src/ui/retained_host/host_contract/globals/ui_context.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring/host_shell/drag_resize.rs");
    let docking = source("src/ui/retained_host/app/workspace_docking.rs");
    let resize_capture =
        source("src/ui/retained_host/app/workspace_docking/drawer_resize/capture.rs");
    let resize_movement =
        source("src/ui/retained_host/app/workspace_docking/drawer_resize/movement.rs");
    let resize_surface = source("src/ui/retained_host/shell_pointer/resize_surface.rs");
    let shell_pointer_bridge = source("src/ui/retained_host/shell_pointer/bridge.rs");

    assert!(globals.contains("on_host_resize_pointer_event"));
    assert!(wiring.contains("host_shell.on_host_resize_pointer_event("));
    assert!(resize_surface.contains("componentized_workbench_layout_frames"));
    assert!(resize_surface.contains("resize_splitter_frame("));
    assert!(resize_capture.contains("self.workbench_window_bridge.layout_frames()"));
    assert!(resize_capture.contains(".drawer_shell_frame(region)"));
    assert!(!resize_capture.contains("root_shell_frames_with_componentized_drawers();"));
    assert!(!resize_surface.contains("resolve_root_left_splitter_frame"));
    assert!(!resize_surface.contains("resolve_root_right_splitter_frame"));
    assert!(!resize_surface.contains("resolve_root_bottom_splitter_frame"));
    assert!(!resize_surface.contains(".splitter_frame("));
    assert!(shell_pointer_bridge.contains("update_resize_surface("));
    assert!(shell_pointer_bridge.contains("componentized_workbench_layout_frames,"));
    assert!(!shell_pointer_bridge.contains(
        "update_resize_surface(\n            &mut self.resize_surface,\n            root_size,\n            geometry,"
    ));
    assert!(!shell_pointer_bridge.contains(
        "update_resize_surface(\n            &mut self.resize_surface,\n            root_size,\n            shared_root_frames,"
    ));
    assert!(docking.contains("pub(super) fn host_resize_pointer_event"));
    assert!(docking.contains("active.source_window.as_ref() == source_window_id"));
    for required in [
        "self.begin_drawer_resize_capture(source_window_id, x, y)",
        "self.update_drawer_resize_capture(source_window_id, x, y)",
        "self.finish_drawer_resize_capture(source_window_id, x, y)",
        "self.cancel_drawer_resize_capture(source_window_id)",
    ] {
        assert!(
            docking.contains(required),
            "workspace docking should route `{required}` through its pointer handler"
        );
    }
    assert!(resize_capture.contains("fn begin_drawer_resize_capture("));
    assert!(resize_movement.contains("fn update_drawer_resize_capture("));
    assert!(resize_movement.contains("fn finish_drawer_resize_capture("));
    for legacy in [
        "on_begin_drawer_resize",
        "on_update_drawer_resize",
        "on_finish_drawer_resize",
    ] {
        assert!(
            !wiring.contains(legacy),
            "resize wiring should not keep `{legacy}`"
        );
    }
}

#[test]
fn shell_pointer_bridge_uses_route_intent_only() {
    let retained_host_mod = source("src/ui/retained_host/mod.rs");
    let route_intent_map = source("src/ui/retained_host/route_intent/map.rs");
    let shell_pointer_bridge = source("src/ui/retained_host/shell_pointer/bridge.rs");
    let shell_pointer_route = source("src/ui/retained_host/shell_pointer/route.rs");
    let drag_surface = source("src/ui/retained_host/shell_pointer/drag_surface.rs");
    let resize_surface = source("src/ui/retained_host/shell_pointer/resize_surface.rs");

    assert!(retained_host_mod.contains("pub(crate) mod route_intent;"));
    assert!(route_intent_map.contains("pub(crate) struct EditorRouteIntentMap"));
    assert!(route_intent_map.contains("route_id_for_node"));
    assert!(route_intent_map.contains("shell_pointer_route_for_node"));
    assert!(shell_pointer_bridge.contains("EditorRouteIntentMap"));
    assert!(shell_pointer_bridge.contains("shell_pointer_route_from_input_result"));
    assert!(shell_pointer_bridge.contains("shell_pointer_reply_effect_target"));
    assert!(drag_surface.contains("EditorRouteIntent::ShellPointer"));
    assert!(resize_surface.contains("EditorRouteIntent::ShellPointer"));
    assert!(!shell_pointer_route.contains("drag_route_from_node"));
    assert!(!shell_pointer_route.contains("resize_group_from_dispatch"));
    assert!(!shell_pointer_bridge.contains("handled_by"));
    assert!(!shell_pointer_bridge.contains("captured_by"));
}

#[test]
fn shell_drag_target_frames_are_immutable_and_lock_free() {
    let drag_surface = source("src/ui/retained_host/shell_pointer/drag_surface.rs");
    let drag_frames = source("src/ui/retained_host/shell_pointer/drag_frames.rs");
    let effects = source("src/ui/retained_host/shell_pointer/effects.rs");

    assert!(drag_frames.contains("struct DragTargetFrames"));
    assert!(drag_frames.contains("struct DragHitGeometry"));
    assert!(drag_surface.contains("ArcSwap::from_pointee(resolved_geometry)"));
    assert!(drag_surface.contains("hit_geometry.store(Arc::new(next_geometry))"));
    assert!(drag_surface.contains("let geometry = left_geometry.load();"));
    for candidate in [&drag_surface, &drag_frames, &effects] {
        assert!(!candidate.contains("Mutex"));
        assert!(!candidate.contains(".lock()"));
    }
}

#[test]
fn unchanged_resize_geometry_does_not_rebuild_the_surface() {
    let common = source("src/ui/retained_host/shell_pointer/common.rs");
    let resize = source("src/ui/retained_host/shell_pointer/resize_surface.rs");

    assert!(common.contains(") -> bool"));
    assert!(resize.contains("let mut changed = false;"));
    assert!(resize.contains("changed |= update_target_node("));
    assert!(resize.contains("if changed {"));
}

#[test]
fn drawer_resize_capture_goes_through_reply() {
    let shell_pointer_bridge = source("src/ui/retained_host/shell_pointer/bridge.rs");
    let resize_surface = source("src/ui/retained_host/shell_pointer/resize_surface.rs");
    let resize_dispatch = shell_pointer_bridge
        .split("fn dispatch_resize_event(")
        .nth(1)
        .and_then(|tail| tail.split("fn pointer_input_event(").next())
        .expect("shell pointer bridge should contain resize dispatch");

    assert!(resize_dispatch.contains("dispatch_input_event_with_diagnostics_mode("));
    assert!(resize_dispatch.contains("UiInputDiagnosticsMode::Summary"));
    assert!(shell_pointer_bridge.contains("UiDispatchEffect::CapturePointer"));
    assert!(shell_pointer_bridge.contains("UiDispatchEffect::ReleasePointerCapture"));
    assert!(shell_pointer_bridge.contains("shell_pointer_reply_effect_target(result)"));
    assert!(!shell_pointer_bridge.contains("dispatch_pointer_event("));
    assert!(resize_surface.contains("UiPointerDispatchEffect::capture()"));
}
