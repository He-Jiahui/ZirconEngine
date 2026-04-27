use serde::{Deserialize, Serialize};

use super::ExportGeneratedFile;
use crate::{ExportProfile, ProjectPluginSelection};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportBuildPlan {
    pub profile: ExportProfile,
    pub enabled_runtime_plugins: Vec<String>,
    pub linked_runtime_crates: Vec<String>,
    pub native_dynamic_packages: Vec<String>,
    pub generated_files: Vec<ExportGeneratedFile>,
    pub diagnostics: Vec<String>,
}

impl ExportBuildPlan {
    pub(super) fn new(
        profile: ExportProfile,
        enabled_plugins: &[&ProjectPluginSelection],
        linked_runtime_crates: Vec<String>,
        native_dynamic_packages: Vec<String>,
        generated_files: Vec<ExportGeneratedFile>,
    ) -> Self {
        Self {
            enabled_runtime_plugins: enabled_plugins
                .iter()
                .map(|selection| selection.id.clone())
                .collect(),
            profile,
            linked_runtime_crates,
            native_dynamic_packages,
            generated_files,
            diagnostics: Vec::new(),
        }
    }
}
