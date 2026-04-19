use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;
use crate::{FrameHistoryBinding, FrameHistorySlot};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor() -> RenderFeatureDescriptor
{
    RenderFeatureDescriptor::new(
        "history_resolve",
        vec!["view".to_string(), "post_process".to_string()],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::SceneColor,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "history-resolve",
            QueueLane::Graphics,
        )],
    )
}
