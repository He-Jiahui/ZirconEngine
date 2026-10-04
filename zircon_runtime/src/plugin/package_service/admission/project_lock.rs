mod input;

use super::super::PackageStore;
use super::policy_index::{
    load_host_policy_for_target, load_host_policy_index_for_target, LoadedNativePluginPolicy,
    NativePluginInstalledSelection, NativePluginPolicyError,
};
use super::project_resolution::{
    resolve_project_native_plugin_admission, NativePluginAdmission, NativePluginSelectionStatus,
};
use crate::{
    core::framework::{
        platform::RuntimeTargetMode,
        project::{ExportPackagingStrategy, ProjectPluginManifest},
    },
    plugin::native::NativePluginArtifactTarget,
};
use sha2::{Digest, Sha256};
use std::path::Path;
use zircon_runtime_interface::project::{
    ProjectPackageLock, ProjectPackageLockAuthority, ProjectPackageLockEntry,
    ProjectPackageLockPlatform, ProjectPackageLockProject, ProjectPackageLockRuntimeMode,
    ProjectPackageLockTarget, PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectPackageLockProviderError {
    Policy,
    BuildSetMismatch,
    ProjectOverlap,
    SelectionMissing,
    InstallationMissing,
    InstallationChanged,
    UnsupportedTarget,
    InvalidProjectIdentity,
    InvalidLock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HostGeneration {
    selections: Vec<NativePluginInstalledSelection>,
    inventory_revisions: Vec<(String, String)>,
    authority: ProjectPackageLockAuthority,
}

/// Produces one target-keyed, read-only lock from the authenticated native admission path.
///
/// The helper has already loaded the initial policy from its pinned policy-index argument. This
/// function rechecks the process-selected policy before and after the capture, then compares the
/// installed sidecar, package inventories, and canonical rows. It never accepts package identity
/// or artifact fields from ProjectPluginManifest.
pub fn capture_project_package_lock(
    initial_policy: LoadedNativePluginPolicy,
    project: ProjectPackageLockProject,
    project_root: &Path,
    target: NativePluginArtifactTarget,
) -> Result<ProjectPackageLock, ProjectPackageLockProviderError> {
    let root = project_root
        .canonicalize()
        .map_err(|_| ProjectPackageLockProviderError::InvalidProjectIdentity)?;
    if !root.is_dir() || !initial_policy.is_disjoint_from_project(&root) {
        return Err(ProjectPackageLockProviderError::ProjectOverlap);
    }
    let input = input::read_project_manifest(&root, &project)?;
    let build_set_id = initial_policy.policy().build_set_id.clone();
    let lock = capture_verified_project_package_lock(
        initial_policy,
        project,
        &root,
        Some(&build_set_id),
        target,
        &input.manifest.plugins,
    )?;
    // Keep the source handle alive through capture and reject path/generation drift.
    input::read_project_manifest(&root, &lock.project)?;
    drop(input);
    Ok(lock)
}

fn capture_verified_project_package_lock(
    initial_policy: LoadedNativePluginPolicy,
    project: ProjectPackageLockProject,
    project_root: &Path,
    authenticated_build_set_id: Option<&str>,
    target: NativePluginArtifactTarget,
    approved_selections: &ProjectPluginManifest,
) -> Result<ProjectPackageLock, ProjectPackageLockProviderError> {
    let target_contract = contract_target(&target)?;
    let root = project_root
        .canonicalize()
        .map_err(|_| ProjectPackageLockProviderError::ProjectOverlap)?;
    if !root.is_dir() || !initial_policy.is_disjoint_from_project(&root) {
        return Err(ProjectPackageLockProviderError::ProjectOverlap);
    }
    if initial_policy.policy().target != target {
        return Err(ProjectPackageLockProviderError::UnsupportedTarget);
    }
    if let Some(build_set_id) = authenticated_build_set_id {
        if !is_digest(build_set_id) || build_set_id != initial_policy.policy().build_set_id {
            return Err(ProjectPackageLockProviderError::BuildSetMismatch);
        }
    }

    // Empty and populated captures share the same generation and policy fences.
    // Absence of active native selections is a manifest result, not a policy bypass.
    let build_set_id = match authenticated_build_set_id {
        Some(build_set_id) => build_set_id,
        None if !active_native_selection(approved_selections, &target) => {
            initial_policy.policy().build_set_id.as_str()
        }
        None => return Err(ProjectPackageLockProviderError::BuildSetMismatch),
    };

    ensure_current_policy(&initial_policy, &target, &root)?;
    let first_admission = resolve_project_native_plugin_admission(
        &root,
        Some(build_set_id),
        target.clone(),
        approved_selections,
    );
    if first_admission.has_required_failures() {
        return Err(ProjectPackageLockProviderError::SelectionMissing);
    }
    let first_generation = capture_generation(&initial_policy, &target, approved_selections)?;
    let first_entries = collect_entries(
        &initial_policy,
        &first_admission,
        &target,
        approved_selections,
    )?;

    let second_policy = load_host_policy_index_for_target(initial_policy.index_path(), &target)
        .map_err(|_| ProjectPackageLockProviderError::InstallationChanged)?;
    if second_policy.index_sha256() != initial_policy.index_sha256()
        || second_policy.policy_sha256() != initial_policy.policy_sha256()
        || second_policy.policy().root != initial_policy.policy().root
    {
        return Err(ProjectPackageLockProviderError::InstallationChanged);
    }
    ensure_current_policy(&second_policy, &target, &root)?;
    let second_admission = resolve_project_native_plugin_admission(
        &root,
        Some(build_set_id),
        target.clone(),
        approved_selections,
    );
    if second_admission.has_required_failures() {
        return Err(ProjectPackageLockProviderError::InstallationChanged);
    }
    let second_generation = capture_generation(&second_policy, &target, approved_selections)?;
    let second_entries = collect_entries(
        &second_policy,
        &second_admission,
        &target,
        approved_selections,
    )?;
    if first_generation != second_generation || first_entries != second_entries {
        return Err(ProjectPackageLockProviderError::InstallationChanged);
    }

    let lock = ProjectPackageLock {
        schema_version: PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
        project,
        target: target_contract,
        authority: first_generation.authority,
        entries: first_entries,
    };
    require_bounded_lock(&lock)?;
    Ok(lock)
}

fn require_bounded_lock(lock: &ProjectPackageLock) -> Result<(), ProjectPackageLockProviderError> {
    lock.canonical_bytes()
        .map(|_| ())
        .map_err(|_| ProjectPackageLockProviderError::InvalidLock)
}

fn ensure_current_policy(
    pinned: &LoadedNativePluginPolicy,
    target: &NativePluginArtifactTarget,
    project_root: &Path,
) -> Result<(), ProjectPackageLockProviderError> {
    let current =
        load_host_policy_for_target(target).map_err(|_| ProjectPackageLockProviderError::Policy)?;
    if current.index_sha256() != pinned.index_sha256()
        || current.policy_sha256() != pinned.policy_sha256()
        || current.policy().root != pinned.policy().root
        || !current.is_disjoint_from_project(project_root)
    {
        return Err(ProjectPackageLockProviderError::InstallationChanged);
    }
    Ok(())
}

fn active_native_selection(
    manifest: &ProjectPluginManifest,
    target: &NativePluginArtifactTarget,
) -> bool {
    manifest.selections.iter().any(|selection| {
        selection.enabled
            && selection.packaging == ExportPackagingStrategy::NativeDynamic
            && selection.supports_target(target.runtime_mode)
    })
}

fn capture_generation(
    policy: &LoadedNativePluginPolicy,
    target: &NativePluginArtifactTarget,
    approved_selections: &ProjectPluginManifest,
) -> Result<HostGeneration, ProjectPackageLockProviderError> {
    let mut selections = policy
        .installed_selections()
        .map_err(|_| ProjectPackageLockProviderError::Policy)?;
    selections.sort_by(|left, right| {
        left.plugin_id
            .cmp(&right.plugin_id)
            .then(left.identity_digest.cmp(&right.identity_digest))
            .then(left.package_id.cmp(&right.package_id))
            .then(left.release_revision.cmp(&right.release_revision))
            .then(left.artifact_digest.cmp(&right.artifact_digest))
    });
    let active_ids = approved_selections
        .selections
        .iter()
        .filter(|selection| {
            selection.enabled
                && selection.packaging == ExportPackagingStrategy::NativeDynamic
                && selection.supports_target(target.runtime_mode)
        })
        .map(|selection| selection.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    selections.retain(|selected| {
        selected.target == *target && active_ids.contains(selected.plugin_id.as_str())
    });
    let mut inventory_revisions = Vec::with_capacity(selections.len());
    for selected in &selections {
        let store = PackageStore::open_existing(&policy.policy().root, &selected.identity_digest)
            .map_err(|_| ProjectPackageLockProviderError::InstallationMissing)?
            .ok_or(ProjectPackageLockProviderError::InstallationMissing)?;
        let inventory = store
            .inventory()
            .map_err(|_| ProjectPackageLockProviderError::InstallationMissing)?;
        inventory_revisions.push((selected.identity_digest.clone(), inventory.revision));
    }
    inventory_revisions.sort();
    let provider_revision_digest = format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(&inventory_revisions)
                .map_err(|_| ProjectPackageLockProviderError::Policy)?,
        )
    );
    let mut capabilities = policy.policy().allowed_capabilities.clone();
    capabilities.sort();
    capabilities.dedup();
    let capability_digest = format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(&capabilities)
                .map_err(|_| ProjectPackageLockProviderError::Policy)?,
        )
    );
    Ok(HostGeneration {
        selections,
        inventory_revisions,
        authority: ProjectPackageLockAuthority {
            index_sha256: policy.index_sha256().to_owned(),
            policy_sha256: policy.policy_sha256().to_owned(),
            build_set_id: policy.policy().build_set_id.clone(),
            capability_digest,
            provider_revision_digest,
        },
    })
}

