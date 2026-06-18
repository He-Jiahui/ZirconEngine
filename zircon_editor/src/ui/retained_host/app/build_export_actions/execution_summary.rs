use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) enum DesktopExportExecutionState {
    Exported,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct DesktopExportExecutionSummary {
    pub(in crate::ui::retained_host::app) profile_name: String,
    pub(in crate::ui::retained_host::app) output_root: PathBuf,
    pub(in crate::ui::retained_host::app) state: DesktopExportExecutionState,
    pub(in crate::ui::retained_host::app) invoked_cargo: bool,
    pub(in crate::ui::retained_host::app) generated_files: usize,
    pub(in crate::ui::retained_host::app) copied_packages: usize,
    pub(in crate::ui::retained_host::app) diagnostics: Vec<String>,
    pub(in crate::ui::retained_host::app) fatal_diagnostics: Vec<String>,
}

impl DesktopExportExecutionSummary {
    pub(in crate::ui::retained_host::app) fn from_report(
        output_root: PathBuf,
        report: crate::ui::host::EditorExportBuildReport,
    ) -> Self {
        Self {
            profile_name: report.plan.profile.name,
            output_root,
            state: DesktopExportExecutionState::Exported,
            invoked_cargo: report.invoked_cargo,
            generated_files: report.generated_files.len(),
            copied_packages: report.copied_packages.len(),
            diagnostics: report.diagnostics,
            fatal_diagnostics: report.fatal_diagnostics,
        }
    }

    pub(in crate::ui::retained_host::app) fn failed(
        profile_name: impl Into<String>,
        output_root: PathBuf,
        error: String,
    ) -> Self {
        Self {
            profile_name: profile_name.into(),
            output_root,
            state: DesktopExportExecutionState::Failed,
            invoked_cargo: false,
            generated_files: 0,
            copied_packages: 0,
            diagnostics: Vec::new(),
            fatal_diagnostics: vec![error],
        }
    }

    pub(in crate::ui::retained_host::app) fn cancelled(
        profile_name: impl Into<String>,
        output_root: PathBuf,
        reason: String,
    ) -> Self {
        Self {
            profile_name: profile_name.into(),
            output_root,
            state: DesktopExportExecutionState::Cancelled,
            invoked_cargo: false,
            generated_files: 0,
            copied_packages: 0,
            diagnostics: vec![reason],
            fatal_diagnostics: Vec::new(),
        }
    }

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

    fn status_label(&self) -> &'static str {
        match self.state {
            DesktopExportExecutionState::Exported => "Exported",
            DesktopExportExecutionState::Failed => "Failed",
            DesktopExportExecutionState::Cancelled => "Cancelled",
        }
    }

    fn pane_diagnostics(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "Last export output: {}",
            self.output_root.display()
        ));
        lines.push(if self.invoked_cargo {
            "Last export invoked Cargo".to_string()
        } else {
            "Last export skipped Cargo".to_string()
        });
        lines.extend(self.fatal_diagnostics.iter().cloned());
        lines.extend(self.diagnostics.iter().take(6).cloned());
        lines.join("\n")
    }
}

pub(in crate::ui::retained_host::app) fn apply_summary_to_target(
    target: &mut crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData,
    summary: &DesktopExportExecutionSummary,
) {
    target.status = summary.status_label().into();
    target.generated_files = summary.generated_files.to_string().into();
    target.native_dynamic_packages = summary.copied_packages.to_string().into();
    target.diagnostics = summary.pane_diagnostics().into();
    target.fatal = target.fatal || summary.fatal();
}
