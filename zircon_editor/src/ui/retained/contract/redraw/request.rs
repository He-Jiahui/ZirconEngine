#[path = "request/constructors.rs"]
mod constructors;
#[path = "request/merge.rs"]
mod merge;
#[path = "request/query.rs"]
mod query;
#[path = "request/ready_present.rs"]
mod ready_present;

use crate::ui::retained_host::ui_perf::UiPerfScenario;

use super::HostDamageRegion;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum HostRedrawRequest {
    None,
    RuntimeTick,
    FrameUpdate {
        runtime_tick: bool,
        scenario: UiPerfScenario,
        interactive_frame_update: bool,
    },
    Full {
        runtime_tick: bool,
        frame_update: bool,
        interactive_frame_update: bool,
        scenario: UiPerfScenario,
    },
    Region {
        damage: HostDamageRegion,
        runtime_tick: bool,
        frame_update: bool,
        interactive_frame_update: bool,
        scenario: UiPerfScenario,
    },
}
