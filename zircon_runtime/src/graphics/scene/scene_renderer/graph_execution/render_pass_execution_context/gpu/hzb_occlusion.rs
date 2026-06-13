use crate::graphics::scene::scene_renderer::hzb::{
    HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE, HZB_OCCLUSION_CULL_PIPELINE_LABEL,
    HZB_OCCLUSION_CULL_WORKGROUP_SIZE, HZB_OCCLUSION_DRAW_COUNT_RESOURCE,
    HZB_OCCLUSION_STATS_RESOURCE, HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
};

use super::RenderPassGpuExecutionContext;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphComputeDispatchRecord;

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
        let history_available = self.resources.has_texture_view(previous_hzb_resource_name);
        let previous_hzb_view = if history_available {
            self.resources
                .require_texture_view(previous_hzb_resource_name)?
        } else {
            self.post_process_stack
                .ok_or_else(|| {
                    format!(
                        "HZB occlusion cull graph executor for pass `{pass_name}` requires post-process fallback textures when previous HZB is unavailable"
                    )
                })?
                .white_texture_view()
        };
        let report = culler.execute(
            self.device,
            self.queue,
            self.encoder,
            self.scene_bind_group,
            gpu_scene_bind_group,
            previous_hzb_view,
            mesh_draw_lists,
            history_available,
        );
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
}
