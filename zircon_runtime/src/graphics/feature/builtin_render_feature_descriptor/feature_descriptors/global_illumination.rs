use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;
use crate::{FrameHistoryBinding, FrameHistorySlot};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor() -> RenderFeatureDescriptor
{
    RenderFeatureDescriptor::new(
        "global_illumination",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Lighting,
            "hybrid-gi-resolve",
            QueueLane::Graphics,
        )],
    )
}
