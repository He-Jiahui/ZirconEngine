use super::{render_pass_stage::RenderPassStage, renderer_feature_asset::RendererFeatureAsset};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RendererAsset {
    pub name: String,
    pub stages: Vec<RenderPassStage>,
    pub features: Vec<RendererFeatureAsset>,
}
