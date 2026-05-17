use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::QueueLane;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "ui",
        vec!["view".to_string(), "ui".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Ui,
            "runtime-ui",
            QueueLane::Graphics,
        )
        .with_executor_id("ui.screen-space")
        .with_side_effects()
        .read_external(PostProcessGraphResourceNames::FINAL_COLOR)
        .write_external("viewport-output")],
    )
}
