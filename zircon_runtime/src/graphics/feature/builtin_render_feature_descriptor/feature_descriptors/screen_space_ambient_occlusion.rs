use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::{ComputePassDescriptor, ComputeShaderSource};
use crate::graphics::{FrameHistoryBinding, FrameHistorySlot};
use crate::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, PassFlags, QueueLane, RenderGraphComputeDispatchExtent,
};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use super::compute_workload::SSAO_WORKGROUP_SIZE;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    let compute_pass = ComputePassDescriptor::new(
        "ssao-evaluate",
        RenderPassStage::AmbientOcclusion,
        QueueLane::AsyncCompute,
        ComputeShaderSource::builtin_wgsl(
            "zircon-ssao-pipeline",
            include_str!("../../../scene/scene_renderer/post_process/shaders/ssao.wgsl"),
        ),
        "cs_main",
        SSAO_WORKGROUP_SIZE,
        vec![
            BindingSchemaEntry::new(
                0,
                PostProcessGraphResourceNames::SCENE_DEPTH,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                1,
                PostProcessGraphResourceNames::GBUFFER_NORMAL,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                2,
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_AMBIENT_OCCLUSION,
                ComputeBindingKind::SampledTexture,
            ),
            BindingSchemaEntry::new(
                3,
                PostProcessGraphResourceNames::SSAO_PARAMS,
                ComputeBindingKind::UniformBuffer,
            ),
            BindingSchemaEntry::new(
                4,
                PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
                ComputeBindingKind::StorageTextureWrite,
            ),
            BindingSchemaEntry::new(
                5,
                PostProcessGraphResourceNames::HZB_FURTHEST,
                ComputeBindingKind::SampledTexture,
            )
            .with_texture_full_mip_chain(),
        ],
        RenderGraphComputeDispatchExtent::PerPixel {
            target: PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string(),
            local_size: [SSAO_WORKGROUP_SIZE[0], SSAO_WORKGROUP_SIZE[1]],
        },
        PassFlags::default(),
    );

    RenderFeatureDescriptor::new(
        "screen_space_ambient_occlusion",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::AmbientOcclusion,
            "ssao-evaluate",
            QueueLane::AsyncCompute,
        )
        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
        .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
        .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
        .read_external_texture(PostProcessGraphResourceNames::HISTORY_PREVIOUS_AMBIENT_OCCLUSION)
        .read_external_buffer(PostProcessGraphResourceNames::SSAO_PARAMS)
        .write_storage_external_texture(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
        .with_compute_pass(compute_pass)],
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::render_feature_pass_descriptor::{
        RenderFeatureResourceAccess, RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
    };
    use super::*;
    use crate::render_graph::RenderGraphExternalResourceBinding;

    #[test]
    fn ssao_uses_the_generic_compute_executor_with_dynamic_graph_resources() {
        let descriptor = descriptor();
        let pass = descriptor
            .stage_passes
            .iter()
            .find(|pass| pass.pass_name == "ssao-evaluate")
            .expect("SSAO evaluate pass");
        let compute = pass
            .compute_pass
            .as_ref()
            .expect("generic compute descriptor");

        assert_eq!(pass.executor_id.as_str(), "compute.generic");
        assert_eq!(compute.workgroup_size, SSAO_WORKGROUP_SIZE);
        let RenderGraphComputeDispatchExtent::PerPixel { target, local_size } = &compute.dispatch
        else {
            panic!("SSAO must use a per-pixel compute dispatch");
        };
        assert_eq!(target, PostProcessGraphResourceNames::AMBIENT_OCCLUSION);
        assert_eq!(
            local_size,
            &[SSAO_WORKGROUP_SIZE[0], SSAO_WORKGROUP_SIZE[1]]
        );
        assert_eq!(compute.bindings.len(), 6);
        assert!(compute.bindings.iter().any(|binding| {
            binding.binding == 5
                && binding.resource == PostProcessGraphResourceNames::HZB_FURTHEST
                && binding.texture_full_mip_chain
        }));

        assert!(pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::HISTORY_PREVIOUS_AMBIENT_OCCLUSION
                && resource.kind == RenderFeatureResourceKind::External
                && resource.access == RenderFeatureResourceAccess::Read
                && resource.external_binding
                    == RenderGraphExternalResourceBinding::report_only_texture()
        }));
        assert!(pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SSAO_PARAMS
                && resource.kind == RenderFeatureResourceKind::External
                && resource.access == RenderFeatureResourceAccess::Read
                && resource.external_binding
                    == RenderGraphExternalResourceBinding::report_only_buffer()
        }));
        assert!(pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION
                && resource.kind == RenderFeatureResourceKind::External
                && resource.access == RenderFeatureResourceAccess::Write
                && resource.write_mode == RenderFeatureResourceWriteMode::Storage
                && resource.external_binding
                    == RenderGraphExternalResourceBinding::report_only_texture()
        }));
    }
}
