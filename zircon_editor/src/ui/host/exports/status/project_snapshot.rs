use std::sync::Arc;

use super::super::reports::EditorPluginStatusReport;

/// Immutable plugin status published by the editor manager.
///
/// It holds either the builtin baseline or the current project's native load report. The host
/// publishes a new snapshot at lifecycle boundaries, so stable retained reads clone only this
/// `Arc`.
#[derive(Debug)]
pub(in crate::ui::host) struct ProjectPluginStatusSnapshot {
    report: Arc<EditorPluginStatusReport>,
}

impl ProjectPluginStatusSnapshot {
    pub(in crate::ui::host) fn new(report: EditorPluginStatusReport) -> Self {
        Self {
            report: Arc::new(report),
        }
    }

    pub(in crate::ui::host) fn report(&self) -> &Arc<EditorPluginStatusReport> {
        &self.report
    }
}