fn collect_entries(
    policy: &LoadedNativePluginPolicy,
    admission: &NativePluginAdmission,
    target: &NativePluginArtifactTarget,
    approved_selections: &ProjectPluginManifest,
) -> Result<Vec<ProjectPackageLockEntry>, ProjectPackageLockProviderError> {
    let installed = policy
        .installed_selections()
        .map_err(|_| ProjectPackageLockProviderError::Policy)?;
    let mut entries = Vec::new();
    for selection in approved_selections.selections.iter().filter(|selection| {
        selection.enabled
            && selection.packaging == ExportPackagingStrategy::NativeDynamic
            && selection.supports_target(target.runtime_mode)
    }) {
        let outcome = admission
            .outcomes()
            .iter()
            .find(|outcome| outcome.plugin_id == selection.id);
        if outcome.is_none_or(|outcome| outcome.status != NativePluginSelectionStatus::Admitted) {
            if selection.required {
                return Err(ProjectPackageLockProviderError::SelectionMissing);
            }
            continue;
        }
        let selected_install = installed
            .iter()
            .find(|item| item.target == *target && item.plugin_id == selection.id)
            .ok_or(ProjectPackageLockProviderError::SelectionMissing)?;
        let store =
            PackageStore::open_existing(&policy.policy().root, &selected_install.identity_digest)
                .map_err(|_| ProjectPackageLockProviderError::InstallationMissing)?
                .ok_or(ProjectPackageLockProviderError::InstallationMissing)?;
        let inventory = store
            .inventory()
            .map_err(|_| ProjectPackageLockProviderError::InstallationMissing)?;
        let package = inventory
            .packages
            .iter()
            .find(|package| {
                package.package_id == selected_install.package_id
                    && package.release_revision == selected_install.release_revision
                    && package.artifact_digest == selected_install.artifact_digest
            })
            .ok_or(ProjectPackageLockProviderError::InstallationChanged)?;
        let expectation = admission
            .authority()
            .expectation(&selection.id)
            .ok_or(ProjectPackageLockProviderError::InstallationChanged)?;
        if expectation.target != *target || expectation.package_id != package.package_id {
            return Err(ProjectPackageLockProviderError::InstallationChanged);
        }
        let mut capabilities = expectation.capabilities.clone();
        capabilities.sort();
        capabilities.dedup();
        entries.push(ProjectPackageLockEntry {
            plugin_id: selection.id.clone(),
            package_id: package.package_id.clone(),
            version: package.version.clone(),
            release_revision: package.release_revision.clone(),
            artifact_digest: package.artifact_digest.clone(),
            capabilities,
        });
    }
    entries.sort_by(|left, right| {
        left.plugin_id
            .cmp(&right.plugin_id)
            .then(left.package_id.cmp(&right.package_id))
            .then(left.release_revision.cmp(&right.release_revision))
            .then(left.artifact_digest.cmp(&right.artifact_digest))
    });
    Ok(entries)
}

fn contract_target(
    target: &NativePluginArtifactTarget,
) -> Result<ProjectPackageLockTarget, ProjectPackageLockProviderError> {
    let runtime_mode = match target.runtime_mode {
        RuntimeTargetMode::ClientRuntime => ProjectPackageLockRuntimeMode::ClientRuntime,
        RuntimeTargetMode::EditorHost => ProjectPackageLockRuntimeMode::EditorHost,
        RuntimeTargetMode::ServerRuntime => {
            return Err(ProjectPackageLockProviderError::UnsupportedTarget);
        }
    };
    let platform = match target.platform {
        crate::core::framework::project::ExportTargetPlatform::Windows => {
            ProjectPackageLockPlatform::Windows
        }
        crate::core::framework::project::ExportTargetPlatform::Linux => {
            ProjectPackageLockPlatform::Linux
        }
        crate::core::framework::project::ExportTargetPlatform::Macos => {
            ProjectPackageLockPlatform::Macos
        }
        _ => return Err(ProjectPackageLockProviderError::UnsupportedTarget),
    };
    Ok(ProjectPackageLockTarget {
        runtime_mode,
        platform,
    })
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
#[path = "tests/project_lock.rs"]
mod tests;
