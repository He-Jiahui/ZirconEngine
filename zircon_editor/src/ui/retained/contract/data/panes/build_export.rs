use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::TemplatePaneNodeData;

#[derive(Clone, Default)]
pub(crate) struct BuildExportTargetData {
    pub profile_name: SharedString,
    pub platform: SharedString,
    pub target_mode: SharedString,
    pub strategies: SharedString,
    pub status: SharedString,
    pub enabled_plugins: SharedString,
    pub linked_runtime_crates: SharedString,
    pub native_dynamic_packages: SharedString,
    pub generated_files: SharedString,
    pub diagnostics: SharedString,
    pub fatal: bool,
}

#[derive(Clone, Default)]
pub(crate) struct BuildExportPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub targets: ModelRc<BuildExportTargetData>,
    pub diagnostics: SharedString,
}
