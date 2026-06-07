use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::QueueLane;

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "mesh",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "preview-sky",
                QueueLane::Graphics,
            )
            .with_executor_id("sky.preview-scene-color")
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "depth-prepass",
                QueueLane::Graphics,
            )
            .with_executor_id("mesh.depth-prepass")
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Opaque3d,
                "opaque-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("mesh.opaque")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::SHADOW_MAP)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::AlphaMask3d,
                "alpha-mask-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("mesh.alpha-mask")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::SHADOW_MAP)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "transparent-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("mesh.transparent")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_texture(PostProcessGraphResourceNames::SHADOW_MAP)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
        ],
    )
}
