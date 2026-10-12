use super::*;
use crate::ui::retained_host::host_contract::HostDocumentDockSurfaceData;

#[test]
fn split_scene_leaves_project_distinct_runtime_view_owners_and_sizes() {
    let mut presentation = HostWindowPresentationData::default();
    let leaf = |surface_key: &str, view_id: &str, width: f32, height: f32| {
        let mut surface = HostDocumentDockSurfaceData::default();
        surface.surface_key = surface_key.into();
        surface.content_frame.width = width;
        surface.content_frame.height = height;
        surface.pane.id = view_id.into();
        surface.pane.kind = "Scene".into();
        surface
    };
    presentation.host_scene_data.document_leaves = vec![
        leaf("document:left", "editor.scene#1", 360.0, 540.0),
        leaf("document:right", "editor.scene#2", 840.0, 540.0),
    ];

    let surfaces = scene_viewport_surfaces(&presentation, true);

    assert_eq!(surfaces.len(), 2);
    assert_eq!(surfaces[0].surface_key, "document:left");
    assert_eq!(surfaces[0].view_id, ViewInstanceId::new("editor.scene#1"));
    assert_eq!(surfaces[0].size, UVec2::new(360, 540));
    assert_eq!(surfaces[1].surface_key, "document:right");
    assert_eq!(surfaces[1].view_id, ViewInstanceId::new("editor.scene#2"));
    assert_eq!(surfaces[1].size, UVec2::new(840, 540));
}

#[test]
fn welcome_defers_scene_viewport_surfaces_until_a_project_opens() {
    let mut presentation = HostWindowPresentationData::default();
    let mut surface = HostDocumentDockSurfaceData::default();
    surface.surface_key = "document:scene".into();
    surface.content_frame.width = 640.0;
    surface.content_frame.height = 480.0;
    surface.pane.id = "editor.scene#1".into();
    surface.pane.kind = "Scene".into();
    presentation.host_scene_data.document_leaves = vec![surface];

    assert!(scene_viewport_surfaces(&presentation, false).is_empty());
    let opened = scene_viewport_surfaces(&presentation, true);
    assert_eq!(opened.len(), 1);
    assert_eq!(opened[0].surface_key, "document:scene");
}

#[test]
fn successful_render_submission_refreshes_post_submit_diagnostics_without_requeueing_render() {
    let source = include_str!("../render_submission.rs");
    let success_arm = source
        .split_once("Ok(true) => {")
        .and_then(|(_, tail)| tail.split_once("Ok(false) =>"))
        .map(|(arm, _)| arm)
        .expect("render submission success arm should remain explicit");

    assert!(success_arm.contains("submitted = true;"));
    assert!(success_arm.contains("visible_spatial_snapshot"));
    assert!(success_arm.contains("sync_renderer_visible_spatial_snapshot_for_view"));
    assert!(!success_arm.contains("mark_render_and_presentation_dirty"));
}

#[test]
fn diagnostics_refresh_consumes_a_publication_time_target_without_a_hot_path_scan() {
    let source = include_str!("../render_submission.rs");
    let function = source
        .split("fn schedule_runtime_diagnostics_refresh")
        .nth(1)
        .and_then(|tail| tail.split("fn scene_viewport_surfaces").next())
        .expect("runtime diagnostics refresh scheduler");

    assert!(function.contains("std::mem::take"));
    assert!(function.contains("RuntimeDiagnosticsRefreshTarget::Pending"));
    assert!(function.contains("RuntimeDiagnosticsRefreshTarget::ShellContent(scope)"));
    assert!(function.contains("HostInvalidationMask::SHELL_CONTENT"));
    assert!(function.contains("RuntimeDiagnosticsRefreshTarget::FullPresentation"));
    assert!(function.contains("self.mark_presentation_dirty();"));
    assert!(!function.contains("tool_windows.iter"));
    assert!(!function.contains("document_tabs.iter"));
}

#[test]
fn failed_render_submission_records_the_typed_error_in_process_diagnostics() {
    let source = include_str!("../render_submission.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map_or(source, |(head, _)| head);

    assert!(production.contains("write_error("));
    assert!(production.contains("editor_viewport_submission"));
    assert!(production.contains("Viewport submit failed for"));
}

#[test]
fn render_submission_uses_a_runtime_owner_for_every_scene_leaf() {
    let source = include_str!("../render_submission.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map_or(source, |(head, _)| head);

    assert!(production.contains("for surface in surfaces"));
    assert!(production.contains("ensure_runtime_viewport"));
    assert!(production.contains("runtime_viewport"));
    assert!(production.contains("render_frame_submission_for_view"));
    assert!(production.contains("retain_viewport_surfaces"));
    assert!(!production.contains("self.runtime.render_frame_submission()"));
}
