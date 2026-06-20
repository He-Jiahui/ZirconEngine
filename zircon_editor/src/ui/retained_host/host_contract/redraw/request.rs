use super::super::data::FrameRect;
use super::super::frame_geometry::{union_frame, visible_frame};
use crate::ui::retained_host::ui_perf::{
    current_ui_perf_scenario, record_ui_perf_counter, UiPerfCounter, UiPerfScenario,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum HostRedrawRequest {
    None,
    Full {
        frame_update: bool,
        scenario: UiPerfScenario,
    },
    Region {
        frame: FrameRect,
        frame_update: bool,
        scenario: UiPerfScenario,
    },
}

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

    pub(crate) fn merge(self, next: Self) -> Self {
        match (self, next) {
            (
                Self::Full {
                    frame_update,
                    scenario,
                },
                Self::Full {
                    frame_update: next,
                    scenario: next_scenario,
                },
            ) => Self::Full {
                frame_update: frame_update || next,
                scenario: if next { next_scenario } else { scenario },
            },
            (
                Self::Full {
                    frame_update,
                    scenario,
                },
                Self::Region {
                    frame_update: next,
                    scenario: next_scenario,
                    ..
                },
            ) => Self::Full {
                frame_update: frame_update || next,
                scenario: if next { next_scenario } else { scenario },
            },
            (
                Self::Region { frame_update, .. },
                Self::Full {
                    frame_update: next,
                    scenario,
                },
            ) => Self::Full {
                frame_update: frame_update || next,
                scenario,
            },
            (
                Self::Region {
                    frame: current,
                    frame_update,
                    scenario,
                },
                Self::Region {
                    frame: next,
                    frame_update: next_update,
                    scenario: next_scenario,
                },
            ) => Self::Region {
                frame: union_frame(&current, &next),
                frame_update: frame_update || next_update,
                scenario: if next_update { next_scenario } else { scenario },
            },
            (Self::None, next @ Self::Full { .. }) => next,
            (Self::None, next @ Self::Region { .. }) => next,
            (current, Self::None) => current,
        }
    }
}
