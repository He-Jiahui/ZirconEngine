use crate::core::context::EditorContext;
use crate::core::logging::LogSeverityCounts;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StatusSignalsSnapshot {
    pub logs: LogSeverityCounts,
    pub active_notifications: usize,
}

impl StatusSignalsSnapshot {
    pub(crate) fn from_context(context: &EditorContext) -> Self {
        Self {
            logs: context.logs().severity_counts(),
            active_notifications: context.notifications().active_count(),
        }
    }
}
