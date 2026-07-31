use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::{FrameHistoryBinding, FrameHistorySlot};
use crate::render_graph::{QueueLane, RenderGraphComputeWorkload};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use super::compute_workload::{SSAO_PIPELINE_LABEL, SSAO_WORKGROUP_SIZE};

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor()
-> RenderFeatureDescriptor {
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
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::AmbientOcclusion,
                "ssao-evaluate",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("ao.ssao-evaluate")
            .with_compute_workload(RenderGraphComputeWorkload::viewport(
                SSAO_PIPELINE_LABEL,
                SSAO_WORKGROUP_SIZE,
            ))
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .read_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
            .write_storage_external_texture(PostProcessGraphResourceNames::AMBIENT_OCCLUSION),
        ],
    )
}
