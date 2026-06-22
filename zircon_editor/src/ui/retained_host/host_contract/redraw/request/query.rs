use crate::ui::retained_host::ui_perf::{current_ui_perf_scenario, UiPerfScenario};

use super::super::super::data::FrameRect;
use super::HostRedrawRequest;

impl HostRedrawRequest {
    pub(crate) fn request_redraw(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub(crate) fn requires_frame_update(&self) -> bool {
        matches!(
            self,
            Self::Full {
                frame_update: true,
                ..
            } | Self::Region {
                frame_update: true,
                ..
            }
        )
    }

    pub(crate) fn damage_region(&self) -> Option<&FrameRect> {
        match self {
            Self::Region { frame, .. } => Some(frame),
            Self::None | Self::Full { .. } => None,
        }
    }

    pub(crate) fn scenario(&self) -> UiPerfScenario {
        match self {
            Self::Full { scenario, .. } | Self::Region { scenario, .. } => *scenario,
            Self::None => current_ui_perf_scenario(),
        }
    }
}
