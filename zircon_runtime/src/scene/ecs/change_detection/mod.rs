mod change_tick;
mod change_tick_window;
mod component_mutation;
mod component_ticks;
mod stats;
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
