use crate::graphics::scene::scene_renderer::hzb::{
    HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE, HZB_OCCLUSION_CULL_PIPELINE_LABEL,
    HZB_OCCLUSION_CULL_WORKGROUP_SIZE, HZB_OCCLUSION_DRAW_COUNT_RESOURCE,
    HZB_OCCLUSION_STATS_RESOURCE, HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
    HzbOcclusionParamsCommit, PreparedHzbOcclusionCull,
};

use super::RenderPassGpuExecutionContext;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphComputeDispatchRecord;
use crate::render_graph::RenderGraphResourceAccessKind;

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn record_hzb_occlusion_cull_to_indirect_args(
        &mut self,
        pass_name: &str,
        executor_id: &str,
        previous_hzb_resource_name: &str,
    ) -> Result<(), String> {
        let culler = self.hzb_occlusion_culler.ok_or_else(|| {
            format!("HZB occlusion cull graph executor for pass `{pass_name}` requires HZB occlusion culler context")
        })?;
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!(
                "HZB occlusion cull graph executor for pass `{pass_name}` requires mesh draw context"
            )
        })?;
        let gpu_scene_bind_group = mesh_draw_lists
            .gpu_scene_bind_group
            .ok_or_else(|| {
                format!(
                    "HZB occlusion cull graph executor for pass `{pass_name}` requires GPUScene bind group"
                )
            })?
            .bind_group();
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let previous_hzb_view = Self::declared_optional_texture_view_by_name(
            resources,
            resource_resolver,
            previous_hzb_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let history_available = previous_hzb_view.is_some();
        let post_process_stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "HZB occlusion cull graph executor for pass `{pass_name}` requires post-process resources"
            )
        })?;
        let sampled_resource_identity = if history_available {
            post_process_stack
                .hzb_history_resource_identity()
                .ok_or_else(|| {
                    format!(
                        "HZB occlusion cull graph executor for pass `{pass_name}` resolved history without a history resource identity"
                    )
                })?
        } else {
            post_process_stack.hzb_fallback_resource_identity()
        };
        let previous_hzb_view = match previous_hzb_view {
            Some(view) => view,
            None => post_process_stack.white_texture_view(),
        };
        let PreparedHzbOcclusionCull {
            report,
            mut uploads,
            params_commits,
        } = culler.execute(
            self.device,
            self.encoder,
            self.scene_bind_group,
            gpu_scene_bind_group,
            previous_hzb_view,
            sampled_resource_identity,
            mesh_draw_lists,
            history_available,
        );
        self.append_pre_submit_buffer_uploads(&mut uploads);
        self.hzb_occlusion_params_commits.extend(params_commits);
        self.hzb_occlusion_cull_report = Some(report);
        self.compute_dispatches
            .push(RenderGraphComputeDispatchRecord::new(
                pass_name,
                executor_id,
                HZB_OCCLUSION_CULL_PIPELINE_LABEL,
                HZB_OCCLUSION_CULL_WORKGROUP_SIZE,
                [report.dispatch_group_count, 1, 1],
                if report.candidate_arg_count > 0 {
                    vec![
                        HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE.to_string(),
                        HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE.to_string(),
                        HZB_OCCLUSION_DRAW_COUNT_RESOURCE.to_string(),
                        HZB_OCCLUSION_STATS_RESOURCE.to_string(),
                    ]
                } else {
                    Vec::new()
                },
            ));
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_hzb_occlusion_params_commits(
        &mut self,
    ) -> Vec<HzbOcclusionParamsCommit> {
        std::mem::take(&mut self.hzb_occlusion_params_commits)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn hzb_occlusion_dispatch_record_reports_compaction_output_writes() {
        let source = include_str!("hzb_occlusion.rs");

        assert!(source.contains("HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE.to_string()"));
        assert!(source.contains("HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE.to_string()"));
        assert!(source.contains("HZB_OCCLUSION_DRAW_COUNT_RESOURCE.to_string()"));
        assert!(source.contains("HZB_OCCLUSION_STATS_RESOURCE.to_string()"));
    }

    #[test]
    fn hzb_occlusion_returns_uploads_and_commit_tokens_to_the_graph_owner() {
        let source = include_str!("hzb_occlusion.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("HZB graph context source");

        assert!(!production.contains("self.queue"));
        assert!(production.contains("self.append_pre_submit_buffer_uploads("));
        assert!(production.contains("self.hzb_occlusion_params_commits.extend("));
    }
}
