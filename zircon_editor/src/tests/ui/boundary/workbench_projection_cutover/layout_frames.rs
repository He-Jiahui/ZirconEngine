use super::*;

#[test]
fn workbench_root_shell_projection_module_is_removed_after_layout_frame_cutover() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(!root
        .join("src")
        .join("ui")
        .join("retained_host")
        .join("root_shell_projection.rs")
        .exists());

    let retained_host_mod = source_file(&["src", "ui", "retained_host", "mod.rs"]);
    assert_does_not_contain(
        "retained_host/mod.rs",
        &retained_host_mod,
        "root_shell_projection",
    );
}

#[test]
fn shell_pointer_drag_surface_uses_workbench_layout_frames_without_root_fallback() {
    let drag_surface = source_file(&[
        "src",
        "ui",
        "retained_host",
        "shell_pointer",
        "drag_surface.rs",
    ]);

    for required in [
        "componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames",
        "center_band_frame",
        "status_bar_frame",
        "document_region_frame",
        "left_region_frame",
        "right_region_frame",
        "bottom_region_frame",
    ] {
        assert_contains("drag_surface.rs", &drag_surface, required);
    }

    for forbidden in [
        "WorkbenchShellGeometry",
        "ShellRegionId",
        "shared_or_fallback_frame",
        "shared_or_geometry_frame",
        "shared_root_frames",
        "geometry.center_band_frame",
        "geometry.status_bar_frame",
        "geometry.region_frame",
        "resolve_root_center_band_frame",
        "resolve_root_status_bar_frame",
        "resolve_root_document_region_frame",
        "resolve_direct_document_host_frame",
        "resolve_root_left_region_frame(",
        "resolve_root_right_region_frame(",
        "resolve_root_bottom_region_frame(",
    ] {
        assert_does_not_contain("drag_surface.rs", &drag_surface, forbidden);
    }
}

#[test]
fn shell_pointer_bridge_does_not_recreate_root_frames_from_geometry() {
    let bridge = source_file(&["src", "ui", "retained_host", "shell_pointer", "bridge.rs"]);

    for required in [
        "update_layout_with_root_shell_frames(",
        "test_workbench_layout_frames_from_root_frames(",
        "update_layout_with_workbench_layout_frames(",
        "build_drag_surface(",
        "update_resize_surface(",
        "componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames",
    ] {
        assert_contains("bridge.rs", &bridge, required);
    }

    for forbidden in [
        "build_drag_surface(\n            root_size,\n            drawers_visible,\n            floating_windows,\n            shared_root_frames,",
        "root_frames_from_geometry",
        "geometry.center_band_frame",
        "geometry.status_bar_frame",
        "geometry.region_frame",
        "ShellRegionId",
    ] {
        assert_does_not_contain("bridge.rs", &bridge, forbidden);
    }
}

#[test]
fn host_presentation_and_viewport_use_workbench_frames_without_root_fallback() {
    let apply_presentation =
        source_file(&["src", "ui", "retained_host", "ui", "apply_presentation.rs"]);
    let host_lifecycle = source_file(&["src", "ui", "retained_host", "app", "host_lifecycle.rs"]);
    let viewport = source_file(&["src", "ui", "retained_host", "app", "viewport.rs"]);

    for required in [
        "componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames",
        "host_window_layout(componentized_workbench_layout_frames)",
        "center_band_frame",
        "status_bar_frame",
        "document_region_frame",
        "viewport_content_frame",
    ] {
        assert_contains("apply_presentation.rs", &apply_presentation, required);
    }

    for forbidden in [
        "BuiltinHostRootShellFrames",
        "shared_root_frames",
        "document_pane_shows_viewport_toolbar",
        "resolve_root_center_band_frame",
        "resolve_root_status_bar_frame",
        "resolve_root_document_region_frame",
        "resolve_root_viewport_content_frame",
    ] {
        assert_does_not_contain("apply_presentation.rs", &apply_presentation, forbidden);
    }

    for forbidden in [
        "resolve_root_viewport_content_frame",
        "active_document_shows_viewport_toolbar",
    ] {
        assert_does_not_contain("host_lifecycle.rs", &host_lifecycle, forbidden);
        assert_does_not_contain("viewport.rs", &viewport, forbidden);
    }
}

#[test]
fn tab_drag_strip_hitbox_uses_workbench_layout_frames_without_root_fallback() {
    let strip_hitbox = source_file(&["src", "ui", "retained_host", "tab_drag", "strip_hitbox.rs"]);

    for required in [
        "center_band_frame",
        "document_tabs_frame",
        "document_region_frame",
        "left_region_frame",
        "right_region_frame",
        "bottom_region_frame",
    ] {
        assert_contains("strip_hitbox.rs", &strip_hitbox, required);
    }

    for forbidden in [
        "WorkbenchShellGeometry",
        "shared_or_geometry_frame",
        "geometry.region_frame",
        "geometry.center_band_frame",
        "resolve_root_left_region_frame(",
        "resolve_root_right_region_frame(",
        "resolve_root_bottom_region_frame(",
        "resolve_root_center_band_frame",
        "resolve_root_document_region_frame",
        "resolve_direct_document_host_frame",
    ] {
        assert_does_not_contain("strip_hitbox.rs", &strip_hitbox, forbidden);
    }
}

#[test]
fn floating_window_projection_uses_shared_source_without_geometry_fallback() {
    let projection = source_file(&[
        "src",
        "ui",
        "retained_host",
        "floating_window_projection.rs",
    ]);

    for required in [
        "resolve_floating_window_projection_shared_source(",
        "build_floating_window_projection_bundle_from_windows_with_shared_source(",
        "resolve_floating_window_outer_frame_from_shared_source(",
        "window.requested_frame",
        "resolve_native_floating_window_host_frame(",
    ] {
        assert_contains("floating_window_projection.rs", &projection, required);
    }

    for forbidden in [
        "floating_window_projection_shared_source_from_geometry",
        "build_floating_window_projection_bundle_from_windows_with_geometry",
        "resolve_floating_window_projected_outer_frame_with_fallback",
        "WorkbenchShellGeometry",
        ".floating_window_frame(",
        "geometry.region_frame",
        "geometry.center_band_frame",
    ] {
        assert_does_not_contain("floating_window_projection.rs", &projection, forbidden);
    }
}
