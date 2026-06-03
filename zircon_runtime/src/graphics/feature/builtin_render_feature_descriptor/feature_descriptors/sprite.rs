use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::QueueLane;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "sprite",
        vec![
            "view".to_string(),
            "sprites".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Opaque2d,
                "opaque-sprite",
                QueueLane::Graphics,
            )
            .with_executor_id("sprite.opaque")
            .with_side_effects()
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::AlphaMask2d,
                "alpha-mask-sprite",
                QueueLane::Graphics,
            )
            .with_executor_id("sprite.alpha-mask")
            .with_side_effects()
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent2d,
                "transparent-sprite",
                QueueLane::Graphics,
            )
            .with_executor_id("sprite.transparent")
            .with_side_effects()
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
        ],
    )
}
