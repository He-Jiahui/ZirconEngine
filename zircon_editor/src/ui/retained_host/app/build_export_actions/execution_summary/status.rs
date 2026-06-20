use super::{DesktopExportExecutionState, DesktopExportExecutionSummary};

impl DesktopExportExecutionSummary {
    pub(in crate::ui::retained_host::app) fn fatal(&self) -> bool {
        !self.fatal_diagnostics.is_empty()
    }

    pub(in crate::ui::retained_host::app) fn status_message(&self) -> String {
        if self.state == DesktopExportExecutionState::Cancelled {
            return format!(
                "Export {} cancelled -> {}",
                self.profile_name,
                self.output_root.display()
            );
        }
        if self.fatal() {
            return format!(
                "Export {} failed: {}",
                self.profile_name,
                self.fatal_diagnostics.join("; ")
            );
        }
        let cargo = if self.invoked_cargo {
            "cargo build invoked"
        } else {
            "cargo build skipped"
        };
        format!(
            "Export {} finished: {} files, {} native packages, {cargo} -> {}",
            self.profile_name,
            self.generated_files,
            self.copied_packages,
            self.output_root.display()
        )
    }
}

pub(super) fn summary_status_label(summary: &DesktopExportExecutionSummary) -> &'static str {
    match summary.state {
        DesktopExportExecutionState::Exported => "Exported",
        DesktopExportExecutionState::Failed => "Failed",
        DesktopExportExecutionState::Cancelled => "Cancelled",
    }
}

pub(super) fn summary_pane_diagnostics(summary: &DesktopExportExecutionSummary) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Last export output: {}",
        summary.output_root.display()
    ));
    lines.push(if summary.invoked_cargo {
        "Last export invoked Cargo".to_string()
    } else {
        "Last export skipped Cargo".to_string()
    });
    lines.extend(summary.fatal_diagnostics.iter().cloned());
    lines.extend(summary.diagnostics.iter().take(6).cloned());
    lines.join("\n")
}
