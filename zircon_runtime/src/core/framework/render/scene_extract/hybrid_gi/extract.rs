use serde::{Deserialize, Serialize};

use super::{
    RenderHybridGiDebugView, RenderHybridGiMode, RenderHybridGiProfile, RenderHybridGiQuality,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RenderHybridGiExtract {
    pub enabled: bool,
    pub mode: RenderHybridGiMode,
    pub profile: RenderHybridGiProfile,
    pub quality: RenderHybridGiQuality,
    pub trace_budget: u32,
    pub card_budget: u32,
    pub voxel_budget: u32,
    pub debug_view: RenderHybridGiDebugView,
}

impl Default for RenderHybridGiExtract {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: RenderHybridGiMode::DynamicOnly,
            profile: RenderHybridGiProfile::Custom,
            quality: RenderHybridGiQuality::Medium,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            debug_view: RenderHybridGiDebugView::None,
        }
    }
}
