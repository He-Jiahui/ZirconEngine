use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    ExportGeneratedFile, ExportLinkedFeatureSourceReceipt, LibraryEmbedCompileHostPlan,
    NativeDynamicPackageExportPlan, SourceTemplateBuildValidationPlan,
};
use crate::{
    core::framework::project::ExportPlatformPolicy,
    core::framework::project::ExportProfile,
    core::framework::project::ProjectPluginSelection,
    plugin::{PluginPackageRole, RuntimePluginAvailabilityReport},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportBuildPlan {
    pub profile: ExportProfile,
    #[serde(default)]
    pub platform_policy: ExportPlatformPolicy,
    pub enabled_runtime_plugins: Vec<String>,
    pub linked_runtime_crates: Vec<String>,
    /// Number of emitted linked feature providers that require source receipts.
    pub linked_feature_source_count: usize,
    pub linked_feature_sources: Vec<ExportLinkedFeatureSourceReceipt>,
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
    /// In-memory admission of the exact canonical plan payload; deserialized plans must be replanned.
    #[serde(skip)]
    pub admitted_plan_proof: Option<ExportPlanAdmissionProof>,
}

/// Opaque proof minted only by canonical export planning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportPlanAdmissionProof {
    payload_sha256: [u8; 32],
}

impl ExportPlanAdmissionProof {
    fn for_plan(plan: &ExportBuildPlan) -> Option<Self> {
        let payload = serde_json::to_vec(plan).ok()?;
        let digest = Sha256::digest(payload);
        let mut payload_sha256 = [0; 32];
        payload_sha256.copy_from_slice(&digest);
        Some(Self { payload_sha256 })
    }

    fn matches(&self, plan: &ExportBuildPlan) -> bool {
        Self::for_plan(plan).is_some_and(|current| current == *self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExportLinkedRuntimeCrate {
    pub crate_name: String,
    pub path: String,
    pub registration_kind: ExportRuntimeCrateRegistrationKind,
    pub provider_package_id: Option<String>,
    pub feature_id: Option<String>,
    pub owner_plugin_id: Option<String>,
    pub provider_package_role: Option<PluginPackageRole>,
    pub admitted_source_path: Option<std::path::PathBuf>,
}

impl ExportLinkedRuntimeCrate {
    pub fn runtime_plugin(crate_name: String, path: String) -> Self {
        Self {
            crate_name,
            path,
            registration_kind: ExportRuntimeCrateRegistrationKind::RuntimePlugin,
            provider_package_id: None,
            feature_id: None,
            owner_plugin_id: None,
            provider_package_role: None,
            admitted_source_path: None,
        }
    }

    pub fn runtime_feature_with_provider(
        crate_name: String,
        path: String,
        provider_package_id: Option<String>,
        feature_id: String,
        owner_plugin_id: String,
    ) -> Self {
        Self {
            crate_name,
            path,
            registration_kind: ExportRuntimeCrateRegistrationKind::RuntimeFeature,
            provider_package_id,
            feature_id: Some(feature_id),
            owner_plugin_id: Some(owner_plugin_id),
            provider_package_role: None,
            admitted_source_path: None,
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
        linked_feature_source_count: usize,
        linked_feature_sources: Vec<ExportLinkedFeatureSourceReceipt>,
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
            linked_feature_source_count,
            linked_feature_sources,
            native_dynamic_packages,
            native_dynamic_package_exports,
            runtime_plugin_availability,
            library_embed_compile_host: None,
            source_template_build: None,
            generated_files,
            diagnostics: Vec::new(),
            fatal_diagnostics: Vec::new(),
            admitted_plan_proof: None,
        }
    }

    pub(crate) fn seal_admitted_plan(&mut self) {
        self.admitted_plan_proof = Some(
            ExportPlanAdmissionProof::for_plan(self)
                .expect("canonical export plan must serialize for admission"),
        );
    }

    pub(crate) fn has_valid_admitted_plan_proof(&self) -> bool {
        self.admitted_plan_proof
            .as_ref()
            .is_some_and(|proof| proof.matches(self))
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
#[path = "tests/export_build_plan.rs"]
mod tests;

#[cfg(test)]
#[path = "export_build_plan/tests/effective_fatal_diagnostics_tests.rs"]
mod effective_fatal_diagnostics_tests;
