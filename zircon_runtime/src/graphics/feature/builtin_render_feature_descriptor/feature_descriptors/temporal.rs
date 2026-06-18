use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "temporal",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "velocity-object",
                QueueLane::Graphics,
            )
            .with_executor_id("temporal.velocity-object")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_VELOCITY,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "velocity-camera",
                QueueLane::Graphics,
            )
            .with_executor_id("temporal.velocity-camera")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_VELOCITY,
                RenderGraphAttachmentOps::load_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "taa-reactive-mask-clear",
                QueueLane::Graphics,
            )
            .with_executor_id("temporal.taa-reactive-mask-clear")
            .write_texture_with_ops(
                PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
                RenderGraphAttachmentOps::clear_store(),
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
                RenderGraphAttachmentOps::load_store(),
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
            .read_external_texture(PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS)
            .read_texture(PostProcessGraphResourceNames::TAA_REACTIVE_MASK)
            .write_external_texture(PostProcessGraphResourceNames::TAA_HISTORY_CURRENT)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::TAA_OUTPUT,
                RenderGraphAttachmentOps::clear_store(),
            ),
        ],
    )
}
