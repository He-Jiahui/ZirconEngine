use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::retained_host::drawer_resize::{
    resolve_host_resize_target_group_with_workbench_layout_frames, HostResizeTargetGroup,
};
use crate::ui::retained_host::shell_pointer::{HostShellPointerBridge, HostShellPointerRoute};
use crate::ui::workbench::autolayout::{ShellSizePx, WorkbenchChromeMetrics};
use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint};

fn workbench_layout_frames() -> BuiltinWorkbenchWindowLayoutFrames {
    BuiltinWorkbenchWindowLayoutFrames {
        left_resize_splitter_frame: Some(UiFrame::new(312.0, 48.0, 8.0, 832.0)),
        right_resize_splitter_frame: Some(UiFrame::new(1128.0, 48.0, 8.0, 832.0)),
        bottom_resize_splitter_frame: Some(UiFrame::new(0.0, 720.0, 1440.0, 8.0)),
        ..Default::default()
    }
}

#[test]
fn shared_resize_target_route_resolves_left_right_and_bottom_splitters() {
    let shell_size = ShellSizePx::new(1440.0, 900.0);

    assert_eq!(
        resolve_host_resize_target_group_with_workbench_layout_frames(
            shell_size,
            workbench_layout_frames(),
            UiPoint::new(312.0, 420.0)
        ),
        Some(HostResizeTargetGroup::Left)
    );
    assert_eq!(
        resolve_host_resize_target_group_with_workbench_layout_frames(
            shell_size,
            workbench_layout_frames(),
            UiPoint::new(1128.0, 420.0)
        ),
        Some(HostResizeTargetGroup::Right)
    );
    let metrics = WorkbenchChromeMetrics::default();
    let bottom_splitter_y = 724.0 - metrics.separator_thickness;
    assert_eq!(
        resolve_host_resize_target_group_with_workbench_layout_frames(
            shell_size,
            workbench_layout_frames(),
            UiPoint::new(720.0, bottom_splitter_y)
        ),
        Some(HostResizeTargetGroup::Bottom)
    );
}

#[test]
fn shared_resize_target_route_ignores_points_outside_splitter_frames() {
    assert_eq!(
        resolve_host_resize_target_group_with_workbench_layout_frames(
            ShellSizePx::new(1440.0, 900.0),
            workbench_layout_frames(),
            UiPoint::new(420.0, 420.0),
        ),
        None
    );
}

#[test]
fn resize_target_route_prefers_componentized_workbench_splitter_frame() {
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_workbench_layout_frames(
        ShellSizePx::new(1440.0, 900.0),
        true,
        &[],
        BuiltinWorkbenchWindowLayoutFrames {
            left_resize_splitter_frame: Some(UiFrame::new(496.0, 48.0, 8.0, 832.0)),
            ..Default::default()
        },
        None,
    );

    assert_eq!(
        bridge.begin_resize(UiPoint::new(500.0, 420.0)),
        Some(HostShellPointerRoute::Resize(HostResizeTargetGroup::Left))
    );
    assert_eq!(
        bridge.finish_resize(UiPoint::new(500.0, 420.0)),
        Some(HostResizeTargetGroup::Left)
    );
    assert_eq!(bridge.begin_resize(UiPoint::new(312.0, 420.0)), None);
}
