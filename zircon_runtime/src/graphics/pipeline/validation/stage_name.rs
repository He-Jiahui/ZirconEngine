use crate::graphics::pipeline::declarations::RenderPassStage;

pub(in crate::graphics::pipeline) fn stage_name(stage: RenderPassStage) -> &'static str {
    match stage {
        RenderPassStage::DepthPrepass => "depth_prepass",
        RenderPassStage::Shadow => "shadow",
        RenderPassStage::GBuffer => "gbuffer",
        RenderPassStage::AmbientOcclusion => "ambient_occlusion",
        RenderPassStage::Lighting => "lighting",
        RenderPassStage::Opaque => "opaque",
        RenderPassStage::Transparent => "transparent",
        RenderPassStage::PostProcess => "post_process",
        RenderPassStage::Overlay => "overlay",
    }
}
