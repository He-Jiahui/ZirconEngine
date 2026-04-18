use super::*;
use crate::host::slint_host::floating_window_projection::{
    resolve_floating_window_projection_base_outer_frame,
    resolve_floating_window_projection_shared_source,
};
use zircon_ui::UiSize;

#[test]
fn child_window_hierarchy_pointer_move_prefers_projected_floating_window_content_frame_over_outer_window_frame(
) {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_slint_child_window_floating_content_projection");
    let child = harness.detach_view_to_child_window("editor.hierarchy#1", "window:hierarchy");
    let expected_size = {
        let host = harness.host.borrow();
        let window_id = MainPageId::new("window:hierarchy");
        let frame = host
            .shell_geometry
            .as_ref()
            .expect("child window host should have shell geometry")
            .floating_window_frame(&window_id);
        UiSize::new(
            frame.width.max(0.0),
            (frame.height
                - host.chrome_metrics.document_header_height
                - host.chrome_metrics.separator_thickness)
                .max(0.0),
        )
    };

    child.invoke_hierarchy_pointer_moved(24.0, 24.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert_eq!(
        host.hierarchy_pointer_size, expected_size,
        "child-window hierarchy fallback should use floating window content frame rather than the outer shell frame"
    );
}

#[test]
fn child_window_hierarchy_pointer_falls_back_to_native_window_bounds_when_projection_bundle_is_missing(
) {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_slint_child_window_floating_content_native_fallback");
    let child = harness.detach_view_to_child_window("editor.hierarchy#1", "window:hierarchy");
    let native_bounds = child.get_native_window_bounds();
    let expected_size = {
        let mut host = harness.host.borrow_mut();
        let window_id = MainPageId::new("window:hierarchy");
        host.floating_window_projection_bundle = FloatingWindowProjectionBundle::default();
        host.shell_geometry
            .as_mut()
            .expect("child window host should have shell geometry")
            .floating_window_frames
            .insert(window_id, ShellFrame::new(18.0, 22.0, 96.0, 64.0));
        UiSize::new(
            native_bounds.width.max(0.0),
            (native_bounds.height
                - host.chrome_metrics.document_header_height
                - host.chrome_metrics.separator_thickness)
                .max(0.0),
        )
    };

    child.invoke_hierarchy_pointer_moved(24.0, 24.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert_eq!(
        host.hierarchy_pointer_size, expected_size,
        "child-window fallback should derive content size from native window bounds once the cached projection bundle is missing"
    );
}

#[test]
fn child_window_host_recompute_caches_floating_window_projection_bundle_for_detached_window() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_child_window_projection_bundle_cache");
    harness.detach_view_to_child_window("editor.hierarchy#1", "window:hierarchy");

    let host = harness.host.borrow();
    let window_id = MainPageId::new("window:hierarchy");
    let geometry_frame = host
        .shell_geometry
        .as_ref()
        .expect("child window host should have shell geometry")
        .floating_window_frame(&window_id);
    let chrome = host.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let window_index = model
        .floating_windows
        .iter()
        .position(|window| window.window_id == window_id)
        .expect("detached child window should exist in the shared view model");
    let expected_outer_frame = resolve_floating_window_projection_base_outer_frame(
        &model.floating_windows[window_index],
        window_index,
        resolve_floating_window_projection_shared_source(
            &host.floating_window_source_bridge.source_frames(),
        ),
    );
    let frames = host
        .floating_window_projection_bundle
        .frames(&window_id)
        .expect("host recompute should cache shared floating-window projection frames");

    assert_ne!(
        expected_outer_frame, geometry_frame,
        "shared floating-window projection should no longer be forced to match the legacy geometry fallback"
    );
    assert_eq!(frames.outer_frame, expected_outer_frame);
    assert_eq!(frames.host_frame, Some(expected_outer_frame));
    assert!(frames.native_host_present);
    assert_eq!(
        frames.tab_strip_frame,
        ShellFrame::new(
            expected_outer_frame.x,
            expected_outer_frame.y,
            expected_outer_frame.width,
            host.chrome_metrics.document_header_height,
        )
    );
    assert_eq!(
        frames.content_frame,
        ShellFrame::new(
            expected_outer_frame.x,
            expected_outer_frame.y
                + host.chrome_metrics.document_header_height
                + host.chrome_metrics.separator_thickness,
            expected_outer_frame.width,
            expected_outer_frame.height
                - host.chrome_metrics.document_header_height
                - host.chrome_metrics.separator_thickness,
        )
    );
}
