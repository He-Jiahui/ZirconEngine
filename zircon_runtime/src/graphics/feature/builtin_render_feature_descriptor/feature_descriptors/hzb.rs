use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::shader::hzb_build_dispatch_plan;
use crate::graphics::{FrameHistoryBinding, FrameHistorySlot};
use crate::render_graph::{
    QueueLane, RenderGraphComputeWorkload, RenderGraphResourceAccessIntent,
    RenderGraphShaderStages, RenderGraphTextureSubresourceRange,
};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use super::compute_workload::{
    HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE, HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE,
    HZB_OCCLUSION_CULL_PIPELINE_LABEL, HZB_OCCLUSION_CULL_WORKGROUP_SIZE,
    HZB_OCCLUSION_DRAW_COUNT_RESOURCE, HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE,
    HZB_OCCLUSION_STATS_RESOURCE, HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
};

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor()
-> RenderFeatureDescriptor {
    let hzb_build_dispatch = hzb_build_dispatch_plan();

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
            .read_persistent_external_texture_with_access(
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST,
                RenderGraphTextureSubresourceRange::full(),
                RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
            )
            .read_required_external_buffer(HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE)
            .read_required_external_buffer(HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE)
            .write_required_external_buffer(HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE)
            .write_required_external_buffer(HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE)
            .write_required_external_buffer(HZB_OCCLUSION_DRAW_COUNT_RESOURCE)
            .write_required_external_buffer(HZB_OCCLUSION_STATS_RESOURCE),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::AmbientOcclusion,
                "hzb-build",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("visibility.hzb-build")
            .with_compute_workload(RenderGraphComputeWorkload::from_shader_dispatch(
                hzb_build_dispatch,
            ))
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_persistent_storage_texture(PostProcessGraphResourceNames::HZB_FURTHEST),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::render_feature_pass_descriptor::{
        RenderFeatureResourceAccess, RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
    };
    use super::*;
    use crate::render_graph::{
        RenderGraphExternalResourceBinding, RenderGraphResourceAccessIntent,
        RenderGraphResourceAccessMetadata, RenderGraphResourceAccessRange, RenderGraphShaderStages,
        RenderGraphTextureSubresourceRange,
    };

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
                && resource.external_binding
                    == RenderGraphExternalResourceBinding::required_buffer()
        }));
        assert!(pass.resources.iter().any(|resource| {
            resource.name == HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE
                && resource.kind == RenderFeatureResourceKind::External
                && resource.access == RenderFeatureResourceAccess::Read
                && resource.external_binding
                    == RenderGraphExternalResourceBinding::required_buffer()
        }));
        assert!(pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST
                && resource.kind == RenderFeatureResourceKind::External
                && resource.access == RenderFeatureResourceAccess::Read
                && resource.external_binding
                    == RenderGraphExternalResourceBinding::report_only_texture()
                && resource.usage.persistent
                && resource.schema.is_none()
                && resource.access_metadata
                    == Some(RenderGraphResourceAccessMetadata::new(
                        RenderGraphResourceAccessRange::Texture(
                            RenderGraphTextureSubresourceRange::full(),
                        ),
                        RenderGraphResourceAccessIntent::sampled_texture(
                            RenderGraphShaderStages::COMPUTE,
                        ),
                    ))
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
                    && resource.external_binding
                        == RenderGraphExternalResourceBinding::required_buffer()
            }));
        }
    }

    #[test]
    fn hzb_build_retains_the_actual_cross_frame_history_copy_source() {
        let descriptor = descriptor();
        let pass = descriptor
            .stage_passes
            .iter()
            .find(|pass| pass.pass_name == "hzb-build")
            .expect("hzb build pass");

        assert!(!pass.flags.has_side_effects);
        assert!(pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::HZB_FURTHEST
                && resource.kind == RenderFeatureResourceKind::Texture
                && resource.access == RenderFeatureResourceAccess::Write
                && resource.write_mode == RenderFeatureResourceWriteMode::Storage
                && resource.usage.persistent
        }));
    }
}
