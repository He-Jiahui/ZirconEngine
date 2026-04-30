use crate::core::framework::render::RenderPipelineHandle;

use crate::graphics::feature::BuiltinRenderFeature;
use crate::graphics::pipeline::declarations::{
    RenderPassStage, RenderPipelineAsset, RendererAsset, RendererFeatureAsset,
};

impl RenderPipelineAsset {
    pub fn default_forward_plus() -> Self {
        Self {
            handle: RenderPipelineHandle::new(1),
            name: "forward-plus".to_string(),
            renderer: RendererAsset {
                name: "default-forward-plus".to_string(),
                stages: vec![
                    RenderPassStage::DepthPrepass,
                    RenderPassStage::Shadow,
                    RenderPassStage::AmbientOcclusion,
                    RenderPassStage::Lighting,
                    RenderPassStage::Opaque,
                    RenderPassStage::Transparent,
                    RenderPassStage::PostProcess,
                    RenderPassStage::Overlay,
                ],
                features: vec![
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::Shadows),
                    RendererFeatureAsset::builtin(
                        BuiltinRenderFeature::ScreenSpaceAmbientOcclusion,
                    ),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::ClusteredLighting),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::Bloom),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::ReflectionProbes),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::BakedLighting),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::PostProcess),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::ColorGrading),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::HistoryResolve),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::DebugOverlay),
                ],
            },
        }
    }
}
