use serde::{Deserialize, Serialize};

use super::ExportGeneratedFile;
use crate::{plugin::ExportProfile, plugin::ProjectPluginSelection};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportBuildPlan {
    pub profile: ExportProfile,
    pub enabled_runtime_plugins: Vec<String>,
    pub linked_runtime_crates: Vec<String>,
    pub native_dynamic_packages: Vec<String>,
    pub generated_files: Vec<ExportGeneratedFile>,
    pub diagnostics: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fatal_diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExportLinkedRuntimeCrate {
    pub crate_name: String,
    pub path: String,
    pub registration_kind: ExportRuntimeCrateRegistrationKind,
}

impl ExportLinkedRuntimeCrate {
    pub fn runtime_plugin(crate_name: String, path: String) -> Self {
        Self {
            crate_name,
            path,
            registration_kind: ExportRuntimeCrateRegistrationKind::RuntimePlugin,
        }
    }

    pub fn runtime_feature(crate_name: String, path: String) -> Self {
        Self {
            crate_name,
            path,
            registration_kind: ExportRuntimeCrateRegistrationKind::RuntimeFeature,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportRuntimeCrateRegistrationKind {
    RuntimePlugin,
    RuntimeFeature,
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
            fatal_diagnostics: Vec::new(),
        }
    }

    pub fn has_fatal_diagnostics(&self) -> bool {
        !self.fatal_diagnostics.is_empty()
    }
}
