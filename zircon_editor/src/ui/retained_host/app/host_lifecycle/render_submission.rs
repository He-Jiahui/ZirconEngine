use std::collections::{BTreeMap, BTreeSet};

use super::super::*;
use crate::core::editor_event::ViewInstanceId;
use crate::ui::retained_host::host_contract::HostWindowPresentationData;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, write_error, DiagnosticLogLevel,
};

struct SceneViewportSurface {
    surface_key: String,
    view_id: ViewInstanceId,
    size: UVec2,
}

impl RetainedEditorHost {
    pub(super) fn submit_render_frame_if_dirty(&mut self) {
        if !self.render_dirty {
            return;
        }

        let pending_render = self
            .invalidation
            .consume_recompute_reasons(HostInvalidationMask::RENDER);
        let render_reasons = if pending_render.is_empty() {
            HostInvalidationMask::RENDER
        } else {
            pending_render
        };
        let render_rebuild = self.invalidation.record_render_rebuild();
        record_current_ui_perf_counter(UiPerfCounter::RenderPathCount, 1.0);
        self.publish_refresh_invalidation_diagnostics();
        if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
            write_diagnostic_log(
                "editor_host_invalidation",
                format!(
                    "render_path count={} reasons={} {}",
                    render_rebuild,
                    render_reasons.summary(),
                    self.invalidation.stats_summary()
                ),
            );
        }

        let project_open = self.runtime.editor_snapshot().project_open;
        let generation = self.ui.get_host_presentation_generation();
        let surfaces = scene_viewport_surfaces(generation.structure(), project_open);
        if !project_open {
            self.runtime.retain_scene_viewports(&BTreeSet::new());
            self.render_dirty = false;
            return;
        }
        let retained = surfaces
            .iter()
            .map(|surface| surface.surface_key.clone())
            .collect::<BTreeSet<_>>();
        let retained_views = self
            .runtime
            .view_instance_ids_for_descriptor_key("editor.scene")
            .into_iter()
            .map(|instance_id| ViewInstanceId::new(instance_id.0))
            .collect::<BTreeSet<_>>();
        self.runtime.retain_scene_viewports(&retained_views);
        let mut keep_render_dirty = false;
        if let Err(error) = self.viewport.retain_viewport_surfaces(&retained) {
            write_error(
                "editor_viewport_retirement",
                format!("Viewport retirement failed: {error}"),
            );
            keep_render_dirty = true;
        }

        let mut submitted = false;
        for surface in surfaces {
            let runtime_viewport = match self
                .viewport
                .ensure_runtime_viewport(&surface.surface_key, surface.size)
            {
                Ok(Some(viewport)) => viewport,
                Ok(None) => {
                    keep_render_dirty = true;
                    continue;
                }
                Err(error) => {
                    keep_render_dirty = true;
                    write_error(
                        "editor_viewport_creation",
                        format!(
                            "Viewport creation failed for {}: {error}",
                            surface.surface_key
                        ),
                    );
                    continue;
                }
            };
            let Some(submission) = self.runtime.render_frame_submission_for_view(
                &surface.view_id,
                surface.size,
                runtime_viewport,
            ) else {
                continue;
            };
            zircon_runtime::profile_scope!("editor", "retained_host", "submit_viewport_extract");
            match self.viewport.submit_extract_with_ui(
                &surface.surface_key,
                submission.extract,
                submission.ui,
                surface.size,
            ) {
                Ok(true) => {
                    submitted = true;
                    let visible_spatial_snapshot =
                        match self.viewport.visible_spatial_snapshot(&surface.surface_key) {
                            Ok(snapshot) => snapshot,
                            Err(error) => {
                                write_diagnostic_log(
                                    "editor_viewport_visible_spatial_query",
                                    format!(
                                    "renderer-visible spatial query unavailable for {}: {error}",
                                    surface.surface_key
                                ),
                                );
                                None
                            }
                        };
                    self.runtime
                        .sync_renderer_visible_spatial_snapshot_for_view(
                            &surface.view_id,
                            visible_spatial_snapshot,
                        );
                }
                Ok(false) => keep_render_dirty = true,
                Err(error) => {
                    write_error(
                        "editor_viewport_submission",
                        format!(
                            "Viewport submit failed for {}: {error}",
                            surface.surface_key
                        ),
                    );
                    self.set_status_line(format!(
                        "Viewport submit failed for {}: {error}",
                        surface.surface_key
                    ));
                }
            }
        }
        if submitted {
            self.schedule_runtime_diagnostics_refresh();
        }
        self.render_dirty = keep_render_dirty;
        if keep_render_dirty {
            let frame = self.ui.get_host_window_bootstrap().viewport_content_frame;
            self.ui.request_frame_update_region(frame);
        }
    }

    fn schedule_runtime_diagnostics_refresh(&mut self) {
        match std::mem::take(&mut self.runtime_diagnostics_refresh_target) {
            RuntimeDiagnosticsRefreshTarget::None => {}
            RuntimeDiagnosticsRefreshTarget::Pending => {
                self.runtime_diagnostics_refresh_target = RuntimeDiagnosticsRefreshTarget::Pending;
            }
            RuntimeDiagnosticsRefreshTarget::ShellContent(scope) => {
                self.runtime_diagnostics_refresh_target = RuntimeDiagnosticsRefreshTarget::Pending;
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.runtime_diagnostics.shell_content_refresh_count",
                    1
                );
                self.invalidate_host_for_shell_content(scope, HostInvalidationMask::SHELL_CONTENT);
            }
            RuntimeDiagnosticsRefreshTarget::FullPresentation => {
                self.runtime_diagnostics_refresh_target = RuntimeDiagnosticsRefreshTarget::Pending;
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.runtime_diagnostics.full_presentation_fallback_count",
                    1
                );
                self.mark_presentation_dirty();
            }
        }
    }
}

