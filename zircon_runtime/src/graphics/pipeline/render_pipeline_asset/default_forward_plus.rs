use crate::core::framework::render::{CorePipelineKind, RenderPhase, RenderPipelineHandle};

use crate::graphics::feature::BuiltinRenderFeature;
use crate::graphics::pipeline::declarations::{
    RenderPassStage, RenderPipelineAsset, RendererAsset, RendererFeatureAsset,
};

impl RenderPipelineAsset {
    pub fn default_forward_plus() -> Self {
        Self {
            handle: RenderPipelineHandle::new(1),
            name: "forward-plus".to_string(),
            core_pipeline: CorePipelineKind::Core3d,
            phase_mapping: vec![
                RenderPhase::Prepass,
                RenderPhase::Shadow,
                RenderPhase::Opaque3d,
                RenderPhase::AlphaMask3d,
                RenderPhase::Transparent3d,
                RenderPhase::PostProcess,
                RenderPhase::Ui,
                RenderPhase::Overlay,
                RenderPhase::Debug,
            ],
            renderer: RendererAsset {
                name: "default-forward-plus".to_string(),
                stages: vec![
                    RenderPassStage::DepthPrepass,
                    RenderPassStage::Shadow,
                    RenderPassStage::AmbientOcclusion,
                    RenderPassStage::Lighting,
                    RenderPassStage::Opaque3d,
                    RenderPassStage::AlphaMask3d,
                    RenderPassStage::Transparent3d,
                    RenderPassStage::PostProcess,
                    RenderPassStage::Ui,
                    RenderPassStage::Overlay,
                    RenderPassStage::Debug,
                ],
                features: vec![
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::Shadows),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::ClusteredLighting),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::PostProcess),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::Bloom),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::ColorGrading),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::HistoryResolve),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::AntiAlias),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::Ui),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::DebugOverlay),
                ],
            },
        }
    }
}
