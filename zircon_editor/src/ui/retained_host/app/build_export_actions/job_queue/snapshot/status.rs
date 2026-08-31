use super::{DesktopExportJobPhase, DesktopExportJobSnapshot, DesktopExportProgressSnapshot};

pub(super) fn job_status_label(snapshot: &DesktopExportJobSnapshot) -> &'static str {
    match snapshot.phase {
        DesktopExportJobPhase::Queued => "Queued",
        DesktopExportJobPhase::Running => "Running",
        DesktopExportJobPhase::CancelRequested => "Cancel requested",
    }
}

pub(super) fn job_pane_diagnostics(snapshot: &DesktopExportJobSnapshot) -> String {
    let phase = match snapshot.phase {
        DesktopExportJobPhase::Queued => "waiting for the current desktop export job",
        DesktopExportJobPhase::Running => "export backend is running",
        DesktopExportJobPhase::CancelRequested => {
            "cancel requested; backend result will be ignored when it returns"
        }
    };
    match snapshot.progress.as_ref() {
        Some(progress) => format!(
            "Output: {}\nProgress: {phase}\nStage: {}% {} - {}",
            snapshot.output_root.display(),
            progress.percent,
            progress.stage,
            progress.message
        ),
        None => format!(
            "Output: {}\nProgress: {phase}",
            snapshot.output_root.display()
        ),
    }
}

pub(super) fn progress_pane_diagnostic(progress: &DesktopExportProgressSnapshot) -> String {
    format!(
        "Stage: {}% {} - {}",
        progress.percent, progress.stage, progress.message
    )
}

#[cfg(test)]
#[path = "status/direct_format_tests.rs"]
mod direct_format_tests;
