#[path = "decision/mod.rs"]
mod decision;
#[path = "slot.rs"]
mod slot;
#[path = "view.rs"]
mod view;
#[path = "window.rs"]
mod window;

#[cfg(test)]
#[path = "tests/capacity_tests.rs"]
mod capacity_tests;

pub(crate) use decision::{
    activity_decision_options, ActivityDecisionOption, ActivityDecisionSelectionError,
    ActivityDecisionSelectionId,
};
pub use slot::ActivityDrawerSlotPreference;
pub use view::ActivityViewDescriptor;
pub(crate) use view::{
    activity_log_views, activity_progress_views, activity_toast_views, ActivityLogView,
    ActivityProgressView, ActivityToastView,
};
pub use window::ActivityWindowDescriptor;
