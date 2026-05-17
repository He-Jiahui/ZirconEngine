use crate::core::framework::render::{CorePipelineKind, RenderPhase, RenderPipelineHandle};

use crate::graphics::feature::BuiltinRenderFeature;
use crate::graphics::pipeline::declarations::{
    RenderPassStage, RenderPipelineAsset, RendererAsset, RendererFeatureAsset,
};

impl RenderPipelineAsset {
    pub fn default_deferred() -> Self {
        Self {
            handle: RenderPipelineHandle::new(2),
            name: "deferred".to_string(),
            core_pipeline: CorePipelineKind::Core3d,
            phase_mapping: vec![
                RenderPhase::Prepass,
                RenderPhase::Shadow,
                RenderPhase::Deferred,
                RenderPhase::AlphaMask3d,
                RenderPhase::Transparent3d,
                RenderPhase::PostProcess,
                RenderPhase::Ui,
                RenderPhase::Overlay,
                RenderPhase::Debug,
            ],
            renderer: RendererAsset {
                name: "default-deferred".to_string(),
                stages: vec![
                    RenderPassStage::DepthPrepass,
                    RenderPassStage::Shadow,
                    RenderPassStage::Deferred,
                    RenderPassStage::AlphaMask3d,
                    RenderPassStage::AmbientOcclusion,
                    RenderPassStage::Lighting,
                    RenderPassStage::Transparent3d,
                    RenderPassStage::PostProcess,
                    RenderPassStage::Ui,
                    RenderPassStage::Overlay,
                    RenderPassStage::Debug,
                ],
                features: vec![
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::DeferredGeometry),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::Shadows),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::ClusteredLighting),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::DeferredLighting),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::Bloom),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::ColorGrading),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::HistoryResolve),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::Ui),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::DebugOverlay),
                ],
            },
        }
    }
}
