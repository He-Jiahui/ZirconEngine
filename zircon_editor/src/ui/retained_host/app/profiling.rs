pub(super) const PERFORMANCE_TIMELINE_ACTION_CONTROL_ID: &str = "PerformanceTimelineCaptureControl";

mod actions;
mod diagnostics;
#[cfg(feature = "profiling")]
mod snapshot_merge;
