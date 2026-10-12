use super::{
    RenderHybridGiFallbackReason, RenderHybridGiMode, RenderHybridGiProfile, RenderHybridGiQuality,
};

/// provider 实际采用的 GI 配置及回退原因，供帧统计和编辑器诊断回显。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderHybridGiResolvedSettings {
    pub mode: RenderHybridGiMode,
    pub profile: RenderHybridGiProfile,
    pub quality: RenderHybridGiQuality,
    pub trace_budget: u32,
    pub card_budget: u32,
    pub voxel_budget: u32,
    pub fallback_reason: Option<RenderHybridGiFallbackReason>,
}
