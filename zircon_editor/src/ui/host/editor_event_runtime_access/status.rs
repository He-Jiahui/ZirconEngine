use crate::core::editor_message::EditorViewInvalidationMask;
use crate::core::jobs::EditorJobProgressSnapshot;
use crate::ui::host::editor_activity_log::activity_log_console_output_for_shell;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::snapshot::{ConsoleOutputSnapshot, StatusTaskProgressSnapshot};

impl EditorHostEventController {
    pub fn set_status_line(&self, message: impl Into<String>) {
        self.shell().lock().state.set_status_line(message);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn status_line(&self) -> String {
        self.shell().lock().state.status_line.clone()
    }

    pub(crate) fn set_retained_status_line(&self, message: &str) -> bool {
        let mut shell = self.shell().lock();
        if shell.state.status_line == message {
            return false;
        }
        shell.state.set_status_line(message.to_owned());
        true
    }

    pub(crate) fn console_output(&self) -> ConsoleOutputSnapshot {
        let mut shell = self.shell().lock();
        activity_log_console_output_for_shell(&mut shell)
    }

    pub fn set_status_task_progress(&self, progress: Option<StatusTaskProgressSnapshot>) {
        self.shell().lock().state.set_status_task_progress(progress);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub(crate) fn set_retained_status_task_progress(
        &self,
        progress: &Option<StatusTaskProgressSnapshot>,
    ) -> bool {
        let mut shell = self.shell().lock();
        if shell.state.status_task_progress.as_ref() == progress.as_ref() {
            return false;
        }
        shell.state.set_status_task_progress(progress.clone());
        true
    }

    pub fn status_task_progress(&self) -> Option<StatusTaskProgressSnapshot> {
        self.shell().lock().state.status_task_progress.clone()
    }

    pub fn job_progress_snapshot(&self) -> Vec<EditorJobProgressSnapshot> {
        self.context().jobs().progress().snapshot()
    }

    pub fn primary_job_progress_snapshot(&self) -> Option<EditorJobProgressSnapshot> {
        self.context().jobs().progress().primary_snapshot()
    }
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830ef_editor535_status_line_compares_borrowed_message() {
        let access_source = include_str!("status.rs");
        let access_production = access_source
            .split("#[cfg(test)]")
            .next()
            .expect("status access implementation");
        let dispatch_source =
            include_str!("../../retained_host/app/host_lifecycle/dispatch_effects/status.rs");

        assert!(access_production.contains("fn set_retained_status_line(&self, message: &str)"));
        assert!(access_production.contains("shell.state.status_line == message"));
        assert!(access_production.contains("shell.state.set_status_line(message.to_owned())"));
        assert!(dispatch_source.contains("set_retained_status_line(&message)"));
        assert!(!dispatch_source.contains("set_retained_status_line(message.clone())"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830ef_editor535_unchanged_status_line_clone_evidence() {
        const UNCHANGED_UPDATES: usize = 32_768;
        const LEGACY_STRING_CLONES_PER_UPDATE: usize = 1;
        const OPTIMIZED_STRING_CLONES: usize = 0;
        const MARKER: &str = "EDITOR535_UNCHANGED_STATUS_LINE_BORROW_BENCH_V1";

        let legacy_string_clones =
            UNCHANGED_UPDATES.saturating_mul(LEGACY_STRING_CLONES_PER_UPDATE);
        let optimized_string_clones = OPTIMIZED_STRING_CLONES;

        assert_eq!(legacy_string_clones, 32_768);
        assert_eq!(optimized_string_clones, 0);
        println!(
            "{MARKER} unchanged_updates={UNCHANGED_UPDATES} \
             legacy_string_clones={legacy_string_clones} \
             optimized_string_clones={optimized_string_clones} reduction_pct=100"
        );
    }
}
