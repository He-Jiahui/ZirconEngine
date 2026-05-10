use crate::core::framework::render::{CorePipelineKind, RenderPhase, RenderPipelineHandle};

use crate::graphics::feature::BuiltinRenderFeature;
use crate::graphics::pipeline::declarations::{
    RenderPassStage, RenderPipelineAsset, RendererAsset, RendererFeatureAsset,
};

impl RenderPipelineAsset {
    pub fn default_core2d() -> Self {
        Self {
            handle: RenderPipelineHandle::new(3),
            name: "core-2d".to_string(),
            core_pipeline: CorePipelineKind::Core2d,
            phase_mapping: vec![
                RenderPhase::Opaque2d,
                RenderPhase::AlphaMask2d,
                RenderPhase::Transparent2d,
                RenderPhase::PostProcess,
                RenderPhase::Ui,
                RenderPhase::Overlay,
                RenderPhase::Debug,
            ],
            renderer: RendererAsset {
                name: "default-core-2d".to_string(),
                stages: vec![
                    RenderPassStage::Opaque2d,
                    RenderPassStage::AlphaMask2d,
                    RenderPassStage::Transparent2d,
                    RenderPassStage::PostProcess,
                    RenderPassStage::Ui,
                    RenderPassStage::Overlay,
                    RenderPassStage::Debug,
                ],
                features: vec![
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::PostProcess)
                        .with_enabled(false),
                    RendererFeatureAsset::builtin(BuiltinRenderFeature::DebugOverlay),
                ],
            },
        }
    }
}
