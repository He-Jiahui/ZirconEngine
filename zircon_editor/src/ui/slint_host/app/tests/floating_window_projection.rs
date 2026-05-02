use super::*;
use crate::ui::slint_host::floating_window_projection::{
    resolve_floating_window_projection_base_outer_frame,
    resolve_floating_window_projection_shared_source,
};
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn floating_window_drag_surface_drops_geometry_outer_frame_fallback() {
    let drag_surface = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/shell_pointer/drag_surface.rs"
    ));

    for legacy_fallback in [
        "resolve_floating_window_outer_frame(geometry, &window.window_id)",
        "resolve_floating_window_outer_frame(\n                geometry,\n                &window.window_id,\n            )",
    ] {
        assert!(
            !drag_surface.contains(legacy_fallback),
            "floating drag surface should not fall back to geometry outer frame `{legacy_fallback}`"
        );
    }
}

#[test]
fn child_window_hierarchy_pointer_move_prefers_projected_floating_window_content_frame_over_outer_window_frame(
) {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_slint_child_window_floating_content_projection");
    let child = harness.detach_view_to_child_window("editor.hierarchy#1", "window:hierarchy");
    let (expected_size, geometry_content_size) = {
        let host = harness.host.borrow();
        let window_id = MainPageId::new("window:hierarchy");
        let projected_frame = host
            .floating_window_projection_bundle
            .content_frame(&window_id)
            .expect("child window host should cache projected floating window content frame");
        let geometry_frame = host
            .shell_geometry
            .as_ref()
            .expect("child window host should have shell geometry")
            .floating_window_frame(&window_id);
        (
            UiSize::new(
                projected_frame.width.max(0.0),
                projected_frame.height.max(0.0),
            ),
            UiSize::new(
                geometry_frame.width.max(0.0),
                (geometry_frame.height
                    - host.chrome_metrics.document_header_height
                    - host.chrome_metrics.separator_thickness)
                    .max(0.0),
            ),
        )
    };
    assert_ne!(
        expected_size, geometry_content_size,
        "fixture should distinguish shared projection content from legacy geometry content"
    );

    pane_surface_host(&child).invoke_hierarchy_pointer_moved(24.0, 24.0, 0.0, 0.0);

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
    let native_bounds = child
        .get_host_presentation()
        .host_shell
        .native_window_bounds;
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

    pane_surface_host(&child).invoke_hierarchy_pointer_moved(24.0, 24.0, 0.0, 0.0);

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

    let _geometry_frame = geometry_frame;
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
