use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{QueueLane, RenderGraphComputeWorkload};

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use super::compute_workload::{
    CLUSTERED_LIGHTING_PIPELINE_LABEL, CLUSTERED_LIGHTING_WORKGROUP_SIZE,
};

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "clustered_lighting",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Lighting,
            "clustered-light-culling",
            QueueLane::AsyncCompute,
        )
        .with_executor_id("lighting.clustered-cull")
        .with_compute_workload(RenderGraphComputeWorkload::cluster_grid(
            CLUSTERED_LIGHTING_PIPELINE_LABEL,
            CLUSTERED_LIGHTING_WORKGROUP_SIZE,
        ))
        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
        .write_buffer(PostProcessGraphResourceNames::LIGHT_LIST)],
    )
}
