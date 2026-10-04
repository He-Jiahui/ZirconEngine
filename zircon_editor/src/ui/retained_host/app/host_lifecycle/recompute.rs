use std::sync::Arc;

use super::super::*;

mod floating_projection;
mod invalidation;
mod pointer_surfaces;
mod presentation;
mod shell;
mod viewport_surfaces;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn recompute_if_dirty(&mut self) {
        if !self.presentation_dirty
            && !self.layout_dirty
            && !self.window_metrics_dirty
            && !self.invalidation.has_pending_presentation_recompute()
        {
            return;
        }
        zircon_runtime::profile_scope!("editor", "retained_host", "recompute_if_dirty");

        // Presenter retirement can hide a child while this host is mutably borrowed. Retire a
        // resize whose source disappeared from the runtime layout before taking new geometry.
        self.cancel_removed_drawer_resize_owner();

        let Some(recompute_decision) = self.begin_recompute_invalidation_phase() else {
            return;
        };
        let paint_only_reasons = recompute_decision.paint_only_reasons;
        let requested_shell_layout_reuse = recompute_decision.reuse_shell_layout;
        let shell_content_scope = match &recompute_decision.target {
            invalidation::RecomputeInvalidationTarget::ShellContent(scope) => Some(scope.clone()),
            _ => None,
        };
        if self
            .workbench_window_bridge
            .has_pending_surface_state_change()
        {
            if let Err(error) = self
                .workbench_window_bridge
                .refresh_prepared_state_change()
                .map_err(|error| error.to_string())
            {
                self.set_status_line(error);
            }
        }
        if matches!(
            &recompute_decision.target,
            invalidation::RecomputeInvalidationTarget::WorkbenchProjection
        ) {
            if self.apply_workbench_projection_presentation() {
                return;
            }
            self.record_slow_path_recompute(&HostInvalidationMask::WORKBENCH_PROJECTION, 1);
        }
        if let invalidation::RecomputeInvalidationTarget::ViewPresentation(view_ids) =
            &recompute_decision.target
        {
            if self.apply_scoped_ui_asset_presentation(view_ids) {
                return;
            }
            self.record_slow_path_recompute(
                &HostInvalidationMask::PRESENTATION_DATA,
                view_ids.len(),
            );
        }
        if let Some(scope) = shell_content_scope.as_ref() {
            if requested_shell_layout_reuse
                && self.apply_committed_shell_content_presentation(scope)
            {
                self.presentation_dirty = false;
                self.layout_dirty = false;
                self.window_metrics_dirty = false;
                self.workbench_window_bridge
                    .mark_host_projection_committed();
                self.publish_refresh_invalidation_diagnostics();
                return;
            }
            self.record_slow_path_recompute(&HostInvalidationMask::SHELL_CONTENT, 1);
        }

        if paint_only_reasons.requires_layout() {
            // layout-affecting invalidations still require the full shell recompute below;
            // pure paint-only reasons stay eligible for a lighter path after the shared
            // root frames are already stable.
        }
        let window_metrics_target = matches!(
            &recompute_decision.target,
            invalidation::RecomputeInvalidationTarget::WindowMetrics
        );
        let mut shell = if window_metrics_target {
            self.build_window_metrics_shell_snapshot()
                .unwrap_or_else(|| {
                    zircon_runtime::profile_counter!(
                        "editor",
                        "ui.window_metrics.committed_shell_stage_miss_count",
                        1
                    );
                    self.build_recompute_shell_snapshot(false)
                })
        } else {
            self.build_recompute_shell_snapshot(requested_shell_layout_reuse)
        };
        self.runtime_diagnostics_refresh_target =
            runtime_diagnostics_visibility::runtime_diagnostics_refresh_target(&shell.model);
        let componentized_workbench_layout_frames = shell.componentized_workbench_layout_frames;
        let floating_window_projection_bundle =
            self.build_recompute_floating_window_projection_bundle(&shell.model);
        let reuse_shell_layout = shell.reuse_shell_layout && shell_content_scope.is_none();
        self.sync_recompute_viewport_and_pointer_layouts(
            &mut shell.model,
            &mut shell.chrome,
            componentized_workbench_layout_frames,
            &floating_window_projection_bundle,
            window_metrics_target,
        );

