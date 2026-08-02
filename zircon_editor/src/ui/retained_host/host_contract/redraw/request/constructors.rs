use crate::ui::retained_host::ui_perf::{
    UiPerfCounter, UiPerfScenario, current_ui_perf_scenario, record_ui_perf_counter,
};

use super::super::super::data::FrameRect;
use super::super::super::frame_geometry::visible_frame;
use super::HostRedrawRequest;

impl HostRedrawRequest {
    pub(crate) fn none() -> Self {
        Self::None
    }

    pub(in crate::ui::retained_host::host_contract) fn full_frame() -> Self {
        Self::full_frame_for_scenario(current_ui_perf_scenario(), true)
    }

    pub(crate) fn full_frame_for_scenario(scenario: UiPerfScenario, frame_update: bool) -> Self {
        record_ui_perf_counter(scenario, UiPerfCounter::RedrawFullFrame, 1.0);
        Self::Full {
            frame_update,
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
                frame,
                frame_update,
                scenario,
            }
        } else {
            Self::None
        }
    }
}
