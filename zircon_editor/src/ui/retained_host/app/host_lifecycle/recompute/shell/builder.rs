use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::*;
use super::snapshot::RecomputeShellSnapshot;
use crate::ui::retained_host::ui_perf::{UiPerfCounter, record_current_ui_perf_counter};
use crate::ui::workbench::autolayout::{
    ResolutionContext, ShellRegionId, WorkbenchSkeleton,
    compute_workbench_shell_geometry_with_region_defaults_and_scale_mode,
};
use crate::ui::workbench::model::WorkbenchViewModel;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) fn build_recompute_shell_snapshot(
        &mut self,
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
        let token_region_preferred: BTreeMap<_, _> = {
            let resolution = ResolutionContext::from_physical_size_with_scale_mode(
                self.shell_size,
                self.shell_scale_factor,
                self.shell_scale_mode,
            );
            self.shell_token_region_defaults()
                .iter()
                .map(|(&region, &logical_extent)| (region, resolution.to_physical(logical_extent)))
                .collect()
        };
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
        let componentized_workbench_layout_frames =
            self.recompute_shell_template_bridge_layout_frames(&model);
        RecomputeShellSnapshot {
            chrome,
            model,
            geometry,
            componentized_workbench_layout_frames,
        }
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
        assert!(
            production
                .contains("compute_workbench_shell_geometry_with_region_defaults_and_scale_mode")
        );
        assert!(production.matches("self.shell_scale_mode").count() >= 2);
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
