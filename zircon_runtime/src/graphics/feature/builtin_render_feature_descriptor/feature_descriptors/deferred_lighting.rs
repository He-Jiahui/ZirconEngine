use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

// 延迟光照读取 G-buffer、阴影图与聚簇光照缓冲，并以 clear/store 写入场景颜色，作为后续颜色链的输入。
pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "deferred_lighting",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Lighting,
            "deferred-lighting",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.deferred")
        .read_texture(PostProcessGraphResourceNames::GBUFFER_ALBEDO)
        .read_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
        .read_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
        .read_texture(PostProcessGraphResourceNames::GBUFFER_EMISSIVE)
        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
        .read_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS)
        .read_buffer(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
        .read_buffer(PostProcessGraphResourceNames::LIGHT_ZBINS)
        .read_buffer(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
        .write_texture_with_ops(
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderGraphAttachmentOps::clear_store(),
        )],
    )
}

#[cfg(test)]
#[path = "tests/deferred_lighting.rs"]
mod tests;
