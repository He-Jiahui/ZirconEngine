use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "shadows",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Shadow,
            "shadow-map",
            QueueLane::Graphics,
        )
        .with_executor_id("shadow.map")
        .with_side_effects()
        .write_texture_with_ops(
            PostProcessGraphResourceNames::SHADOW_MAP,
            RenderGraphAttachmentOps::clear_store(),
        )],
    )
}
