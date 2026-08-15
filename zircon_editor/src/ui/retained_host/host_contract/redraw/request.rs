mod constructors;
mod merge;
mod query;

use crate::ui::retained_host::ui_perf::UiPerfScenario;

use super::super::data::FrameRect;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum HostRedrawRequest {
    None,
    FrameUpdate {
        scenario: UiPerfScenario,
    },
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
