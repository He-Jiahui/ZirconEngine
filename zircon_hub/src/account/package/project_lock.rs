use std::path::Path;

use serde_json::json;
use zircon_runtime_interface::project::{
    ProjectGuid, ProjectManifestDigest, ProjectPackageLock, ProjectPackageLockAuthority,
    ProjectPackageLockPlatform, ProjectPackageLockProject, ProjectPackageLockRuntimeMode,
    ProjectPackageLockTarget,
};

use super::{
    process, validate_target_echo, AccountError, PackageClient, PackageInstallTarget,
    PackageRuntimeMode, PackageTargetPlatform,
};

/// Read-only request through the existing pinned executable and host policy transport.
/// No runtime DLL or project-provided package identity enters Hub's dependency graph.
pub(in crate::account) async fn capture_project_lock(
    project_root: &Path,
    project_guid: ProjectGuid,
    manifest_digest: ProjectManifestDigest,
    target_mode: PackageRuntimeMode,
) -> Result<ProjectPackageLock, AccountError> {
    let target = PackageInstallTarget::for_runtime_mode(target_mode)?;
    let project = ProjectPackageLockProject {
        project_guid,
        manifest_digest,
    };
    let client = PackageClient::load(target).await?;
    let response = process::query(
        &client,
        json!({
            "action": "project-lock",
            "schemaVersion": 1,
            "target": target,
            "projectRoot": project_root,
            "project": &project,
        }),
    )
    .await?;
    validate_target_echo(&response, target)?;
    if response.get("status").and_then(serde_json::Value::as_str) != Some("ready") {
        return Err(AccountError::PackageUnavailable);
    }
    let lock: ProjectPackageLock = serde_json::from_value(
        response
            .get("lock")
            .cloned()
            .ok_or(AccountError::PackageUnavailable)?,
    )
    .map_err(|_| AccountError::PackageUnavailable)?;
    validate_lock_response(lock, project, target)
}

fn validate_lock_response(
    lock: ProjectPackageLock,
    project: ProjectPackageLockProject,
    target: PackageInstallTarget,
) -> Result<ProjectPackageLock, AccountError> {
    let expected_target = ProjectPackageLockTarget {
        runtime_mode: match target.runtime_mode {
            PackageRuntimeMode::EditorHost => ProjectPackageLockRuntimeMode::EditorHost,
            PackageRuntimeMode::ClientRuntime => ProjectPackageLockRuntimeMode::ClientRuntime,
        },
        platform: match target.platform {
            PackageTargetPlatform::Windows => ProjectPackageLockPlatform::Windows,
            PackageTargetPlatform::Linux => ProjectPackageLockPlatform::Linux,
            PackageTargetPlatform::Macos => ProjectPackageLockPlatform::Macos,
        },
    };
    if lock.project != project || lock.target != expected_target {
        return Err(AccountError::PackageUnavailable);
    }
    lock.validate()
        .map_err(|_| AccountError::PackageUnavailable)?;
    lock.require_canonical_wire()
        .map_err(|_| AccountError::PackageUnavailable)?;
    Ok(lock)
}

#[cfg(test)]
#[path = "tests/project_lock.rs"]
mod tests;
