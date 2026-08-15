use super::support::*;

#[test]
fn host_shell_pointer_route_group_key_normalizes_document_and_floating_routes() {
    let window_id = MainPageId::new("window:preview");

    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::DragTarget(
            HostDragTargetGroup::Right,
        )),
        Some("right".to_string())
    );
    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::DocumentEdge(DockEdge::Bottom,)),
        Some("document-bottom".to_string())
    );
    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::FloatingWindow(
            window_id.clone(),
        )),
        Some(floating_window_group_key(&window_id))
    );
    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::FloatingWindowEdge {
            window_id: window_id.clone(),
            edge: DockEdge::Left,
        }),
        Some(floating_window_edge_group_key(&window_id, DockEdge::Left))
    );
}

#[test]
fn shared_shell_pointer_route_reports_floating_window_attach_from_shared_surface() {
    let window_id = MainPageId::new("window:preview");
    let floating_windows = vec![floating_window(
        window_id.clone(),
        "Preview Popout",
        Vec::new(),
        None,
    )];
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_floating_windows(
        UiSize::new(1440.0, 900.0),
        false,
        &floating_windows,
    );

    assert_eq!(
        bridge.drag_route_at(UiPoint::new(600.0, 300.0)),
        Some(HostShellPointerRoute::FloatingWindow(window_id))
    );
    assert_eq!(
        bridge.drag_target_at(UiPoint::new(600.0, 300.0)),
        Some(HostDragTargetGroup::Document)
    );
}

#[test]
fn shared_shell_pointer_route_does_not_fall_back_to_legacy_geometry_when_projection_bundle_is_explicitly_provided_but_missing_window(
) {
    let window_id = MainPageId::new("window:preview");
    let floating_windows = vec![floating_window(
        window_id.clone(),
        "Preview Popout",
        Vec::new(),
        None,
    )];
    let mut bridge = HostShellPointerBridge::new();
    let empty_bundle = FloatingWindowProjectionBundle::default();
    bridge.update_layout_with_root_shell_frames(
        UiSize::new(1440.0, 900.0),
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
    let floating_windows = vec![floating_window(
        window_id.clone(),
        "Preview Popout",
        Vec::new(),
        None,
    )];
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_native_window_hosts(
        UiSize::new(1440.0, 900.0),
        false,
        &floating_windows,
        None,
        &[NativeWindowHostState::new_for_test(
            window_id.clone(),
            None,
            [640.0, 320.0, 700.0, 420.0],
        )],
    );

    assert_eq!(
        bridge.drag_route_at(UiPoint::new(900.0, 500.0)),
        Some(HostShellPointerRoute::FloatingWindow(window_id)),
        "drag attach surface should move to native host bounds instead of stale layout geometry"
    );
}

#[test]
fn stable_floating_window_identity_patches_geometry_without_replacing_authority() {
    let window_id = MainPageId::new("window:preview");
    let floating_windows = vec![floating_window(
        window_id.clone(),
        "Preview Popout",
        Vec::new(),
        None,
    )];
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_native_window_hosts(
        UiSize::new(1440.0, 900.0),
        false,
        &floating_windows,
        None,
        &[NativeWindowHostState::new_for_test(
            window_id.clone(),
            None,
            [420.0, 180.0, 520.0, 360.0],
        )],
    );
    let authority_generation = bridge.drag_surface_authority_generation_for_test();

    bridge.update_layout_with_native_window_hosts(
        UiSize::new(1440.0, 900.0),
        false,
        &floating_windows,
        None,
        &[NativeWindowHostState::new_for_test(
            window_id.clone(),
            None,
            [760.0, 260.0, 520.0, 360.0],
        )],
    );

    assert_eq!(
        bridge.drag_surface_authority_generation_for_test(),
        authority_generation,
        "the same ordered floating-window IDs must reuse drag surface and callback authority"
    );
    assert_eq!(bridge.drag_route_at(UiPoint::new(500.0, 260.0)), None);
    assert_eq!(
        bridge.drag_route_at(UiPoint::new(980.0, 440.0)),
        Some(HostShellPointerRoute::FloatingWindow(window_id.clone()))
    );
    assert_eq!(
        bridge.drag_route_at(UiPoint::new(766.0, 440.0)),
        Some(HostShellPointerRoute::FloatingWindowEdge {
            window_id,
            edge: DockEdge::Left,
        }),
        "retained edge callbacks must read the newly published geometry snapshot"
    );
}

#[test]
fn shared_shell_pointer_route_reports_floating_window_edge_from_shared_surface() {
    let window_id = MainPageId::new("window:preview");
    let floating_windows = vec![floating_window(
        window_id.clone(),
        "Preview Popout",
        Vec::new(),
        None,
    )];
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_floating_windows(
        UiSize::new(1440.0, 900.0),
        false,
        &floating_windows,
    );

    assert_eq!(
        bridge.drag_route_at(UiPoint::new(426.0, 300.0)),
        Some(HostShellPointerRoute::FloatingWindowEdge {
            window_id,
            edge: DockEdge::Left,
        })
    );
    assert_eq!(
        bridge.drag_target_at(UiPoint::new(426.0, 300.0)),
        Some(HostDragTargetGroup::Document)
    );
}
