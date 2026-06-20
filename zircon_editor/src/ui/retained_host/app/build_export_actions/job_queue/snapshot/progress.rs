use super::DesktopExportProgressSnapshot;

impl DesktopExportProgressSnapshot {
    pub(in crate::ui::retained_host::app::build_export_actions::job_queue) fn from_report(
        progress: crate::ui::host::EditorExportBuildProgress,
    ) -> Self {
        Self {
            stage: progress.stage,
            percent: progress.percent,
            message: progress.message,
        }
    }
}
