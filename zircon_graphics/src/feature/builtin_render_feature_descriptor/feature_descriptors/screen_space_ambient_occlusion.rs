use zircon_render_graph::QueueLane;

use crate::pipeline::RenderPassStage;
use crate::{FrameHistoryBinding, FrameHistorySlot};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::feature::builtin_render_feature_descriptor) fn descriptor() -> RenderFeatureDescriptor
{
    RenderFeatureDescriptor::new(
        "screen_space_ambient_occlusion",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::AmbientOcclusion,
            "ssao-evaluate",
            QueueLane::AsyncCompute,
        )],
    )
}
