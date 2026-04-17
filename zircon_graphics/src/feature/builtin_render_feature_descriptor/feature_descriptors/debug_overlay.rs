use zircon_render_graph::QueueLane;

use crate::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::feature::builtin_render_feature_descriptor) fn descriptor() -> RenderFeatureDescriptor
{
    RenderFeatureDescriptor::new(
        "debug_overlay",
        vec!["view".to_string(), "debug".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Overlay,
            "overlay-gizmo",
            QueueLane::Graphics,
        )],
    )
}
