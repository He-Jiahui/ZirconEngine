use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

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
        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
        .read_external(PostProcessGraphResourceNames::SHADOW_ATLAS)
        .read_buffer(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
        .read_buffer(PostProcessGraphResourceNames::LIGHT_ZBINS)
        .read_buffer(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
        .read_external(PostProcessGraphResourceNames::FINAL_COLOR)
        .write_texture(PostProcessGraphResourceNames::SCENE_COLOR)],
    )
}
