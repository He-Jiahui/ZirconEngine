use zircon_render_server::RenderPipelineHandle;

use crate::feature::BuiltinRenderFeature;
use crate::pipeline::declarations::{
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
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::VirtualGeometry,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::DeferredGeometry,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::Shadows,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::ScreenSpaceAmbientOcclusion,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::ClusteredLighting,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::DeferredLighting,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::GlobalIllumination,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::Particle,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::Bloom,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::ReflectionProbes,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::BakedLighting,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::PostProcess,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::ColorGrading,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::HistoryResolve,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::DebugOverlay,
                        enabled: true,
                    },
                ],
            },
        }
    }
}
