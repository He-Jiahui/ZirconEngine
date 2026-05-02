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