        if window_metrics_target {
            if let (Some(retained_shell_presentation), Some(retained_pane_payloads)) = (
                shell.retained_shell_presentation.as_ref(),
                shell.retained_pane_payloads.as_ref(),
            ) {
                let geometry_published =
                    crate::ui::retained_host::ui::apply_window_metrics_geometry_presentation(
                        &self.ui,
                        retained_shell_presentation,
                        &shell.model,
                        &shell.geometry,
                        componentized_workbench_layout_frames,
                        Some(self.template_bridge.host_projection()),
                        Some(self.workbench_window_bridge.host_projection()),
                        self.workbench_window_bridge
                            .pending_host_projection_geometry_patch_indices()
                            .as_deref(),
                        self.template_bridge.presentation_scale_factor(),
                        &floating_window_projection_bundle,
                    );
                if geometry_published {
                    self.sync_recompute_viewport_surfaces(componentized_workbench_layout_frames);
                    {
                        zircon_runtime::profile_scope!(
                            "editor",
                            "retained_host",
                            "recompute_native_window_presenters"
                        );
                        self.sync_native_window_presenters(
                            &shell.model,
                            &shell.chrome,
                            &shell.geometry,
                            &retained_pane_payloads.preset_names,
                            &retained_pane_payloads.ui_asset_panes,
                            &retained_pane_payloads.animation_panes,
                            &retained_pane_payloads.runtime_diagnostics,
                            &floating_window_projection_bundle,
                        );
                    }
                    self.sync_recompute_pointer_surfaces(
                        &shell.model,
                        &shell.chrome,
                        &retained_pane_payloads.preset_names,
                        true,
                    );
                    self.floating_window_projection_bundle = floating_window_projection_bundle;
                    self.shell_geometry = Some(shell.geometry.clone());
                    self.committed_shell_state = Some(committed_shell_state::CommittedShellState {
                        layout: shell.layout,
                        chrome: shell.chrome,
                        model: shell.model,
                        geometry: shell.geometry,
                        layout_frames: componentized_workbench_layout_frames,
                        descriptors: shell.descriptors,
                        pane_payloads: shell.retained_pane_payloads,
                        retained_shell_presentation: shell.retained_shell_presentation,
                    });
                    self.presentation_dirty = false;
                    self.layout_dirty = false;
                    self.window_metrics_dirty = false;
                    self.workbench_window_bridge
                        .mark_host_projection_committed();
                    self.publish_refresh_invalidation_diagnostics();
                    return;
                }
            }
        }

        let pane_payloads = if let Some(retained) = shell.retained_pane_payloads.take() {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.window_metrics.pane_payload_cache_hit_count",
                1
            );
            retained
        } else {
            if window_metrics_target {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.window_metrics.pane_payload_cache_miss_count",
                    1
                );
            }
            Arc::new(self.collect_host_lifecycle_pane_payloads(&shell.model, &shell.chrome))
        };
        let retained_shell_presentation = self.apply_recompute_presentation(
            &shell.model,
            &shell.chrome,
            &shell.geometry,
            &pane_payloads,
            componentized_workbench_layout_frames,
            &floating_window_projection_bundle,
            reuse_shell_layout,
        );
        self.sync_recompute_viewport_surfaces(componentized_workbench_layout_frames);
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_native_window_presenters"
            );
            self.sync_native_window_presenters(
                &shell.model,
                &shell.chrome,
                &shell.geometry,
                &pane_payloads.preset_names,
                &pane_payloads.ui_asset_panes,
                &pane_payloads.animation_panes,
                &pane_payloads.runtime_diagnostics,
                &floating_window_projection_bundle,
            );
        }
        self.sync_recompute_pointer_surfaces(
            &shell.model,
            &shell.chrome,
            &pane_payloads.preset_names,
            window_metrics_target,
        );
        self.floating_window_projection_bundle = floating_window_projection_bundle;
        self.shell_geometry = Some(shell.geometry.clone());
        self.committed_shell_state = Some(committed_shell_state::CommittedShellState {
            layout: shell.layout,
            chrome: shell.chrome,
            model: shell.model,
            geometry: shell.geometry,
            layout_frames: componentized_workbench_layout_frames,
            descriptors: shell.descriptors,
            pane_payloads: Some(pane_payloads),
            retained_shell_presentation,
        });
        self.presentation_dirty = false;
        self.layout_dirty = false;
        self.window_metrics_dirty = false;
        self.workbench_window_bridge
            .mark_host_projection_committed();
    }
}

#[cfg(test)]
#[path = "tests/recompute.rs"]
mod tests;
