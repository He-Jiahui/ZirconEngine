mod constructors;
mod merge;
mod query;

use crate::ui::retained_host::ui_perf::UiPerfScenario;

use super::HostDamageRegion;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum HostRedrawRequest {
    None,
    FrameUpdate {
        scenario: UiPerfScenario,
        interactive_frame_update: bool,
    },
    Full {
        frame_update: bool,
        interactive_frame_update: bool,
        scenario: UiPerfScenario,
    },
    Region {
        damage: HostDamageRegion,
        frame_update: bool,
        interactive_frame_update: bool,
        scenario: UiPerfScenario,
    },
}
