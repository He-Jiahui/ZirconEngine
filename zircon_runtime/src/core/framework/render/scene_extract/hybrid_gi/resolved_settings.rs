use super::{
    RenderHybridGiFallbackReason, RenderHybridGiMode, RenderHybridGiProfile, RenderHybridGiQuality,
};

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
