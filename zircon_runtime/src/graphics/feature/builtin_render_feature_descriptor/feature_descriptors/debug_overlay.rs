use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "debug_overlay",
        vec!["view".to_string(), "debug".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Overlay,
            "overlay-gizmo",
            QueueLane::Graphics,
        )
        .with_executor_id("overlay.gizmo")
        .read_texture("scene-color")
        .write_external("viewport-output")],
    )
}
