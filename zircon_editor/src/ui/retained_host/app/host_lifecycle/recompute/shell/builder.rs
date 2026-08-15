use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::*;
use super::snapshot::RecomputeShellSnapshot;
use super::template_bridges::emit_template_bridge_layout_error;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry_with_region_defaults_and_scale_mode, ResolutionContext,
    ShellRegionId, WorkbenchSkeleton,
};
use crate::ui::workbench::model::WorkbenchViewModel;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) fn build_window_metrics_shell_snapshot(
        &mut self,
    ) -> Option<RecomputeShellSnapshot> {
        let committed = self.committed_shell_state.take()?;
        let token_region_preferred = self.physical_shell_token_region_defaults();
        let geometry = compute_workbench_shell_geometry_with_region_defaults_and_scale_mode(
            &committed.model,
            &committed.chrome,
            &committed.layout,
            &committed.descriptors,
            self.shell_size,
            self.shell_scale_factor,
            self.shell_scale_mode,
            &self.chrome_metrics,
            (!self.transient_region_preferred.is_empty())
                .then_some(&self.transient_region_preferred),
            Some(&token_region_preferred),
        );
        let componentized_workbench_layout_frames =
            self.recompute_shell_template_bridge_layout_frames(&committed.model);
        zircon_runtime::profile_counter!(
            "editor",
            "ui.window_metrics.committed_shell_stage_hit_count",
            1
        );
        Some(RecomputeShellSnapshot {
            layout: committed.layout,
            chrome: committed.chrome,
            model: committed.model,
            geometry,
            componentized_workbench_layout_frames,
            reuse_shell_layout: false,
            descriptors: committed.descriptors,
        })
    }

    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) fn build_recompute_shell_snapshot(
        &mut self,
        requested_shell_layout_reuse: bool,
    ) -> RecomputeShellSnapshot {
        let layout = {
            zircon_runtime::profile_scope!("editor", "retained_host", "recompute_read_layout");
            self.runtime.current_layout()
        };
        let descriptors = {
            zircon_runtime::profile_scope!("editor", "retained_host", "recompute_read_descriptors");
            self.runtime.descriptors()
        };
        let chrome = {
            zircon_runtime::profile_scope!("editor", "retained_host", "recompute_build_chrome");
            self.build_chrome()
        };
        record_current_ui_perf_counter(UiPerfCounter::WorkbenchModelBuildCount, 1.0);
        let model = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_build_workbench_model"
            );
            let context = self.runtime.project_command_eval_snapshot(&chrome);
            let commands = self.runtime.commands().lock();
            WorkbenchViewModel::build_with_context(&commands, &chrome, &context)
        };
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_prepare_workbench_chrome_state"
            );
            if let Err(error) = self
                .workbench_window_bridge
                .prepare_chrome_state_for_layout(&chrome)
            {
                emit_template_bridge_layout_error(
                    self.runtime.context().logs(),
                    "editor_workbench_template_bridge_chrome_state",
                    format!("Workbench template bridge chrome sync failed: {error}"),
                );
            }
        }
        let token_region_preferred = self.physical_shell_token_region_defaults();
        let geometry = {
            zircon_runtime::profile_scope!("editor", "retained_host", "recompute_shell_geometry");
            compute_workbench_shell_geometry_with_region_defaults_and_scale_mode(
                &model,
                &chrome,
                &layout,
                &descriptors,
                self.shell_size,
                self.shell_scale_factor,
                self.shell_scale_mode,
                &self.chrome_metrics,
                (!self.transient_region_preferred.is_empty())
                    .then_some(&self.transient_region_preferred),
                Some(&token_region_preferred),
            )
        };
        let reuse_shell_layout = requested_shell_layout_reuse
            && self
                .shell_geometry
                .as_ref()
                .is_some_and(|previous| previous.shares_mounted_layout_frames_with(&geometry));
        if requested_shell_layout_reuse && !reuse_shell_layout {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.layout_cache_geometry_fallback_count",
                1
            );
        }
        let componentized_workbench_layout_frames = if reuse_shell_layout {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.layout_cache_hit_count",
                1
            );
            self.workbench_window_bridge.layout_frames()
        } else {
            self.recompute_shell_template_bridge_layout_frames(&model)
        };
        RecomputeShellSnapshot {
            layout,
            chrome,
            model,
            geometry,
            componentized_workbench_layout_frames,
            reuse_shell_layout,
            descriptors,
        }
    }

    fn physical_shell_token_region_defaults(&mut self) -> BTreeMap<ShellRegionId, f32> {
        let resolution = ResolutionContext::from_physical_size_with_scale_mode(
            self.shell_size,
            self.shell_scale_factor,
            self.shell_scale_mode,
        );
        self.shell_token_region_defaults()
            .iter()
            .map(|(&region, &logical_extent)| (region, resolution.to_physical(logical_extent)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn shell_geometry_uses_the_declared_root_scale_mode_for_every_conversion() {
        let source = include_str!("builder.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("test module should remain isolated from shell recompute production code");

        assert!(production.contains("ResolutionContext::from_physical_size_with_scale_mode"));
        assert!(production
            .contains("compute_workbench_shell_geometry_with_region_defaults_and_scale_mode"));
        assert!(production.matches("self.shell_scale_mode").count() >= 2);
    }

    #[test]
    fn stable_shell_content_reuses_mounted_layout_frames() {
        let source = include_str!("builder.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("builder production source");

        assert!(production.contains("requested_shell_layout_reuse"));
        assert!(production.contains("shares_mounted_layout_frames_with(&geometry)"));
        assert!(production.contains("self.workbench_window_bridge.layout_frames()"));
        assert!(production.contains("ui.shell_content.layout_cache_hit_count"));
        assert!(production.contains("ui.shell_content.layout_cache_geometry_fallback_count"));
        assert!(production.contains("reuse_shell_layout,"));
    }
}

impl RetainedEditorHost {
    fn shell_token_region_defaults(&mut self) -> &BTreeMap<ShellRegionId, f32> {
        let tokens = crate::ui::v2_design_tokens::active_editor_v2_design_tokens_snapshot();
        let refresh_required = match self.shell_token_region_defaults.as_ref() {
            Some((cached_tokens, _)) => !Arc::ptr_eq(cached_tokens, &tokens),
            None => true,
        };
        if refresh_required {
            self.shell_token_region_defaults = None;
        }
        let (_, extents) = self.shell_token_region_defaults.get_or_insert_with(|| {
            let extents = WorkbenchSkeleton::default_region_extents_from_tokens(&tokens);
            (tokens, extents)
        });
        extents
    }
}
