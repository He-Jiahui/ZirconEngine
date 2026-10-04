use super::{render_pass_stage::RenderPassStage, renderer_feature_asset::RendererFeatureAsset};

/// Renderer 的阶段顺序与启用的 feature 清单；编译入口据此筛选 pass 并建立执行图。
/// `stages` 保持作者声明顺序，`features` 的重复项由管线校验在编译前拒绝。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RendererAsset {
    pub name: String,
    pub stages: Vec<RenderPassStage>,
    pub features: Vec<RendererFeatureAsset>,
}
