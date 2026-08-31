use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{
    ExportGeneratedFile, LibraryEmbedCompileHostPlan, NativeDynamicPackageExportPlan,
    SourceTemplateBuildValidationPlan,
};
use crate::{
    core::framework::project::ExportPlatformPolicy, core::framework::project::ExportProfile,
    core::framework::project::ProjectPluginSelection, plugin::RuntimePluginAvailabilityReport,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportBuildPlan {
    pub profile: ExportProfile,
    #[serde(default)]
    pub platform_policy: ExportPlatformPolicy,
    pub enabled_runtime_plugins: Vec<String>,
    pub linked_runtime_crates: Vec<String>,
    pub native_dynamic_packages: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub native_dynamic_package_exports: Vec<NativeDynamicPackageExportPlan>,
    #[serde(default)]
    pub runtime_plugin_availability: RuntimePluginAvailabilityReport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_embed_compile_host: Option<LibraryEmbedCompileHostPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_template_build: Option<SourceTemplateBuildValidationPlan>,
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
        native_dynamic_package_exports: Vec<NativeDynamicPackageExportPlan>,
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
            native_dynamic_package_exports,
            runtime_plugin_availability,
            library_embed_compile_host: None,
            source_template_build: None,
            generated_files,
            diagnostics: Vec::new(),
            fatal_diagnostics: Vec::new(),
        }
    }

    pub fn effective_fatal_diagnostics(&self) -> Vec<String> {
        merge_unique_diagnostics(
            self.fatal_diagnostics.clone(),
            self.runtime_plugin_availability
                .missing_required
                .iter()
                .map(|entry| {
                    format!(
                        "required runtime plugin {} is unavailable for export profile {}: {}",
                        entry.id, self.profile.name, entry.reason
                    )
                }),
        )
    }

    pub fn has_fatal_diagnostics(&self) -> bool {
        !self.fatal_diagnostics.is_empty()
            || !self.runtime_plugin_availability.missing_required.is_empty()
    }
}

fn merge_unique_diagnostics(
    mut diagnostics: Vec<String>,
    additions: impl IntoIterator<Item = String>,
) -> Vec<String> {
    let mut existing = HashSet::<&str>::with_capacity(diagnostics.len());
    existing.extend(diagnostics.iter().map(String::as_str));
    let additions = additions.into_iter();
    let (minimum_additions, maximum_additions) = additions.size_hint();
    let addition_capacity = maximum_additions.unwrap_or(minimum_additions);
    let mut accepted_keys = HashSet::with_capacity(addition_capacity);
    let mut accepted = Vec::with_capacity(addition_capacity);
    for diagnostic in additions {
        if !existing.contains(diagnostic.as_str()) && accepted_keys.insert(diagnostic.clone()) {
            accepted.push(diagnostic);
        }
    }
    drop(existing);
    diagnostics.extend(accepted);
    diagnostics
}

#[cfg(test)]
mod tests {
    #[test]
    fn fatal_presence_check_does_not_materialize_diagnostics() {
        let source = include_str!("export_build_plan.rs");
        let allocating_check = ["!self.effective_fatal_diagnostics()", ".is_empty()"].concat();

        assert!(!source.contains(&allocating_check));
    }
}

#[cfg(test)]
mod effective_fatal_diagnostics_tests;
