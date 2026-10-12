pub(super) const PERFORMANCE_TIMELINE_ACTION_CONTROL_ID: &str = "PerformanceTimelineCaptureControl";

#[path = "profiling/actions.rs"]
mod actions;
#[path = "profiling/diagnostics.rs"]
mod diagnostics;
#[cfg(feature = "profiling")]
#[path = "profiling/snapshot_merge.rs"]
mod snapshot_merge;
