use std::path::PathBuf;

use zircon_runtime::plugin::ExportBuildPlan;

use super::cargo_invocation::EditorExportCargoInvocation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorExportBuildReport {
    pub plan: ExportBuildPlan,
    pub invoked_cargo: bool,
    pub cargo_invocation: Option<EditorExportCargoInvocation>,
    pub native_cargo_invocations: Vec<EditorExportCargoInvocation>,
    pub generated_files: Vec<PathBuf>,
    pub copied_packages: Vec<PathBuf>,
    pub diagnostics: Vec<String>,
    pub fatal_diagnostics: Vec<String>,
}

impl EditorExportBuildReport {
    pub fn failure_reason(&self) -> Option<&str> {
        if let Some(reason) = self
            .fatal_diagnostics
            .first()
            .or(self.plan.fatal_diagnostics.first())
        {
            return Some(reason);
        }
        if let Some(missing) = self
            .plan
            .runtime_plugin_availability
            .missing_required
            .first()
        {
            return Some(&missing.reason);
        }
        if self.invoked_cargo != self.cargo_invocation.is_some() {
            return Some("export Cargo invocation receipt is missing or inconsistent");
        }
        if self
            .cargo_invocation
            .as_ref()
            .is_some_and(|invocation| !invocation.success || invocation.status_code != Some(0))
        {
            return Some("export Cargo build did not exit successfully");
        }
        if self
            .native_cargo_invocations
            .iter()
            .any(|invocation| !invocation.success || invocation.status_code != Some(0))
        {
            return Some("native plugin Cargo build did not exit successfully");
        }
        None
    }

    pub fn into_result(self) -> Result<Self, super::error::EditorExportBuildError> {
        if self.failure_reason().is_some() {
            Err(super::error::EditorExportBuildError::ReportFailed {
                report: Box::new(self),
            })
        } else {
            Ok(self)
        }
    }
}

#[cfg(test)]
#[path = "report/tests/astra_outcome_tests.rs"]
mod astra_outcome_tests;
