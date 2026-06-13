use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{QueueLane, RenderGraphComputeWorkload};
use crate::{FrameHistoryBinding, FrameHistorySlot};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use super::compute_workload::{
    HZB_BUILD_PIPELINE_LABEL, HZB_BUILD_WORKGROUP_SIZE,
    HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE, HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE,
    HZB_OCCLUSION_CULL_PIPELINE_LABEL, HZB_OCCLUSION_CULL_WORKGROUP_SIZE,
    HZB_OCCLUSION_DRAW_COUNT_RESOURCE, HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE,
    HZB_OCCLUSION_STATS_RESOURCE, HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
};

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "hzb",
        vec!["view".to_string(), "visibility".to_string()],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::HzbFurthest,
        )],
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "hzb-occlusion-cull",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("visibility.hzb-occlusion-cull")
            .with_side_effects()
            .with_compute_workload(RenderGraphComputeWorkload::indirect_args(
                HZB_OCCLUSION_CULL_PIPELINE_LABEL,
                HZB_OCCLUSION_CULL_WORKGROUP_SIZE,
            ))
            .read_texture(PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST)
            .read_external(HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE)
            .read_external(HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE)
            .write_storage_external(HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE)
            .write_storage_external(HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE)
            .write_storage_external(HZB_OCCLUSION_DRAW_COUNT_RESOURCE)
            .write_storage_external(HZB_OCCLUSION_STATS_RESOURCE),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::AmbientOcclusion,
                "hzb-build",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("visibility.hzb-build")
            .with_side_effects()
            .with_compute_workload(RenderGraphComputeWorkload::hzb_furthest(
                HZB_BUILD_PIPELINE_LABEL,
                HZB_BUILD_WORKGROUP_SIZE,
            ))
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_storage_texture(PostProcessGraphResourceNames::HZB_FURTHEST),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::render_feature_pass_descriptor::{
        RenderFeatureResourceAccess, RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
    };
    use super::*;

    #[test]
    fn hzb_occlusion_cull_declares_execution_owned_external_buffers() {
        let descriptor = descriptor();
        let pass = descriptor
            .stage_passes
            .iter()
            .find(|pass| pass.pass_name == "hzb-occlusion-cull")
            .expect("hzb occlusion cull pass");

        assert!(pass.resources.iter().any(|resource| {
            resource.name == HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE
                && resource.kind == RenderFeatureResourceKind::External
                && resource.access == RenderFeatureResourceAccess::Read
        }));
        assert!(pass.resources.iter().any(|resource| {
            resource.name == HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE
                && resource.kind == RenderFeatureResourceKind::External
                && resource.access == RenderFeatureResourceAccess::Read
        }));
        for name in [
            HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE,
            HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
            HZB_OCCLUSION_DRAW_COUNT_RESOURCE,
            HZB_OCCLUSION_STATS_RESOURCE,
        ] {
            assert!(pass.resources.iter().any(|resource| {
                resource.name == name
                    && resource.kind == RenderFeatureResourceKind::External
                    && resource.access == RenderFeatureResourceAccess::Write
                    && resource.write_mode == RenderFeatureResourceWriteMode::Storage
            }));
        }
    }
}
