mod constructors;
mod status;
mod target;

use std::path::PathBuf;

pub(in crate::ui::retained_host::app) use target::apply_summary_to_target;

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
