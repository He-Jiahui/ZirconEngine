use std::collections::BTreeMap;

use crate::ui::slint_host::drawer_resize::WorkbenchResizeTargetGroup;
use crate::ui::slint_host::shell_pointer::{
    WorkbenchShellPointerBridge, WorkbenchShellPointerRoute,
};
use crate::ui::slint_host::tab_drag::WorkbenchDragTargetGroup;
use crate::ui::workbench::autolayout::{
    ShellFrame, ShellRegionId, ShellSizePx, WorkbenchShellGeometry,
};
use zircon_runtime::ui::layout::UiPoint;

#[test]
fn unified_shell_pointer_bridge_routes_drag_targets_and_resize_targets_from_one_surface() {
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 48.0, 1440.0, 832.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::from([
            (
                ShellRegionId::Left,
                ShellFrame::new(0.0, 48.0, 308.0, 832.0),
            ),
            (
                ShellRegionId::Document,
                ShellFrame::new(316.0, 48.0, 808.0, 668.0),
            ),
            (
                ShellRegionId::Right,
                ShellFrame::new(1132.0, 48.0, 308.0, 668.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(0.0, 724.0, 1440.0, 156.0),
            ),
        ]),
        splitter_frames: BTreeMap::from([
            (
                ShellRegionId::Left,
                ShellFrame::new(308.0, 48.0, 8.0, 832.0),
            ),
            (
                ShellRegionId::Right,
                ShellFrame::new(1124.0, 48.0, 8.0, 668.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(0.0, 716.0, 1440.0, 8.0),
            ),
        ]),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::new(316.0, 82.0, 808.0, 634.0),
    };
    let mut bridge = WorkbenchShellPointerBridge::new();
    bridge.update_layout_with_floating_windows(
        ShellSizePx::new(1440.0, 900.0),
        &geometry,
        true,
        &[],
        &[],
    );

    assert_eq!(
        bridge.drag_target_at(UiPoint::new(1428.0, 240.0)),
        Some(WorkbenchDragTargetGroup::Right)
    );
    assert_eq!(
        bridge.begin_resize(UiPoint::new(312.0, 420.0)),
        Some(WorkbenchShellPointerRoute::Resize(
            WorkbenchResizeTargetGroup::Left
        ))
    );
}

#[test]
fn unified_shell_pointer_bridge_keeps_resize_route_captured_until_pointer_up() {
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 48.0, 1440.0, 832.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::from([
            (
                ShellRegionId::Left,
                ShellFrame::new(0.0, 48.0, 308.0, 832.0),
            ),
            (
                ShellRegionId::Document,
                ShellFrame::new(316.0, 48.0, 1124.0, 668.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(0.0, 724.0, 1440.0, 156.0),
            ),
        ]),
        splitter_frames: BTreeMap::from([(
            ShellRegionId::Left,
            ShellFrame::new(308.0, 48.0, 8.0, 832.0),
        )]),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::new(316.0, 82.0, 1124.0, 634.0),
    };
    let mut bridge = WorkbenchShellPointerBridge::new();
    bridge.update_layout_with_floating_windows(
        ShellSizePx::new(1440.0, 900.0),
        &geometry,
        true,
        &[],
        &[],
    );

    assert_eq!(
        bridge.begin_resize(UiPoint::new(312.0, 420.0)),
        Some(WorkbenchShellPointerRoute::Resize(
            WorkbenchResizeTargetGroup::Left
        ))
    );
    assert_eq!(
        bridge.update_resize(UiPoint::new(900.0, 420.0)),
        Some(WorkbenchResizeTargetGroup::Left)
    );
    assert_eq!(
        bridge.finish_resize(UiPoint::new(900.0, 420.0)),
        Some(WorkbenchResizeTargetGroup::Left)
    );
    assert_eq!(bridge.update_resize(UiPoint::new(900.0, 420.0)), None);
}