fn scene_viewport_surfaces(
    presentation: &HostWindowPresentationData,
    project_open: bool,
) -> Vec<SceneViewportSurface> {
    if !project_open {
        return Vec::new();
    }
    let scene = &presentation.host_scene_data;
    let mut surfaces = BTreeMap::new();
    for surface in scene.document_surfaces() {
        insert_scene_surface(
            &mut surfaces,
            surface.surface_key.as_str(),
            &surface.pane,
            surface.content_frame.width,
            surface.content_frame.height,
        );
    }
    for (surface_key, pane, width, height) in [
        (
            scene.left_dock.surface_key.as_str(),
            &scene.left_dock.pane,
            scene.left_dock.content_frame.width,
            scene.left_dock.content_frame.height,
        ),
        (
            scene.right_dock.surface_key.as_str(),
            &scene.right_dock.pane,
            scene.right_dock.content_frame.width,
            scene.right_dock.content_frame.height,
        ),
        (
            scene.bottom_dock.surface_key.as_str(),
            &scene.bottom_dock.pane,
            scene.bottom_dock.content_frame.width,
            scene.bottom_dock.content_frame.height,
        ),
    ] {
        insert_scene_surface(&mut surfaces, surface_key, pane, width, height);
    }
    for window in scene.floating_layer.floating_windows.iter() {
        insert_scene_surface(
            &mut surfaces,
            window.window_id.as_str(),
            &window.active_pane,
            window.frame.width,
            window.frame.height - scene.floating_layer.header_height_px,
        );
    }
    for window in presentation
        .native_floating_surface_data
        .floating_windows
        .iter()
    {
        insert_scene_surface(
            &mut surfaces,
            window.window_id.as_str(),
            &window.active_pane,
            window.frame.width,
            window.frame.height - presentation.native_floating_surface_data.header_height_px,
        );
    }
    surfaces.into_values().collect()
}

fn insert_scene_surface(
    surfaces: &mut BTreeMap<String, SceneViewportSurface>,
    surface_key: &str,
    pane: &crate::ui::retained_host::host_contract::PaneData,
    width: f32,
    height: f32,
) {
    if pane.kind.as_str() != "Scene" || surface_key.is_empty() || pane.id.is_empty() {
        return;
    }
    let width = width.max(0.0).round() as u32;
    let height = height.max(0.0).round() as u32;
    if width == 0 || height == 0 {
        return;
    }
    surfaces.insert(
        surface_key.to_string(),
        SceneViewportSurface {
            surface_key: surface_key.to_string(),
            view_id: ViewInstanceId::new(pane.id.as_str()),
            size: UVec2::new(width, height),
        },
    );
}

#[cfg(test)]
#[path = "tests/render_submission.rs"]
mod tests;
