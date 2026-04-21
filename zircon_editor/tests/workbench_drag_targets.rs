use std::collections::BTreeMap;

use zircon_editor::ui::slint_host::tab_drag::{
    resolve_workbench_drag_target_group, WorkbenchDragTargetGroup,
};
use zircon_editor::ui::workbench::autolayout::{
    ShellFrame, ShellRegionId, ShellSizePx, WorkbenchShellGeometry,
};
use zircon_runtime::ui::layout::UiPoint;

fn shell_geometry(
    right_region: ShellFrame,
    document_region: ShellFrame,
    bottom_region: ShellFrame,
) -> WorkbenchShellGeometry {
    WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 50.0, 1440.0, 830.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::from([
            (
                ShellRegionId::Left,
                ShellFrame::new(0.0, 50.0, 320.0, 738.0),
            ),
            (ShellRegionId::Document, document_region),
            (ShellRegionId::Right, right_region),
            (ShellRegionId::Bottom, bottom_region),
        ]),
        splitter_frames: BTreeMap::new(),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::new(0.0, 0.0, 0.0, 0.0),
    }
}

#[test]
fn workbench_drag_target_route_allows_dragging_into_empty_right_region() {
    let geometry = shell_geometry(
        ShellFrame::new(1348.0, 50.0, 0.0, 666.0),
        ShellFrame::new(34.0, 50.0, 1314.0, 666.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 0.0),
    );

    assert_eq!(
        resolve_workbench_drag_target_group(
            ShellSizePx::new(1440.0, 900.0),
            &geometry,
            true,
            UiPoint::new(1428.0, 240.0),
        ),
        Some(WorkbenchDragTargetGroup::Right)
    );
}

#[test]
fn workbench_drag_target_route_allows_dragging_into_empty_bottom_region() {
    let geometry = shell_geometry(
        ShellFrame::new(1348.0, 50.0, 0.0, 666.0),
        ShellFrame::new(34.0, 50.0, 1314.0, 666.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 0.0),
    );

    assert_eq!(
        resolve_workbench_drag_target_group(
            ShellSizePx::new(1440.0, 900.0),
            &geometry,
            true,
            UiPoint::new(720.0, 860.0),
        ),
        Some(WorkbenchDragTargetGroup::Bottom)
    );
}

#[test]
fn workbench_drag_target_route_prefers_right_target_in_bottom_right_overlap_when_pointer_is_closer_to_right_edge(
) {
    let geometry = shell_geometry(
        ShellFrame::new(1348.0, 50.0, 0.0, 666.0),
        ShellFrame::new(34.0, 50.0, 1314.0, 666.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 0.0),
    );

    assert_eq!(
        resolve_workbench_drag_target_group(
            ShellSizePx::new(1440.0, 900.0),
            &geometry,
            true,
            UiPoint::new(1428.0, 860.0),
        ),
        Some(WorkbenchDragTargetGroup::Right)
    );
}

#[test]
fn workbench_drag_target_route_prefers_bottom_target_in_bottom_right_overlap_when_pointer_is_closer_to_bottom_edge(
) {
    let geometry = shell_geometry(
        ShellFrame::new(1348.0, 50.0, 0.0, 666.0),
        ShellFrame::new(34.0, 50.0, 1314.0, 666.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 0.0),
    );

    assert_eq!(
        resolve_workbench_drag_target_group(
            ShellSizePx::new(1440.0, 900.0),
            &geometry,
            true,
            UiPoint::new(1380.0, 860.0),
        ),
        Some(WorkbenchDragTargetGroup::Bottom)
    );
}
