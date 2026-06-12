use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{QueueLane, RenderGraphComputeWorkload};
use crate::{FrameHistoryBinding, FrameHistorySlot};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use super::compute_workload::{HZB_BUILD_PIPELINE_LABEL, HZB_BUILD_WORKGROUP_SIZE};

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "hzb",
        vec!["view".to_string(), "visibility".to_string()],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::HzbFurthest,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::AmbientOcclusion,
            "hzb-build",
            QueueLane::AsyncCompute,
        )
        .with_executor_id("visibility.hzb-build")
        .with_side_effects()
        .with_compute_workload(RenderGraphComputeWorkload::hzb_furthest(
            HZB_BUILD_PIPELINE_LABEL,
            HZB_BUILD_WORKGROUP_SIZE,
        ))
        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
        .write_storage_texture(PostProcessGraphResourceNames::HZB_FURTHEST)],
    )
}
