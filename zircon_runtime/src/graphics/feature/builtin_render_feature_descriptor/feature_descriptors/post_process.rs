use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "post_process",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "depth-of-field-prepare",
                QueueLane::Graphics,
            )
            .with_executor_id("post.depth-of-field-prepare")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_external_with_ops(
                PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
                RenderGraphAttachmentOps::clear_store(),
            )
            .write_external_with_ops(
                PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "post-process",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .with_side_effects()
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_external(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
            .read_external(PostProcessGraphResourceNames::BLOOM)
            .write_external(PostProcessGraphResourceNames::FINAL_COMPOSITED)
            .write_external(PostProcessGraphResourceNames::FINAL_COLOR)
            .write_external(PostProcessGraphResourceNames::GLOBAL_ILLUMINATION),
        ],
    )
}
