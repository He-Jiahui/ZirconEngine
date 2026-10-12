use crate::ui::retained_host::ui_perf::{current_ui_perf_scenario, UiPerfScenario};

use super::super::super::data::FrameRect;
use super::super::HostDamageRegionMetrics;
use super::HostRedrawRequest;

impl HostRedrawRequest {
    pub(crate) fn request_redraw(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub(crate) fn requires_frame_update(&self) -> bool {
        matches!(
            self,
            Self::FrameUpdate { .. }
                | Self::Full {
                    frame_update: true,
                    ..
                }
                | Self::Region {
                    frame_update: true,
                    ..
                }
        )
    }

    pub(crate) fn prefers_interactive_frame_update(&self) -> bool {
        matches!(
            self,
            Self::FrameUpdate {
                interactive_frame_update: true,
                ..
            } | Self::Full {
                interactive_frame_update: true,
                ..
            } | Self::Region {
                interactive_frame_update: true,
                ..
            }
        )
    }

    pub(crate) fn requires_present(&self) -> bool {
        matches!(self, Self::Full { .. } | Self::Region { .. })
    }

    pub(crate) fn damage_region(&self) -> Option<&FrameRect> {
        match self {
            Self::Region { damage, .. } => Some(damage.bounding_frame()),
            Self::None | Self::FrameUpdate { .. } | Self::Full { .. } => None,
        }
    }

    pub(crate) fn damage_region_metrics(&self) -> Option<HostDamageRegionMetrics> {
        match self {
            Self::Region { damage, .. } => Some(damage.metrics()),
            Self::None | Self::FrameUpdate { .. } | Self::Full { .. } => None,
        }
    }

    pub(crate) fn scenario(&self) -> UiPerfScenario {
        match self {
            Self::FrameUpdate { scenario, .. }
            | Self::Full { scenario, .. }
            | Self::Region { scenario, .. } => *scenario,
            Self::None => current_ui_perf_scenario(),
        }
    }
}
