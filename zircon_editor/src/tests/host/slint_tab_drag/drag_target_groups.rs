use super::support::*;

#[test]
fn shared_drag_target_route_prefers_right_over_bottom_in_overlap_when_pointer_is_closer_to_right_edge(
) {
    let geometry = shell_geometry(
        ShellFrame::new(1348.0, 50.0, 0.0, 666.0),
        ShellFrame::new(34.0, 50.0, 1314.0, 666.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 0.0),
    );

    assert_eq!(
        resolve_workbench_drag_target_group(
            UiSize::new(1440.0, 900.0),
            &geometry,
            true,
            UiPoint::new(1428.0, 860.0),
        ),
        Some(WorkbenchDragTargetGroup::Right)
    );
}

#[test]
fn shared_drag_target_route_prefers_bottom_over_right_in_overlap_when_pointer_is_closer_to_bottom_edge(
) {
    let geometry = shell_geometry(
        ShellFrame::new(1348.0, 50.0, 0.0, 666.0),
        ShellFrame::new(34.0, 50.0, 1314.0, 666.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 0.0),
    );

    assert_eq!(
        resolve_workbench_drag_target_group(
            UiSize::new(1440.0, 900.0),
            &geometry,
            true,
            UiPoint::new(1380.0, 860.0),
        ),
        Some(WorkbenchDragTargetGroup::Bottom)
    );
}

#[test]
fn shared_drag_target_route_returns_document_inside_document_region() {
    let geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );

    assert_eq!(
        resolve_workbench_drag_target_group(
            UiSize::new(1440.0, 900.0),
            &geometry,
            true,
            UiPoint::new(720.0, 240.0),
        ),
        Some(WorkbenchDragTargetGroup::Document)
    );
}

#[test]
fn resolve_workbench_drag_target_group_with_root_frames_uses_shared_root_projection_document_bounds_when_drawers_are_collapsed(
) {
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 50.0, 1440.0, 650.0),
        status_bar_frame: ShellFrame::new(0.0, 700.0, 1440.0, 20.0),
        region_frames: BTreeMap::from([
            (ShellRegionId::Left, ShellFrame::new(0.0, 50.0, 0.0, 650.0)),
            (
                ShellRegionId::Document,
                ShellFrame::new(21.0, 50.0, 1419.0, 650.0),
            ),
            (
                ShellRegionId::Right,
                ShellFrame::new(1440.0, 50.0, 0.0, 650.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(0.0, 700.0, 1440.0, 0.0),
            ),
        ]),
        splitter_frames: BTreeMap::new(),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::default(),
    };
    let root_projection = BuiltinWorkbenchRootShellFrames {
        workbench_body_frame: Some(UiFrame::new(0.0, 40.0, 1440.0, 656.0)),
        document_host_frame: Some(UiFrame::new(56.0, 40.0, 1384.0, 656.0)),
        status_bar_frame: Some(UiFrame::new(0.0, 696.0, 1440.0, 24.0)),
        ..BuiltinWorkbenchRootShellFrames::default()
    };

    assert_eq!(
        resolve_workbench_drag_target_group_with_root_frames(
            UiSize::new(1440.0, 720.0),
            &geometry,
            false,
            UiPoint::new(40.0, 240.0),
            Some(&root_projection),
        ),
        None
    );
    assert_eq!(
        resolve_workbench_drag_target_group_with_root_frames(
            UiSize::new(1440.0, 720.0),
            &geometry,
            false,
            UiPoint::new(80.0, 240.0),
            Some(&root_projection),
        ),
        Some(WorkbenchDragTargetGroup::Document)
    );
}

#[test]
fn resolve_workbench_drag_target_group_with_root_frames_prefers_shared_left_drawer_shell_width_when_legacy_geometry_is_stale(
) {
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 50.0, 1440.0, 650.0),
        status_bar_frame: ShellFrame::new(0.0, 700.0, 1440.0, 20.0),
        region_frames: BTreeMap::from([
            (ShellRegionId::Left, ShellFrame::new(0.0, 50.0, 24.0, 650.0)),
            (
                ShellRegionId::Document,
                ShellFrame::new(24.0, 50.0, 1416.0, 650.0),
            ),
            (
                ShellRegionId::Right,
                ShellFrame::new(1440.0, 50.0, 0.0, 650.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(0.0, 700.0, 1440.0, 0.0),
            ),
        ]),
        splitter_frames: BTreeMap::new(),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::default(),
    };
    let root_projection = BuiltinWorkbenchRootShellFrames {
        workbench_body_frame: Some(UiFrame::new(0.0, 40.0, 1440.0, 656.0)),
        left_drawer_shell_frame: Some(UiFrame::new(0.0, 40.0, 312.0, 656.0)),
        document_host_frame: Some(UiFrame::new(313.0, 40.0, 1127.0, 656.0)),
        status_bar_frame: Some(UiFrame::new(0.0, 696.0, 1440.0, 24.0)),
        ..BuiltinWorkbenchRootShellFrames::default()
    };

    assert_eq!(
        resolve_workbench_drag_target_group_with_root_frames(
            UiSize::new(1440.0, 720.0),
            &geometry,
            true,
            UiPoint::new(180.0, 240.0),
            Some(&root_projection),
        ),
        Some(WorkbenchDragTargetGroup::Left)
    );
}

#[test]
fn shared_drag_target_route_disables_empty_tool_regions_when_drawers_are_hidden() {
    let geometry = shell_geometry(
        ShellFrame::new(1348.0, 50.0, 0.0, 666.0),
        ShellFrame::new(34.0, 50.0, 1314.0, 666.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 0.0),
    );

    assert_eq!(
        resolve_workbench_drag_target_group(
            UiSize::new(1440.0, 900.0),
            &geometry,
            false,
            UiPoint::new(12.0, 240.0),
        ),
        None
    );
}
