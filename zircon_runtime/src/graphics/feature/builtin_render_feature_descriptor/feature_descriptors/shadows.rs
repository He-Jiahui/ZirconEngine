use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

// 阴影 pass 将清除后的 shadow atlas 作为 required external texture 发布给几何和光照消费者。
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
            "shadow-atlas",
            QueueLane::Graphics,
        )
        .with_executor_id("shadow.atlas")
        .write_required_external_texture_with_ops(
            PostProcessGraphResourceNames::SHADOW_ATLAS,
            RenderGraphAttachmentOps::clear_store(),
        )],
    )
}

#[cfg(test)]
#[path = "tests/shadows.rs"]
mod tests;
