use crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::slint_host::drawer_resize::HostResizeTargetGroup;
use crate::ui::slint_host::shell_pointer::{HostShellPointerBridge, HostShellPointerRoute};
use crate::ui::slint_host::tab_drag::HostDragTargetGroup;
use crate::ui::workbench::autolayout::ShellSizePx;
use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint};

fn root_shell_frames() -> BuiltinHostRootShellFrames {
    BuiltinHostRootShellFrames {
        shell_frame: Some(UiFrame::new(0.0, 0.0, 1440.0, 900.0)),
        host_body_frame: Some(UiFrame::new(0.0, 48.0, 1440.0, 832.0)),
        left_drawer_shell_frame: Some(UiFrame::new(0.0, 48.0, 316.0, 832.0)),
        right_drawer_shell_frame: Some(UiFrame::new(1132.0, 48.0, 308.0, 668.0)),
        bottom_drawer_shell_frame: Some(UiFrame::new(0.0, 724.0, 1440.0, 156.0)),
        document_host_frame: Some(UiFrame::new(324.0, 48.0, 800.0, 668.0)),
        status_bar_frame: Some(UiFrame::new(0.0, 880.0, 1440.0, 20.0)),
        ..Default::default()
    }
}

#[test]
fn unified_shell_pointer_bridge_routes_drag_targets_and_resize_targets_from_one_surface() {
    let mut bridge = HostShellPointerBridge::new();
    let shared_root_frames = root_shell_frames();
    bridge.update_layout_with_root_shell_frames(
        ShellSizePx::new(1440.0, 900.0),
        true,
        &[],
        Some(&shared_root_frames),
        None,
    );

    assert_eq!(
        bridge.drag_target_at(UiPoint::new(1428.0, 240.0)),
        Some(HostDragTargetGroup::Right)
    );
    assert_eq!(
        bridge.begin_resize(UiPoint::new(312.0, 420.0)),
        Some(HostShellPointerRoute::Resize(HostResizeTargetGroup::Left))
    );
}

#[test]
fn unified_shell_pointer_bridge_keeps_resize_route_captured_until_pointer_up() {
    let mut bridge = HostShellPointerBridge::new();
    let shared_root_frames = root_shell_frames();
    bridge.update_layout_with_root_shell_frames(
        ShellSizePx::new(1440.0, 900.0),
        true,
        &[],
        Some(&shared_root_frames),
        None,
    );

    assert_eq!(
        bridge.begin_resize(UiPoint::new(312.0, 420.0)),
        Some(HostShellPointerRoute::Resize(HostResizeTargetGroup::Left))
    );
    assert_eq!(
        bridge.update_resize(UiPoint::new(900.0, 420.0)),
        Some(HostResizeTargetGroup::Left)
    );
    assert_eq!(
        bridge.finish_resize(UiPoint::new(900.0, 420.0)),
        Some(HostResizeTargetGroup::Left)
    );
    assert_eq!(bridge.update_resize(UiPoint::new(900.0, 420.0)), None);
}
