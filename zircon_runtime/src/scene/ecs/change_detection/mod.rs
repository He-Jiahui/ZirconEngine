#[path = "change_tick.rs"]
mod change_tick;
#[path = "change_tick_window.rs"]
mod change_tick_window;
#[path = "component_mutation.rs"]
mod component_mutation;
#[path = "component_ticks.rs"]
mod component_ticks;
#[path = "stats.rs"]
mod stats;
#[path = "wrappers.rs"]
mod wrappers;

pub use change_tick::ChangeTick;
pub use change_tick_window::ChangeTickWindow;
pub(crate) use component_mutation::{
    ComponentMutationRecord, ComponentMutationRecorder, ComponentMutationSink,
};
pub use component_ticks::ComponentTicks;
pub use stats::{
    ChangeDetectionScanStats, ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC,
    ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC, ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC,
};
pub use wrappers::{Mut, Ref};
