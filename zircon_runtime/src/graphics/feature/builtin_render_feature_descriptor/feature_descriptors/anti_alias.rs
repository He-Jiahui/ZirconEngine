use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::feature::RenderFeatureCapabilityRequirement;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::anti_alias::fxaa::{FXAA_EXECUTOR_ID, FXAA_PASS_NAME};
use crate::render_graph::QueueLane;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "anti_alias",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            FXAA_PASS_NAME,
            QueueLane::Graphics,
        )
        .with_executor_id(FXAA_EXECUTOR_ID)
        .with_side_effects()
        .read_external(PostProcessGraphResourceNames::FINAL_COMPOSITED)
        .write_external(PostProcessGraphResourceNames::FINAL_COLOR)],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::ScreenSpaceAntiAlias)
}
