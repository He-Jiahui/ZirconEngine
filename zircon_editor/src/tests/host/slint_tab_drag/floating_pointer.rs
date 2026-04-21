use super::support::*;

#[test]
fn workbench_shell_pointer_route_group_key_normalizes_document_and_floating_routes() {
    let window_id = MainPageId::new("window:preview");

    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::DragTarget(
            WorkbenchDragTargetGroup::Right,
        )),
        Some("right".to_string())
    );
    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::DocumentEdge(
            DockEdge::Bottom,
        )),
        Some("document-bottom".to_string())
    );
    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::FloatingWindow(
            window_id.clone(),
        )),
        Some(floating_window_group_key(&window_id))
    );
    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::FloatingWindowEdge {
            window_id: window_id.clone(),
            edge: DockEdge::Left,
        }),
        Some(floating_window_edge_group_key(&window_id, DockEdge::Left))
    );
}

#[test]
fn shared_shell_pointer_route_reports_floating_window_attach_from_shared_surface() {
    let window_id = MainPageId::new("window:preview");
    let mut geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );
    geometry.floating_window_frames.insert(
        window_id.clone(),
        ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    );
    let floating_windows = vec![floating_window(
        window_id.clone(),
        "Preview Popout",
        Vec::new(),
        None,
    )];
    let mut bridge = WorkbenchShellPointerBridge::new();
    bridge.update_layout_with_floating_windows(
        UiSize::new(1440.0, 900.0),
        &geometry,
        false,
        &floating_windows,
        &[],
    );

    assert_eq!(
        bridge.drag_route_at(UiPoint::new(600.0, 300.0)),
        Some(WorkbenchShellPointerRoute::FloatingWindow(window_id))
    );
    assert_eq!(
        bridge.drag_target_at(UiPoint::new(600.0, 300.0)),
        Some(WorkbenchDragTargetGroup::Document)
    );
}

#[test]
fn shared_shell_pointer_route_does_not_fall_back_to_legacy_geometry_when_projection_bundle_is_explicitly_provided_but_missing_window(
) {
    let window_id = MainPageId::new("window:preview");
    let mut geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );
    geometry.floating_window_frames.insert(
        window_id.clone(),
        ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    );
    let floating_windows = vec![floating_window(
        window_id.clone(),
        "Preview Popout",
        Vec::new(),
        None,
    )];
    let mut bridge = WorkbenchShellPointerBridge::new();
    let empty_bundle = FloatingWindowProjectionBundle::default();
    bridge.update_layout_with_root_shell_frames(
        UiSize::new(1440.0, 900.0),
        &geometry,
        false,
        &floating_windows,
        None,
        Some(&empty_bundle),
    );

    assert_eq!(
        bridge.drag_route_at(UiPoint::new(600.0, 300.0)),
        None,
        "once a floating-window projection bundle is supplied, drag routing should not revive stale geometry frames"
    );
}

#[test]
fn shared_shell_pointer_route_prefers_native_window_host_bounds_for_floating_attach_surface() {
    let window_id = MainPageId::new("window:preview");
    let mut geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );
    geometry.floating_window_frames.insert(
        window_id.clone(),
        ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    );
    let floating_windows = vec![floating_window(
        window_id.clone(),
        "Preview Popout",
        Vec::new(),
        None,
    )];
    let mut bridge = WorkbenchShellPointerBridge::new();
    bridge.update_layout_with_native_window_hosts(
        UiSize::new(1440.0, 900.0),
        &geometry,
        false,
        &floating_windows,
        None,
        &[NativeWindowHostState {
            window_id: window_id.clone(),
            handle: None,
            bounds: [640.0, 320.0, 700.0, 420.0],
        }],
    );

    assert_eq!(
        bridge.drag_route_at(UiPoint::new(900.0, 500.0)),
        Some(WorkbenchShellPointerRoute::FloatingWindow(window_id)),
        "drag attach surface should move to native host bounds instead of stale layout geometry"
    );
}

#[test]
fn shared_shell_pointer_route_reports_floating_window_edge_from_shared_surface() {
    let window_id = MainPageId::new("window:preview");
    let mut geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );
    geometry.floating_window_frames.insert(
        window_id.clone(),
        ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    );
    let floating_windows = vec![floating_window(
        window_id.clone(),
        "Preview Popout",
        Vec::new(),
        None,
    )];
    let mut bridge = WorkbenchShellPointerBridge::new();
    bridge.update_layout_with_floating_windows(
        UiSize::new(1440.0, 900.0),
        &geometry,
        false,
        &floating_windows,
        &[],
    );

    assert_eq!(
        bridge.drag_route_at(UiPoint::new(426.0, 300.0)),
        Some(WorkbenchShellPointerRoute::FloatingWindowEdge {
            window_id,
            edge: DockEdge::Left,
        })
    );
    assert_eq!(
        bridge.drag_target_at(UiPoint::new(426.0, 300.0)),
        Some(WorkbenchDragTargetGroup::Document)
    );
}
