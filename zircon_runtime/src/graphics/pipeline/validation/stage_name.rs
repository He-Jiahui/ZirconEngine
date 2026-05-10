use crate::graphics::pipeline::declarations::RenderPassStage;

pub(in crate::graphics::pipeline) fn stage_name(stage: RenderPassStage) -> &'static str {
    match stage {
        RenderPassStage::DepthPrepass => "depth_prepass",
        RenderPassStage::Shadow => "shadow",
        RenderPassStage::Deferred => "deferred",
        RenderPassStage::AmbientOcclusion => "ambient_occlusion",
        RenderPassStage::Lighting => "lighting",
        RenderPassStage::Opaque2d => "opaque_2d",
        RenderPassStage::AlphaMask2d => "alpha_mask_2d",
        RenderPassStage::Transparent2d => "transparent_2d",
        RenderPassStage::Opaque3d => "opaque_3d",
        RenderPassStage::AlphaMask3d => "alpha_mask_3d",
        RenderPassStage::Transparent3d => "transparent_3d",
        RenderPassStage::Opaque => "opaque",
        RenderPassStage::Transparent => "transparent",
        RenderPassStage::PostProcess => "post_process",
        RenderPassStage::Ui => "ui",
        RenderPassStage::Overlay => "overlay",
        RenderPassStage::Debug => "debug",
    }
}
