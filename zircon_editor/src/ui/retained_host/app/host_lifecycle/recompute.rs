use super::super::*;

mod floating_projection;
mod invalidation;
mod pointer_surfaces;
mod presentation;
mod shell;
mod viewport_surfaces;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn recompute_if_dirty(&mut self) {
        if !self.presentation_dirty && !self.layout_dirty && !self.window_metrics_dirty {
            return;
        }
        zircon_runtime::profile_scope!("editor", "retained_host", "recompute_if_dirty");

        let Some(recompute_decision) = self.begin_recompute_invalidation_phase() else {
            return;
        };
        let paint_only_reasons = recompute_decision.paint_only_reasons;

        if paint_only_reasons.requires_layout() {
            // layout-affecting invalidations still require the full shell recompute below;
            // pure paint-only reasons stay eligible for a lighter path after the shared
            // root frames are already stable.
        }
        let mut shell = self.build_recompute_shell_snapshot();
        let floating_window_projection_bundle =
            self.build_recompute_floating_window_projection_bundle(&shell.model);
        let componentized_workbench_layout_frames = shell.componentized_workbench_layout_frames;
        self.sync_recompute_viewport_and_pointer_layouts(
            &mut shell.model,
            &mut shell.chrome,
            componentized_workbench_layout_frames,
            &floating_window_projection_bundle,
        );

        let pane_payloads = self.collect_host_lifecycle_pane_payloads(&shell.model, &shell.chrome);
        self.apply_recompute_presentation(
            &shell.model,
            &shell.chrome,
            &shell.geometry,
            &pane_payloads,
            componentized_workbench_layout_frames,
            &floating_window_projection_bundle,
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
        );
        self.floating_window_projection_bundle = floating_window_projection_bundle;
        self.shell_geometry = Some(shell.geometry);
        self.presentation_dirty = false;
        self.layout_dirty = false;
        self.window_metrics_dirty = false;
        if !paint_only_reasons.is_empty() && !paint_only_reasons.requires_layout() {
            self.render_dirty = false;
        }
    }
}
