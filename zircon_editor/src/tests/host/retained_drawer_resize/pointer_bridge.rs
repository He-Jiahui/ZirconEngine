use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::retained_host::drawer_resize::HostResizeTargetGroup;
use crate::ui::retained_host::shell_pointer::{HostShellPointerBridge, HostShellPointerRoute};
use crate::ui::retained_host::tab_drag::HostDragTargetGroup;
use crate::ui::workbench::autolayout::ShellSizePx;
use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint};

fn workbench_layout_frames() -> BuiltinWorkbenchWindowLayoutFrames {
    BuiltinWorkbenchWindowLayoutFrames {
        center_band_frame: Some(UiFrame::new(0.0, 48.0, 1440.0, 832.0)),
        document_region_frame: Some(UiFrame::new(324.0, 48.0, 800.0, 668.0)),
        status_bar_frame: Some(UiFrame::new(0.0, 880.0, 1440.0, 20.0)),
        left_region_frame: Some(UiFrame::new(0.0, 48.0, 316.0, 832.0)),
        right_region_frame: Some(UiFrame::new(1132.0, 48.0, 308.0, 668.0)),
        bottom_region_frame: Some(UiFrame::new(0.0, 724.0, 1440.0, 156.0)),
        left_resize_splitter_frame: Some(UiFrame::new(312.0, 48.0, 8.0, 832.0)),
        right_resize_splitter_frame: Some(UiFrame::new(1128.0, 48.0, 8.0, 668.0)),
        bottom_resize_splitter_frame: Some(UiFrame::new(0.0, 720.0, 1440.0, 8.0)),
        ..Default::default()
    }
}

#[test]
fn unified_shell_pointer_bridge_routes_drag_targets_and_resize_targets_from_one_surface() {
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_workbench_layout_frames(
        ShellSizePx::new(1440.0, 900.0),
        true,
        &[],
        workbench_layout_frames(),
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
    bridge.update_layout_with_workbench_layout_frames(
        ShellSizePx::new(1440.0, 900.0),
        true,
        &[],
        workbench_layout_frames(),
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

#[test]
fn rejected_resize_setup_can_cancel_capture_without_a_synthetic_pointer_event() {
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_workbench_layout_frames(
        ShellSizePx::new(1440.0, 900.0),
        true,
        &[],
        workbench_layout_frames(),
        None,
    );

    assert_eq!(
        bridge.begin_resize(UiPoint::new(312.0, 420.0)),
        Some(HostShellPointerRoute::Resize(HostResizeTargetGroup::Left))
    );
    bridge.cancel_resize();
    assert_eq!(bridge.update_resize(UiPoint::new(900.0, 420.0)), None);
}

#[test]
fn stable_resize_geometry_patches_drag_surface_without_replacing_authority() {
    let mut bridge = HostShellPointerBridge::new();
    let authority_generation = bridge.drag_surface_authority_generation_for_test();

    bridge.update_layout_with_workbench_layout_frames(
        ShellSizePx::new(1440.0, 900.0),
        true,
        &[],
        workbench_layout_frames(),
        None,
    );
    assert_eq!(
        bridge.drag_surface_authority_generation_for_test(),
        authority_generation,
        "stable empty floating-window topology should patch the retained drag surface"
    );

    let mut resized_frames = workbench_layout_frames();
    resized_frames.right_region_frame = Some(UiFrame::new(1232.0, 48.0, 368.0, 668.0));
    resized_frames.document_region_frame = Some(UiFrame::new(324.0, 48.0, 900.0, 668.0));
    resized_frames.status_bar_frame = Some(UiFrame::new(0.0, 880.0, 1600.0, 20.0));
    bridge.update_layout_with_workbench_layout_frames(
        ShellSizePx::new(1600.0, 900.0),
        true,
        &[],
        resized_frames,
        None,
    );

    assert_eq!(
        bridge.drag_surface_authority_generation_for_test(),
        authority_generation,
        "geometry-only resize must preserve surface, dispatcher, and route authority"
    );
    assert_eq!(
        bridge.drag_target_at(UiPoint::new(1588.0, 240.0)),
        Some(HostDragTargetGroup::Right),
        "the retained authority must publish the resized right target geometry"
    );
}
