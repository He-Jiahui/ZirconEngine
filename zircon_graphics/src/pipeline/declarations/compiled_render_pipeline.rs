use zircon_render_graph::CompiledRenderGraph;
use zircon_render_server::RenderPipelineHandle;

use crate::extract::FrameHistoryBinding;

use super::{render_pass_stage::RenderPassStage, renderer_feature_asset::RendererFeatureAsset};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderPipeline {
    pub handle: RenderPipelineHandle,
    pub name: String,
    pub renderer_name: String,
    pub stages: Vec<RenderPassStage>,
    pub enabled_features: Vec<RendererFeatureAsset>,
    pub required_extract_sections: Vec<String>,
    pub history_bindings: Vec<FrameHistoryBinding>,
    pub graph: CompiledRenderGraph,
}
