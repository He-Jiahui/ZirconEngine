use crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::slint_host::drawer_resize::{
    resolve_host_resize_target_group_with_root_frames, HostResizeTargetGroup,
};
use crate::ui::workbench::autolayout::ShellSizePx;
use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint};

fn root_shell_frames() -> BuiltinHostRootShellFrames {
    BuiltinHostRootShellFrames {
        shell_frame: Some(UiFrame::new(0.0, 0.0, 1440.0, 900.0)),
        host_body_frame: Some(UiFrame::new(0.0, 48.0, 1440.0, 832.0)),
        left_drawer_shell_frame: Some(UiFrame::new(0.0, 48.0, 316.0, 832.0)),
        right_drawer_shell_frame: Some(UiFrame::new(1132.0, 48.0, 308.0, 832.0)),
        bottom_drawer_shell_frame: Some(UiFrame::new(0.0, 724.0, 1440.0, 156.0)),
        document_host_frame: Some(UiFrame::new(324.0, 48.0, 800.0, 668.0)),
        status_bar_frame: Some(UiFrame::new(0.0, 880.0, 1440.0, 20.0)),
        ..Default::default()
    }
}

#[test]
fn shared_resize_target_route_resolves_left_right_and_bottom_splitters() {
    let shell_size = ShellSizePx::new(1440.0, 900.0);
    let shared_root_frames = root_shell_frames();

    assert_eq!(
        resolve_host_resize_target_group_with_root_frames(
            shell_size,
            Some(&shared_root_frames),
            UiPoint::new(312.0, 420.0)
        ),
        Some(HostResizeTargetGroup::Left)
    );
    assert_eq!(
        resolve_host_resize_target_group_with_root_frames(
            shell_size,
            Some(&shared_root_frames),
            UiPoint::new(1128.0, 420.0)
        ),
        Some(HostResizeTargetGroup::Right)
    );
    assert_eq!(
        resolve_host_resize_target_group_with_root_frames(
            shell_size,
            Some(&shared_root_frames),
            UiPoint::new(720.0, 730.0)
        ),
        Some(HostResizeTargetGroup::Bottom)
    );
}

#[test]
fn shared_resize_target_route_ignores_points_outside_splitter_frames() {
    let shared_root_frames = root_shell_frames();

    assert_eq!(
        resolve_host_resize_target_group_with_root_frames(
            ShellSizePx::new(1440.0, 900.0),
            Some(&shared_root_frames),
            UiPoint::new(420.0, 420.0),
        ),
        None
    );
}
