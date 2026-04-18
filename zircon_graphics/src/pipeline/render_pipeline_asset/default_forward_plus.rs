use zircon_framework::render::RenderPipelineHandle;

use crate::feature::BuiltinRenderFeature;
use crate::pipeline::declarations::{
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
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::VirtualGeometry,
                        enabled: true,
                    },
                    RendererFeatureAsset {
                        feature: BuiltinRenderFeature::Mesh,
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
