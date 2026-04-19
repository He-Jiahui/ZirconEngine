use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor() -> RenderFeatureDescriptor
{
    RenderFeatureDescriptor::new(
        "mesh",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "depth-prepass",
                QueueLane::Graphics,
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Opaque,
                "opaque-mesh",
                QueueLane::Graphics,
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent,
                "transparent-mesh",
                QueueLane::Graphics,
            ),
        ],
    )
}
