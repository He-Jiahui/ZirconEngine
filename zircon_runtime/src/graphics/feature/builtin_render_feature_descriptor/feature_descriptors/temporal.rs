use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{
    QueueLane, RenderGraphAttachmentOps, RenderGraphResourceAccessIntent, RenderGraphShaderStages,
    RenderGraphTextureSubresourceRange, RenderResourceSchema, RenderTextureExtentPolicy,
    RenderTextureSchema,
};
use crate::rhi::{TextureFormat, TextureUsage};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

// Temporal 先合成相机与物体速度，再以带物理 schema 的持久 history 槽执行 TAA resolve。
pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "temporal",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "velocity-camera",
                QueueLane::Graphics,
            )
            .with_executor_id("temporal.velocity-camera")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_VELOCITY,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "velocity-object",
                QueueLane::Graphics,
            )
            .with_executor_id("temporal.velocity-object")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_VELOCITY,
                RenderGraphAttachmentOps::load_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "taa-reactive-mask-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("temporal.taa-reactive-mask-mesh")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "taa-resolve",
                QueueLane::Graphics,
            )
            .with_executor_id("temporal.taa-resolve")
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::SCENE_VELOCITY)
            .read_persistent_external_texture_with_schema_and_access(
                PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
                taa_history_schema(),
                RenderGraphTextureSubresourceRange::full(),
                RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT),
            )
            .read_texture(PostProcessGraphResourceNames::TAA_REACTIVE_MASK)
            .write_persistent_external_texture_with_schema_and_access(
                PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
                taa_history_schema(),
                RenderGraphTextureSubresourceRange::full(),
                RenderGraphResourceAccessIntent::ColorAttachment,
            )
            .write_texture_with_ops(
                PostProcessGraphResourceNames::TAA_OUTPUT,
                RenderGraphAttachmentOps::clear_store(),
            ),
        ],
    )
}

fn taa_history_schema() -> RenderResourceSchema {
    RenderResourceSchema::texture(
        RenderTextureSchema::new(
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT,
        )
        .with_extent(RenderTextureExtentPolicy::View),
    )
}

#[cfg(test)]
#[path = "tests/temporal.rs"]
mod tests;
