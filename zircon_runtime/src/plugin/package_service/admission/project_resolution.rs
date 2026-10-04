use super::super::{
    distinct_members, InstalledPackage, PackageError, PackageHostPolicy, PackageInventory,
    PackageStore, Result, MAX_CONTROL_BYTES,
};
use super::policy_index::{
    load_host_policy_for_target, NativePluginInstalledSelection, NativePluginPolicyError,
};
use crate::{
    core::framework::{
        platform::RuntimeTargetMode,
        project::{ExportPackagingStrategy, ProjectPluginManifest},
    },
    plugin::{
        native::{
            verify_native_package_receipts, NativePackageModuleArtifact,
            NativePackageReceiptPolicy, NativePackageReceiptTrust, NativePluginArtifactAuthority,
            NativePluginArtifactTarget, VerifiedNativePackageProof,
        },
        PluginPackageManifest,
    },
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

const MAX_ADMISSION_IDENTITIES: usize = 128;
const MAX_ADMISSION_PACKAGES: usize = 512;
const MAX_RECEIPTS_PER_PACKAGE: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativePluginPolicyStatus {
    NotNeeded,
    Configured,
    Unconfigured,
    TargetUnconfigured,
    Rejected,
    StoreUnavailable,
    HostBuildSetUnavailable,
    BuildSetMismatch,
    ProjectOverlap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativePluginSelectionStatus {
    Admitted,
    MissingSelection,
    MissingInstallation,
    IneligibleProductRole,
    AmbiguousInstallation,
    DuplicateSelection,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginSelectionOutcome {
    pub plugin_id: String,
    pub required: bool,
    pub status: NativePluginSelectionStatus,
    pub detail: &'static str,
}

#[derive(Clone, Debug)]
pub struct NativePluginAdmission {
    target: NativePluginArtifactTarget,
    policy_status: NativePluginPolicyStatus,
    installed_root: Option<PathBuf>,
    authority: NativePluginArtifactAuthority,
    outcomes: Vec<NativePluginSelectionOutcome>,
}

impl NativePluginAdmission {
    pub fn target(&self) -> &NativePluginArtifactTarget {
        &self.target
    }

    pub fn policy_status(&self) -> NativePluginPolicyStatus {
        self.policy_status
    }

    pub fn installed_root(&self) -> Option<&Path> {
        self.installed_root.as_deref()
    }

    pub fn authority(&self) -> &NativePluginArtifactAuthority {
        &self.authority
    }

    pub fn outcomes(&self) -> &[NativePluginSelectionOutcome] {
        &self.outcomes
    }

    pub fn has_required_failures(&self) -> bool {
        self.outcomes.iter().any(|outcome| {
            outcome.required && outcome.status != NativePluginSelectionStatus::Admitted
        })
    }

    /// Projects every selection result into a stable diagnostic for product consumers.
    pub fn diagnostics(&self) -> Vec<String> {
        let mut diagnostics = Vec::with_capacity(self.outcomes.len() + 1);
        if !matches!(
            self.policy_status,
            NativePluginPolicyStatus::NotNeeded | NativePluginPolicyStatus::Configured
        ) {
            diagnostics.push(format!(
                "native_plugin_policy target={:?} status={:?}",
                self.target.runtime_mode, self.policy_status
            ));
        }
        diagnostics.extend(self.outcomes.iter().map(|outcome| {
            format!(
                "native_plugin_selection target={:?} plugin_id={} required={} status={:?} detail={}",
                self.target.runtime_mode,
                outcome.plugin_id,
                outcome.required,
                outcome.status,
                outcome.detail
            )
        }));
        diagnostics
    }

    /// Returns the required selection failures that must stop Editor Ready or Play activation.
    pub fn required_failure_diagnostic(&self) -> Option<String> {
        let failures = self
            .outcomes
            .iter()
            .filter(|outcome| {
                outcome.required && outcome.status != NativePluginSelectionStatus::Admitted
            })
            .map(|outcome| {
                format!(
                    "{}:{:?}:{}",
                    outcome.plugin_id, outcome.status, outcome.detail
                )
            })
            .collect::<Vec<_>>();
        if failures.is_empty() {
            return None;
        }
        Some(format!(
            "required native plugin admission failed for {:?}: {}; admission results: {}",
            self.target.runtime_mode,
            failures.join(", "),
            self.diagnostics().join("; ")
        ))
    }

    fn policy_failure(
        target: NativePluginArtifactTarget,
        status: NativePluginPolicyStatus,
        selections: impl IntoIterator<Item = (String, bool)>,
        detail: &'static str,
    ) -> Self {
        Self {
            target,
            policy_status: status,
            installed_root: None,
            authority: NativePluginArtifactAuthority::deny_all(),
            outcomes: selections
                .into_iter()
                .map(|(plugin_id, required)| NativePluginSelectionOutcome {
                    plugin_id,
                    required,
                    status: NativePluginSelectionStatus::Rejected,
                    detail,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InstalledBundlePlan {
    schema_version: u32,
    manifest_path: String,
    manifest_logical_name: String,
    receipts: Vec<String>,
    modules: Vec<NativePackageModuleArtifact>,
}

struct StoredPluginPackage {
    store_index: usize,
    installed: InstalledPackage,
    manifest: PluginPackageManifest,
    manifest_bytes: Vec<u8>,
    plan: InstalledBundlePlan,
}

/// Re-verifies selected, privately installed packages for the active host context.
///
/// Project manifests contribute only the independently selected plugin IDs and required flags.
/// The policy source, signer trust, target, BuildSet, capabilities, and installed root are all
/// supplied by the private host policy index and the App-authenticated BuildSet.
pub fn resolve_project_native_plugin_admission(
    project_root: &Path,
    authenticated_build_set_id: Option<&str>,
    target: NativePluginArtifactTarget,
    selections: &ProjectPluginManifest,
) -> NativePluginAdmission {
    let selected = active_native_selections(selections, &target);
    if selected.is_empty() {
        return NativePluginAdmission {
            target,
            policy_status: NativePluginPolicyStatus::NotNeeded,
            installed_root: None,
            authority: NativePluginArtifactAuthority::deny_all(),
            outcomes: Vec::new(),
        };
    }
    let selection_keys = selected
        .iter()
        .map(|selection| (selection.id.clone(), selection.required))
        .collect::<Vec<_>>();
    let Some(build_set_id) = authenticated_build_set_id else {
        return NativePluginAdmission::policy_failure(
            target,
            NativePluginPolicyStatus::HostBuildSetUnavailable,
            selection_keys,
            "app_authenticated_build_set_unavailable",
        );
    };
    if !super::super::is_digest(build_set_id) {
        return NativePluginAdmission::policy_failure(
            target,
            NativePluginPolicyStatus::HostBuildSetUnavailable,
            selection_keys,
            "app_authenticated_build_set_invalid",
        );
    }

    let loaded = match load_host_policy_for_target(&target) {
        Ok(loaded) => loaded,
        Err(error) => {
            let (status, detail) = match error {
                NativePluginPolicyError::Unconfigured => (
                    NativePluginPolicyStatus::Unconfigured,
                    "native_plugin_policy_unconfigured",
                ),
                NativePluginPolicyError::TargetUnconfigured => (
                    NativePluginPolicyStatus::TargetUnconfigured,
                    "native_plugin_policy_target_unconfigured",
                ),
                NativePluginPolicyError::StoreUnavailable => (
                    NativePluginPolicyStatus::StoreUnavailable,
                    "native_plugin_policy_store_unavailable",
                ),
                NativePluginPolicyError::Rejected => (
                    NativePluginPolicyStatus::Rejected,
                    "native_plugin_policy_rejected",
                ),
            };
            return NativePluginAdmission::policy_failure(target, status, selection_keys, detail);
        }
    };
    if loaded.policy().build_set_id != build_set_id {
        return NativePluginAdmission::policy_failure(
            target,
            NativePluginPolicyStatus::BuildSetMismatch,
            selection_keys,
            "native_plugin_policy_build_set_mismatch",
        );
    }
    if !loaded.is_disjoint_from_project(project_root) {
        return NativePluginAdmission::policy_failure(
            target,
            NativePluginPolicyStatus::ProjectOverlap,
            selection_keys,
            "native_plugin_policy_overlaps_project",
        );
    }

    if selected.len() > MAX_ADMISSION_PACKAGES {
        return NativePluginAdmission::policy_failure(
            target,
            NativePluginPolicyStatus::StoreUnavailable,
            selection_keys,
            "native_plugin_selection_capacity_exceeded",
        );
    }
    let installed_selections = match loaded.installed_selections() {
        Ok(selections) => selections,
        Err(_) => {
            return NativePluginAdmission::policy_failure(
                target,
                NativePluginPolicyStatus::Rejected,
                selection_keys,
                "native_plugin_installation_selection_rejected",
            );
        }
    };
    let mut stores = Vec::new();
    let mut store_indices = HashMap::<String, usize>::new();

    let selection_counts = selected
        .iter()
        .fold(HashMap::new(), |mut counts, selection| {
            *counts.entry(selection.id.clone()).or_insert(0usize) += 1;
            counts
        });
    let mut proofs = Vec::<VerifiedNativePackageProof>::new();
    let mut outcomes = Vec::with_capacity(selected.len());
    for selection in selected {
        if selection_counts
            .get(selection.id.as_str())
            .copied()
            .unwrap_or(0)
            > 1
        {
            outcomes.push(NativePluginSelectionOutcome {
                plugin_id: selection.id,
                required: selection.required,
                status: NativePluginSelectionStatus::DuplicateSelection,
                detail: "project_native_plugin_selection_duplicate",
            });
            continue;
        }
        let Some(installed_selection) = installed_selections
            .iter()
            .find(|entry| entry.plugin_id == selection.id)
        else {
            outcomes.push(NativePluginSelectionOutcome {
                plugin_id: selection.id,
                required: selection.required,
                status: NativePluginSelectionStatus::MissingSelection,
                detail: "native_plugin_installed_selection_missing",
            });
            continue;
        };
        let store_index = if let Some(store_index) = store_indices
            .get(&installed_selection.identity_digest)
            .copied()
        {
            store_index
        } else {
            if stores.len() >= MAX_ADMISSION_IDENTITIES {
                outcomes.push(NativePluginSelectionOutcome {
                    plugin_id: selection.id,
                    required: selection.required,
                    status: NativePluginSelectionStatus::Rejected,
                    detail: "native_plugin_installed_store_capacity_exceeded",
                });
                continue;
            }
            match PackageStore::open_existing(
                &loaded.policy().root,
                &installed_selection.identity_digest,
            ) {
                Ok(Some(store)) => {
                    let store_index = stores.len();
                    stores.push(store);
                    store_indices.insert(installed_selection.identity_digest.clone(), store_index);
                    store_index
                }
                Ok(None) => {
                    outcomes.push(NativePluginSelectionOutcome {
                        plugin_id: selection.id,
                        required: selection.required,
                        status: NativePluginSelectionStatus::MissingInstallation,
                        detail: "native_plugin_installation_missing",
                    });
                    continue;
                }
                Err(_) => {
                    outcomes.push(NativePluginSelectionOutcome {
                        plugin_id: selection.id,
                        required: selection.required,
                        status: NativePluginSelectionStatus::Rejected,
                        detail: "native_plugin_installed_store_rejected",
                    });
                    continue;
                }
            }
        };
        let store = &stores[store_index];
        let inventory = match store.inventory() {
            Ok(inventory) => inventory,
            Err(_) => {
                outcomes.push(NativePluginSelectionOutcome {
                    plugin_id: selection.id,
                    required: selection.required,
                    status: NativePluginSelectionStatus::Rejected,
                    detail: "native_plugin_installed_inventory_rejected",
                });
                continue;
            }
        };
        let installed = match selected_install_from_inventory(installed_selection, &inventory) {
            SelectedInstallLookup::Matched(package) => package.clone(),
            SelectedInstallLookup::Missing => {
                outcomes.push(NativePluginSelectionOutcome {
                    plugin_id: selection.id,
                    required: selection.required,
                    status: NativePluginSelectionStatus::MissingInstallation,
                    detail: "native_plugin_selected_installation_missing",
                });
                continue;
            }
            SelectedInstallLookup::Stale => {
                outcomes.push(NativePluginSelectionOutcome {
                    plugin_id: selection.id,
                    required: selection.required,
                    status: NativePluginSelectionStatus::Rejected,
                    detail: "native_plugin_installation_selection_stale",
                });
                continue;
            }
        };
        let package = match load_stored_plugin_package(store, store_index, installed) {
            Ok(package) => package,
            Err(_) => {
                outcomes.push(NativePluginSelectionOutcome {
                    plugin_id: selection.id,
                    required: selection.required,
                    status: NativePluginSelectionStatus::Rejected,
                    detail: "native_plugin_installed_package_rejected",
                });
                continue;
            }
        };
        if package.manifest.id != selection.id {
            outcomes.push(NativePluginSelectionOutcome {
                plugin_id: selection.id,
                required: selection.required,
                status: NativePluginSelectionStatus::Rejected,
                detail: "native_plugin_installation_selection_identity_mismatch",
            });
            continue;
        }
        if !package.manifest.package_role.is_product_catalog_eligible() {
            outcomes.push(NativePluginSelectionOutcome {
                plugin_id: selection.id,
                required: selection.required,
                status: NativePluginSelectionStatus::IneligibleProductRole,
                detail: "native_plugin_test_or_sample_package_ineligible",
            });
            continue;
        }
        match verify_stored_plugin_package(&stores[package.store_index], &package, loaded.policy())
        {
            Ok(proof) => {
                proofs.push(proof);
                outcomes.push(NativePluginSelectionOutcome {
                    plugin_id: selection.id,
                    required: selection.required,
                    status: NativePluginSelectionStatus::Admitted,
                    detail: "native_plugin_installation_verified",
                });
            }
            Err(_) => outcomes.push(NativePluginSelectionOutcome {
                plugin_id: selection.id,
                required: selection.required,
                status: NativePluginSelectionStatus::Rejected,
                detail: "native_plugin_installation_trust_rejected",
            }),
        }
    }

    let authority = match NativePluginArtifactAuthority::from_verified_packages(proofs) {
        Ok(authority) => authority,
        Err(_) => {
            for outcome in &mut outcomes {
                if outcome.status == NativePluginSelectionStatus::Admitted {
                    outcome.status = NativePluginSelectionStatus::Rejected;
                    outcome.detail = "native_plugin_verified_proof_aggregation_rejected";
                }
            }
            NativePluginArtifactAuthority::deny_all()
        }
    };
    NativePluginAdmission {
        target,
        policy_status: NativePluginPolicyStatus::Configured,
        installed_root: Some(loaded.policy().root.clone()),
        authority,
        outcomes,
    }
}

struct ActiveNativeSelection {
    id: String,
    required: bool,
}

fn active_native_selections(
    manifest: &ProjectPluginManifest,
    target: &NativePluginArtifactTarget,
) -> Vec<ActiveNativeSelection> {
    manifest
        .selections
        .iter()
        .filter(|selection| {
            selection.enabled
                && selection.packaging == ExportPackagingStrategy::NativeDynamic
                && selection.supports_target(target.runtime_mode)
        })
        .map(|selection| ActiveNativeSelection {
            id: selection.id.clone(),
            required: selection.required,
        })
        .collect()
}

enum SelectedInstallLookup<'a> {
    Matched(&'a InstalledPackage),
    Missing,
    Stale,
}

fn selected_install_from_inventory<'a>(
    selection: &NativePluginInstalledSelection,
    inventory: &'a PackageInventory,
) -> SelectedInstallLookup<'a> {
    let Some(package) = inventory
        .packages
        .iter()
        .find(|package| package.package_id == selection.package_id)
    else {
        return SelectedInstallLookup::Missing;
    };
    if package.release_revision != selection.release_revision
        || package.artifact_digest != selection.artifact_digest
    {
        return SelectedInstallLookup::Stale;
    }
    SelectedInstallLookup::Matched(package)
}

fn load_stored_plugin_package(
    store: &PackageStore,
    store_index: usize,
    installed: InstalledPackage,
) -> Result<StoredPluginPackage> {
    let plan_bytes = store.installed_file(&installed, "package-install.json")?;
    if plan_bytes.len() > MAX_CONTROL_BYTES {
        return Err(PackageError::Capacity);
    }
    let plan: InstalledBundlePlan =
        serde_json::from_slice(&plan_bytes).map_err(|_| PackageError::Storage)?;
    if plan.schema_version != 1
        || plan.receipts.is_empty()
        || plan.receipts.len() > MAX_RECEIPTS_PER_PACKAGE
        || plan.modules.is_empty()
        || plan.modules.len() > 3
        || plan.manifest_logical_name.trim().is_empty()
    {
        return Err(PackageError::Storage);
    }
    let manifest_bytes = store.installed_file(&installed, &plan.manifest_path)?;
    if manifest_bytes.len() > MAX_CONTROL_BYTES {
        return Err(PackageError::Capacity);
    }
    let manifest_text = std::str::from_utf8(&manifest_bytes).map_err(|_| PackageError::Storage)?;
    let manifest: PluginPackageManifest =
        toml::from_str(manifest_text).map_err(|_| PackageError::Storage)?;
    let mut expected_members = vec!["package-install.json", plan.manifest_path.as_str()];
    expected_members.extend(plan.receipts.iter().map(String::as_str));
    for module in &plan.modules {
        expected_members.push(module.relative_path.as_str());
        expected_members.extend(
            module
                .dependencies
                .iter()
                .map(|dependency| dependency.relative_path.as_str()),
        );
    }
    if !distinct_members(expected_members.iter().copied())
        || expected_members.len() != installed.files.len()
        || expected_members
            .iter()
            .any(|member| !installed.files.contains_key(*member))
        || manifest.package_id() != installed.package_id
        || manifest.version != installed.version
        || manifest.modules.len() != plan.modules.len()
        || !manifest.dependencies.is_empty()
        || !manifest.optional_features.is_empty()
        || !manifest.feature_extensions.is_empty()
        || manifest
            .modules
            .iter()
            .any(|module| !module.module_dependencies.is_empty())
    {
        return Err(PackageError::Trust);
    }
    Ok(StoredPluginPackage {
        store_index,
        installed,
        manifest,
        manifest_bytes,
        plan,
    })
}

fn verify_stored_plugin_package(
    store: &PackageStore,
    package: &StoredPluginPackage,
    host: &PackageHostPolicy,
) -> Result<VerifiedNativePackageProof> {
    if !package.manifest.package_role.is_product_catalog_eligible() {
        return Err(PackageError::Trust);
    }
    let receipts = package
        .plan
        .receipts
        .iter()
        .map(|member| store.installed_file(&package.installed, member))
        .collect::<Result<Vec<_>>>()?;
    let receipt_refs = receipts.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let registry = serde_json::to_vec(&host.trust_registry).map_err(|_| PackageError::Trust)?;
    let trust = NativePackageReceiptTrust::from_registry_json(
        &registry,
        host.key_policies.clone(),
        host.trust_valid_until,
        MAX_CONTROL_BYTES,
    )
    .map_err(|_| PackageError::Trust)?;
    let policy = NativePackageReceiptPolicy {
        plugin_id: package.manifest.id.clone(),
        package_id: package.installed.package_id.clone(),
        package_version: package.installed.version.clone(),
        sdk_api_version: host.sdk_api_version.clone(),
        build_set_id: host.build_set_id.clone(),
        target: host.target.clone(),
        target_triple: host.target_triple.clone(),
        manifest_logical_name: package.plan.manifest_logical_name.clone(),
        manifest_relative_path: package.plan.manifest_path.clone(),
        modules: package.plan.modules.clone(),
        required_capabilities: Vec::new(),
        allowed_capabilities: host.allowed_capabilities.clone(),
        now: chrono::Utc::now(),
        max_receipt_age_seconds: host.max_receipt_age_seconds,
        max_receipt_bytes: MAX_CONTROL_BYTES,
        max_receipt_count: MAX_RECEIPTS_PER_PACKAGE,
        max_manifest_bytes: MAX_CONTROL_BYTES,
    };
    verify_native_package_receipts(&receipt_refs, &package.manifest_bytes, &trust, &policy)
        .map_err(|_| PackageError::Trust)
}

#[cfg(test)]
#[path = "tests/project_resolution.rs"]
mod tests;
