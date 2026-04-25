use std::collections::BTreeMap;

use crate::ui::slint_host::drawer_resize::{
    resolve_host_resize_target_group, HostResizeTargetGroup,
};
use crate::ui::workbench::autolayout::{
    ShellFrame, ShellRegionId, ShellSizePx, WorkbenchShellGeometry,
};
use zircon_runtime::ui::layout::UiPoint;

#[test]
fn shared_resize_target_route_resolves_left_right_and_bottom_splitters() {
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 48.0, 1440.0, 832.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::new(),
        splitter_frames: BTreeMap::from([
            (
                ShellRegionId::Left,
                ShellFrame::new(308.0, 48.0, 8.0, 832.0),
            ),
            (
                ShellRegionId::Right,
                ShellFrame::new(1124.0, 48.0, 8.0, 832.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(0.0, 716.0, 1440.0, 8.0),
            ),
        ]),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::new(0.0, 0.0, 0.0, 0.0),
    };
    let shell_size = ShellSizePx::new(1440.0, 900.0);

    assert_eq!(
        resolve_host_resize_target_group(shell_size, &geometry, UiPoint::new(312.0, 420.0)),
        Some(HostResizeTargetGroup::Left)
    );
    assert_eq!(
        resolve_host_resize_target_group(shell_size, &geometry, UiPoint::new(1128.0, 420.0)),
        Some(HostResizeTargetGroup::Right)
    );
    assert_eq!(
        resolve_host_resize_target_group(shell_size, &geometry, UiPoint::new(720.0, 720.0)),
        Some(HostResizeTargetGroup::Bottom)
    );
}

#[test]
fn shared_resize_target_route_ignores_points_outside_splitter_frames() {
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 48.0, 1440.0, 832.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::new(),
        splitter_frames: BTreeMap::from([(
            ShellRegionId::Left,
            ShellFrame::new(308.0, 48.0, 8.0, 832.0),
        )]),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::new(0.0, 0.0, 0.0, 0.0),
    };

    assert_eq!(
        resolve_host_resize_target_group(
            ShellSizePx::new(1440.0, 900.0),
            &geometry,
            UiPoint::new(420.0, 420.0),
        ),
        None
    );
}
