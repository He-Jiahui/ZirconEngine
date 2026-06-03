use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::QueueLane;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "bloom",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "bloom-extract",
            QueueLane::Graphics,
        )
        .with_executor_id("post.bloom-extract")
        .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
        .write_external(PostProcessGraphResourceNames::BLOOM)],
    )
}
