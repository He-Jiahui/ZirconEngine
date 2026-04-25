use crate::core::framework::render::RenderPipelineHandle;

use crate::graphics::feature::BuiltinRenderFeature;
use crate::graphics::pipeline::declarations::{
    RenderPassStage, RenderPipelineAsset, RendererAsset, RendererFeatureAsset,
};

impl RenderPipelineAsset {
    pub fn default_deferred() -> Self {
        Self {
            handle: RenderPipelineHandle::new(2),
            name: "deferred".to_string(),
            renderer: RendererAsset {
                name: "default-deferred".to_string(),
                stages: vec![
                    RenderPassStage::DepthPrepass,
                    RenderPassStage::Shadow,
                    RenderPassStage::GBuffer,
                    RenderPassStage::AmbientOcclusion,
                    RenderPassStage::Lighting,
                    RenderPassStage::Transparent,
                    RenderPassStage::PostProcess,
                    RenderPassStage::Overlay,
                ],
                features: vec![
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::VirtualGeometry),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::DeferredGeometry),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::Shadows),
                    RendererFeatureAsset::builtin(
                        BuiltinRenderFeature::ScreenSpaceAmbientOcclusion,
                    ),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::ClusteredLighting),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::DeferredLighting),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::GlobalIllumination),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::Particle),
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
