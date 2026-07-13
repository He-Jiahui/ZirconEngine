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
    let mut lines = vec![
        format!("Output: {}", snapshot.output_root.display()),
        format!("Progress: {phase}"),
    ];
    if let Some(progress) = &snapshot.progress {
        lines.push(progress_pane_diagnostic(progress));
    }
    lines.join("\n")
}

pub(super) fn progress_pane_diagnostic(progress: &DesktopExportProgressSnapshot) -> String {
    format!(
        "Stage: {}% {} - {}",
        progress.percent, progress.stage, progress.message
    )
}
