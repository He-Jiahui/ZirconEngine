use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

use crate::graphics::pipeline::RenderPassStage;

use super::super::render_feature_descriptor::RenderFeatureDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;

// 延迟几何按深度、G-buffer、天空预览、透明网格的顺序声明资源生产和读取关系。
pub(in crate::graphics::feature::builtin_render_feature_descriptor) fn descriptor(
) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "deferred_geometry",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "depth-prepass",
                QueueLane::Graphics,
            )
            .with_executor_id("deferred.depth-prepass")
            .write_texture(PostProcessGraphResourceNames::SCENE_DEPTH),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Deferred,
                "gbuffer-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("deferred.gbuffer")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_ALBEDO)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_NORMAL)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            .write_texture(PostProcessGraphResourceNames::GBUFFER_EMISSIVE),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "preview-sky",
                QueueLane::Graphics,
            )
            .with_executor_id("sky.preview-scene-color")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_COLOR,
                RenderGraphAttachmentOps::load_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                "transparent-mesh",
                QueueLane::Graphics,
            )
            .with_executor_id("mesh.transparent")
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
            .read_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS)
            .write_texture(PostProcessGraphResourceNames::SCENE_COLOR),
        ],
    )
}

#[cfg(test)]
#[path = "tests/deferred_geometry.rs"]
mod tests;
