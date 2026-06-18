use super::viewport_toolbar_projection::attach_viewport_toolbar_surface_frames_to_ui;
use super::*;
use crate::ui::retained_host::floating_window_projection::{
    build_floating_window_projection_bundle_with_shared_source,
    resolve_floating_window_projection_base_outer_frame,
    resolve_floating_window_projection_shared_source,
};
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, record_current_ui_perf_counter, time_ui_perf_scenario, UiPerfCounter,
    UiPerfScenario,
};
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};

mod dispatch_effects;
mod invalidation_bridge;
mod native_window_presenters;
mod pane_payloads;
mod recompute_viewport;
mod render_submission;
mod startup;

fn ui_frame_is_visible(frame: &UiFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}

impl RetainedEditorHost {
    pub(super) fn tick(&mut self) {
        zircon_runtime::profile_frame!("editor", "retained_host_tick");
        zircon_runtime::profile_scope!("editor", "retained_host", "tick");
        self.poll_desktop_export_jobs();
        self.poll_desktop_export_wizard_sessions();

        {
            let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::AssetRefresh);
            let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::AssetRefresh);
            if let Err(error) = self.refresh_project_assets() {
                self.set_status_line(error);
            }
        }

        {
            let frame_scenario = self.pending_ui_perf_scenario.take();
            let _frame_scenario_guard = frame_scenario.map(enter_ui_perf_scenario);
            if let Some(scenario) = frame_scenario {
                self.ui.mark_completed_frame_update_scenario(scenario);
            }

            self.sync_shell_size();
            self.recompute_if_dirty();

            self.submit_render_frame_if_dirty();
        }

        {
            let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::ViewportImage);
            let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::ViewportImage);
            self.poll_viewport_image_for_native_host();
        }
        if let Some(error) = self.viewport.take_error() {
            self.set_status_line(error);
            self.recompute_if_dirty();
        }
    }

    pub(super) fn refresh_ui(&mut self) {
        self.recompute_if_dirty();
    }

    pub(super) fn use_committed_pointer_layout(&self) {
        // Pointer routing must stay on the last committed bridge frames. Dirty
        // presentation/layout state is consumed by tick/refresh instead of
        // rebuilding the whole editor tree inside native pointer callbacks.
        self.publish_refresh_invalidation_diagnostics();
    }

    pub(super) fn build_chrome(&self) -> crate::ui::workbench::snapshot::EditorChromeSnapshot {
        record_current_ui_perf_counter(UiPerfCounter::ChromeSnapshotCount, 1.0);
        self.runtime.chrome_snapshot()
    }

    pub(super) fn sync_shell_size(&mut self) {
        let bootstrap = self.ui.get_host_window_bootstrap();
        let next = ShellSizePx::new(
            bootstrap.shell_frame.width.max(1.0),
            bootstrap.shell_frame.height.max(1.0),
        );
        if (next.width - self.shell_size.width).abs() <= 0.5
            && (next.height - self.shell_size.height).abs() <= 0.5
        {
            return;
        }
        self.shell_size = next;
        self.invalidate_host(
            HostInvalidationMask::WINDOW_METRICS.union(HostInvalidationMask::PRESENTATION_DATA),
        );
    }

    pub(super) fn recompute_if_dirty(&mut self) {
        if !self.presentation_dirty && !self.layout_dirty && !self.window_metrics_dirty {
            return;
        }
        zircon_runtime::profile_scope!("editor", "retained_host", "recompute_if_dirty");

        let pending_reasons = self.invalidation.take_recompute_reasons();
        let recompute_reasons = if pending_reasons.is_empty() {
            HostInvalidationMask::from_dirty_flags(
                self.layout_dirty,
                self.presentation_dirty,
                self.window_metrics_dirty,
                self.render_dirty,
            )
        } else {
            pending_reasons
        };
        let paint_only_reasons = recompute_reasons.intersection(
            HostInvalidationMask::PAINT_ONLY
                .union(HostInvalidationMask::POINTER_HOVER)
                .union(HostInvalidationMask::VIEWPORT_IMAGE),
        );
        let pure_paint_only = !paint_only_reasons.is_empty()
            && !recompute_reasons.requires_layout()
            && !recompute_reasons.requires_presentation()
            && !recompute_reasons.requires_window_metrics()
            && !recompute_reasons.requires_hit_test()
            && !recompute_reasons.requires_render();
        if pure_paint_only {
            record_current_ui_perf_counter(UiPerfCounter::ChromeCommandPatchCount, 1.0);
            self.presentation_dirty = false;
            self.layout_dirty = false;
            self.window_metrics_dirty = false;
            self.publish_refresh_invalidation_diagnostics();
            if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
                write_diagnostic_log(
                    "editor_host_invalidation",
                    format!(
                        "paint_only_fast_path reasons={} legacy_dirty_flags={{layout:{},presentation:{},window_metrics:{},render:{}}} {}",
                        recompute_reasons.summary(),
                        self.layout_dirty,
                        self.presentation_dirty,
                        self.window_metrics_dirty,
                        self.render_dirty,
                        self.invalidation.stats_summary()
                    ),
                );
            }
            return;
        }

        let slow_path_rebuild = self.invalidation.record_slow_path_rebuild();
        record_current_ui_perf_counter(UiPerfCounter::SlowPathRebuildCount, 1.0);
        record_current_ui_perf_counter(UiPerfCounter::ChromeCommandFullRebuildCount, 1.0);
        self.publish_refresh_invalidation_diagnostics();
        if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
            write_diagnostic_log(
                "editor_host_invalidation",
                format!(
                    "slow_path count={} reasons={} legacy_dirty_flags={{layout:{},presentation:{},window_metrics:{},render:{}}} {}",
                    slow_path_rebuild,
                    recompute_reasons.summary(),
                    self.layout_dirty,
                    self.presentation_dirty,
                    self.window_metrics_dirty,
                    self.render_dirty,
                    self.invalidation.stats_summary()
                ),
            );
        }

        let layout = {
            zircon_runtime::profile_scope!("editor", "retained_host", "recompute_read_layout");
            self.runtime.current_layout()
        };
        let descriptors = {
            zircon_runtime::profile_scope!("editor", "retained_host", "recompute_read_descriptors");
            self.runtime.descriptors()
        };
        let mut chrome = {
            zircon_runtime::profile_scope!("editor", "retained_host", "recompute_build_chrome");
            self.build_chrome()
        };
        record_current_ui_perf_counter(UiPerfCounter::WorkbenchModelBuildCount, 1.0);
        let mut model = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_build_workbench_model"
            );
            WorkbenchViewModel::build(&chrome)
        };
        if paint_only_reasons.requires_layout() {
            // layout-affecting invalidations still require the full shell recompute below;
            // pure paint-only reasons stay eligible for a lighter path after the shared
            // root frames are already stable.
        }
        let geometry = {
            zircon_runtime::profile_scope!("editor", "retained_host", "recompute_shell_geometry");
            compute_workbench_shell_geometry(
                &model,
                &chrome,
                &layout,
                &descriptors,
                self.shell_size,
                &self.chrome_metrics,
                if self.transient_region_preferred.is_empty() {
                    None
                } else {
                    Some(&self.transient_region_preferred)
                },
            )
        };
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_root_template_bridge"
            );
            let _ = self.template_bridge.recompute_layout_with_workbench_model(
                UiSize::new(self.shell_size.width, self.shell_size.height),
                &model,
                &self.chrome_metrics,
            );
        }
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_workbench_window_bridge"
            );
            let _ = self
                .workbench_window_bridge
                .recompute_layout_with_workbench_model(
                    UiSize::new(self.shell_size.width, self.shell_size.height),
                    &model,
                    &self.chrome_metrics,
                );
        }
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_floating_source_bridge"
            );
            let _ = self
                .floating_window_source_bridge
                .recompute_layout(UiSize::new(self.shell_size.width, self.shell_size.height));
        }
        let floating_window_shared_source = resolve_floating_window_projection_shared_source(
            &self.floating_window_source_bridge.source_frames(),
        );
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_sync_floating_bounds"
            );
            for (window_index, window) in model.floating_windows.iter().enumerate() {
                let frame = resolve_floating_window_projection_base_outer_frame(
                    window,
                    window_index,
                    floating_window_shared_source,
                );
                self.editor_manager.sync_native_window_projection_bounds(
                    &window.window_id,
                    [frame.x, frame.y, frame.width, frame.height],
                );
            }
        }
        let native_window_hosts = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_native_window_hosts"
            );
            self.editor_manager.native_window_hosts()
        };
        let floating_window_projection_bundle = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_floating_projection_bundle"
            );
            build_floating_window_projection_bundle_with_shared_source(
                &model,
                floating_window_shared_source,
                &self.chrome_metrics,
                &native_window_hosts,
            )
        };
        let componentized_workbench_layout_frames = self.workbench_window_bridge.layout_frames();
        self.sync_recompute_viewport_and_pointer_layouts(
            &mut model,
            &mut chrome,
            componentized_workbench_layout_frames,
            &floating_window_projection_bundle,
        );

        let pane_payloads = self.collect_host_lifecycle_pane_payloads(&model, &chrome);
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_apply_presentation"
            );
            let _ = self.workbench_window_bridge.sync_from_chrome(&chrome);
            let has_component_showcase_runtime =
                self.prepare_component_showcase_runtime_for_presentation(&model);
            let pane_template_runtime = if has_component_showcase_runtime {
                &self.component_showcase_runtime
            } else {
                self.builtin_template_runtime.as_ref()
            };
            apply_presentation(
                &self.ui,
                &model,
                &chrome,
                &geometry,
                &pane_payloads.preset_names,
                self.active_layout_preset.as_deref(),
                &pane_payloads.ui_asset_panes,
                &pane_payloads.animation_panes,
                Some(&pane_payloads.runtime_diagnostics),
                &pane_payloads.module_plugins,
                &pane_payloads.build_export,
                Some(self.template_bridge.host_projection()),
                Some(self.workbench_window_bridge.host_projection()),
                componentized_workbench_layout_frames,
                &floating_window_projection_bundle,
                Some(pane_template_runtime),
            );
        }
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_viewport_surfaces"
            );
            let document_viewport_toolbar_width = componentized_workbench_layout_frames
                .viewport_toolbar_frame
                .map(|frame| frame.width);
            attach_viewport_toolbar_surface_frames_to_ui(
                &self.ui,
                &mut self.viewport_toolbar_bridge,
                document_viewport_toolbar_width,
            );
            let world_space_ui_surfaces =
                crate::ui::retained_host::build_world_space_ui_surface_submissions_from_host_scene(
                    &self.ui.get_host_presentation().host_scene_data,
                );
            self.viewport
                .submit_world_space_ui_surfaces(world_space_ui_surfaces);
        }
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_native_window_presenters"
            );
            self.sync_native_window_presenters(
                &model,
                &chrome,
                &geometry,
                &pane_payloads.preset_names,
                &pane_payloads.ui_asset_panes,
                &pane_payloads.animation_panes,
                &pane_payloads.runtime_diagnostics,
                &floating_window_projection_bundle,
            );
        }
        {
            zircon_runtime::profile_scope!("editor", "retained_host", "recompute_pointer_surfaces");
            self.sync_menu_pointer_layout(&model, &chrome, &pane_payloads.preset_names);
            self.sync_welcome_recent_pointer_layout(&chrome);
            self.sync_hierarchy_pointer_layout(&chrome.scene_entries);
            self.sync_detail_pointer_layouts(&chrome);
            self.sync_asset_pointer_layouts(&chrome);
        }
        self.floating_window_projection_bundle = floating_window_projection_bundle;
        self.shell_geometry = Some(geometry);
        self.presentation_dirty = false;
        self.layout_dirty = false;
        self.window_metrics_dirty = false;
        if !paint_only_reasons.is_empty() && !paint_only_reasons.requires_layout() {
            self.render_dirty = false;
        }
    }

    fn publish_refresh_invalidation_diagnostics(&self) {
        self.ui
            .set_host_refresh_invalidation_diagnostics(self.invalidation.diagnostics_snapshot());
    }
}
