use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "particle",
        vec![
            "view".to_string(),
            "particles".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Transparent,
            "particle-render",
            QueueLane::Graphics,
        )
        .with_executor_id("particle.transparent")
        .read_texture("scene-depth")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}
