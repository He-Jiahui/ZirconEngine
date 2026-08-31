use crate::ui::retained_host::ui_perf::{
    current_ui_perf_scenario, record_ui_perf_counter, UiPerfCounter, UiPerfScenario,
};

use super::super::super::data::FrameRect;
use super::super::super::frame_geometry::visible_frame;
use super::super::HostDamageRegion;
use super::HostRedrawRequest;

impl HostRedrawRequest {
    pub(crate) fn none() -> Self {
        Self::None
    }

    pub(crate) fn frame_update_only_for_scenario(scenario: UiPerfScenario) -> Self {
        Self::FrameUpdate {
            scenario,
            interactive_frame_update: false,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn full_frame() -> Self {
        Self::full_frame_for_scenario(current_ui_perf_scenario(), true)
    }

    pub(crate) fn full_frame_for_scenario(scenario: UiPerfScenario, frame_update: bool) -> Self {
        record_ui_perf_counter(scenario, UiPerfCounter::RedrawFullFrame, 1.0);
        Self::Full {
            frame_update,
            interactive_frame_update: false,
            scenario,
        }
    }

    pub(crate) fn region(frame: FrameRect) -> Self {
        Self::region_for_scenario(current_ui_perf_scenario(), frame)
    }

    pub(crate) fn region_for_scenario(scenario: UiPerfScenario, frame: FrameRect) -> Self {
        Self::region_for_scenario_with_frame_update(scenario, frame, false)
    }

    pub(crate) fn region_with_frame_update(frame: FrameRect) -> Self {
        Self::region_for_scenario_with_frame_update(current_ui_perf_scenario(), frame, true)
    }

    pub(crate) fn region_for_scenario_with_frame_update(
        scenario: UiPerfScenario,
        frame: FrameRect,
        frame_update: bool,
    ) -> Self {
        if visible_frame(&frame) {
            record_ui_perf_counter(scenario, UiPerfCounter::RedrawRegion, 1.0);
            Self::Region {
                damage: HostDamageRegion::from_frame(frame),
                frame_update,
                interactive_frame_update: false,
                scenario,
            }
        } else {
            Self::None
        }
    }

    pub(crate) fn into_present_retry(self, scenario: UiPerfScenario) -> Self {
        match self {
            Self::Full { .. } => Self::full_frame_for_scenario(scenario, false),
            Self::Region { damage, .. } => {
                record_ui_perf_counter(scenario, UiPerfCounter::RedrawRegion, 1.0);
                Self::Region {
                    damage,
                    frame_update: false,
                    interactive_frame_update: false,
                    scenario,
                }
            }
            Self::None | Self::FrameUpdate { .. } => Self::None,
        }
    }

    pub(crate) fn into_interactive_frame_update(self) -> Self {
        match self {
            Self::FrameUpdate { scenario, .. } => Self::FrameUpdate {
                scenario,
                interactive_frame_update: true,
            },
            Self::Full {
                frame_update,
                scenario,
                ..
            } => Self::Full {
                frame_update,
                interactive_frame_update: frame_update,
                scenario,
            },
            Self::Region {
                damage,
                frame_update,
                scenario,
                ..
            } => Self::Region {
                damage,
                frame_update,
                interactive_frame_update: frame_update,
                scenario,
            },
            Self::None => Self::None,
        }
    }
}
