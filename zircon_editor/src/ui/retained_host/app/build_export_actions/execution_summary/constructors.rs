use std::path::PathBuf;

use super::{DesktopExportExecutionState, DesktopExportExecutionSummary};

impl DesktopExportExecutionSummary {
    pub(in crate::ui::retained_host::app) fn from_report(
        output_root: PathBuf,
        mut report: crate::ui::host::EditorExportBuildReport,
    ) -> Self {
        let state = if let Some(reason) = report.failure_reason() {
            if !report
                .fatal_diagnostics
                .iter()
                .any(|message| message == reason)
            {
                report.fatal_diagnostics.push(reason.to_owned());
            }
            DesktopExportExecutionState::Failed
        } else {
            DesktopExportExecutionState::Exported
        };
        Self {
            profile_name: report.plan.profile.name,
            output_root,
            state,
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
}

#[cfg(test)]
#[path = "constructors/tests/astra_outcome_tests.rs"]
mod astra_outcome_tests;
