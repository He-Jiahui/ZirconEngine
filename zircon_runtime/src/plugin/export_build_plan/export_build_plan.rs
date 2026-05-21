use serde::{Deserialize, Serialize};

use super::ExportGeneratedFile;
use crate::{
    plugin::ExportPlatformPolicy, plugin::ExportProfile, plugin::ProjectPluginSelection,
    plugin::RuntimePluginAvailabilityReport,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportBuildPlan {
    pub profile: ExportProfile,
    #[serde(default)]
    pub platform_policy: ExportPlatformPolicy,
    pub enabled_runtime_plugins: Vec<String>,
    pub linked_runtime_crates: Vec<String>,
    pub native_dynamic_packages: Vec<String>,
    #[serde(default)]
    pub runtime_plugin_availability: RuntimePluginAvailabilityReport,
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
    pub provider_package_id: Option<String>,
}

impl ExportLinkedRuntimeCrate {
    pub fn runtime_plugin(crate_name: String, path: String) -> Self {
        Self {
            crate_name,
            path,
            registration_kind: ExportRuntimeCrateRegistrationKind::RuntimePlugin,
            provider_package_id: None,
        }
    }

    pub fn runtime_feature_with_provider(
        crate_name: String,
        path: String,
        provider_package_id: Option<String>,
    ) -> Self {
        Self {
            crate_name,
            path,
            registration_kind: ExportRuntimeCrateRegistrationKind::RuntimeFeature,
            provider_package_id,
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
        runtime_plugin_availability: RuntimePluginAvailabilityReport,
        generated_files: Vec<ExportGeneratedFile>,
    ) -> Self {
        let platform_policy = profile.target_platform.policy();
        Self {
            enabled_runtime_plugins: enabled_plugins
                .iter()
                .map(|selection| selection.id.clone())
                .collect(),
            profile,
            platform_policy,
            linked_runtime_crates,
            native_dynamic_packages,
            runtime_plugin_availability,
            generated_files,
            diagnostics: Vec::new(),
            fatal_diagnostics: Vec::new(),
        }
    }

    pub fn effective_fatal_diagnostics(&self) -> Vec<String> {
        let mut diagnostics = self.fatal_diagnostics.clone();
        for entry in &self.runtime_plugin_availability.missing_required {
            let diagnostic = format!(
                "required runtime plugin {} is unavailable for export profile {}: {}",
                entry.id, self.profile.name, entry.reason
            );
            if !diagnostics.iter().any(|existing| existing == &diagnostic) {
                diagnostics.push(diagnostic);
            }
        }
        diagnostics
    }

    pub fn has_fatal_diagnostics(&self) -> bool {
        !self.effective_fatal_diagnostics().is_empty()
    }
}
