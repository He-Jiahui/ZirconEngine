use std::path::Path;

use zircon_runtime_interface::project::{
    ProjectGuid, ProjectManifestDigest, ProjectPackageLock, ProjectPackageLockAuthority,
    ProjectPackageLockError, ProjectPackageLockState, ProjectPackageLockTarget,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PackageLockProviderError {
    Unavailable,
    ProjectChanged,
    TargetChanged,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectPackageLockContext {
    pub(crate) project_guid: ProjectGuid,
    pub(crate) manifest_digest: ProjectManifestDigest,
    pub(crate) target: ProjectPackageLockTarget,
}

pub(crate) trait ProjectPackageLockProvider {
    fn capture(
        &self,
        project_root: &Path,
        expected: &ProjectPackageLockContext,
    ) -> Result<ProjectPackageLock, PackageLockProviderError>;
}

pub(crate) struct CapturedPackageLock(pub(crate) ProjectPackageLock);

impl ProjectPackageLockProvider for CapturedPackageLock {
    fn capture(
        &self,
        _project_root: &Path,
        _expected: &ProjectPackageLockContext,
    ) -> Result<ProjectPackageLock, PackageLockProviderError> {
        Ok(self.0.clone())
    }
}

/// Validates a runtime producer response before the cloud manifest is assembled.
pub(crate) fn package_lock_digest(
    provider: &dyn ProjectPackageLockProvider,
    project_root: &Path,
    expected: &ProjectPackageLockContext,
) -> Result<String, PackageLockProviderError> {
    let lock = provider.capture(project_root, expected)?;
    lock.validate()
        .map_err(|_| PackageLockProviderError::Invalid)?;
    if lock.project.project_guid != expected.project_guid
        || lock.project.manifest_digest != expected.manifest_digest
    {
        return Err(PackageLockProviderError::ProjectChanged);
    }
    if lock.target != expected.target {
        return Err(PackageLockProviderError::TargetChanged);
    }
    lock.require_canonical_wire()
        .map_err(|_| PackageLockProviderError::Invalid)?;
    lock.digest().map_err(|_| PackageLockProviderError::Invalid)
}

pub(crate) fn package_lock_state(
    provider: &dyn ProjectPackageLockProvider,
    project_root: &Path,
    expected: &ProjectPackageLockContext,
) -> Result<ProjectPackageLockState, PackageLockProviderError> {
    let lock = provider.capture(project_root, expected)?;
    validate_lock(&lock, expected)?;
    let digest = lock
        .digest()
        .map_err(|_| PackageLockProviderError::Invalid)?;
    Ok(ProjectPackageLockState::Present {
        schema_version:
            zircon_runtime_interface::project::PROJECT_PACKAGE_LOCK_STATE_SCHEMA_VERSION_V1,
        lock,
        digest,
    })
}

fn validate_lock(
    lock: &ProjectPackageLock,
    expected: &ProjectPackageLockContext,
) -> Result<(), PackageLockProviderError> {
    lock.validate()
        .map_err(|_| PackageLockProviderError::Invalid)?;
    if lock.project.project_guid != expected.project_guid
        || lock.project.manifest_digest != expected.manifest_digest
    {
        return Err(PackageLockProviderError::ProjectChanged);
    }
    if lock.target != expected.target {
        return Err(PackageLockProviderError::TargetChanged);
    }
    lock.require_canonical_wire()
        .map_err(|_| PackageLockProviderError::Invalid)
}

pub(crate) fn decode_runtime_lock(
    bytes: &[u8],
) -> Result<ProjectPackageLock, PackageLockProviderError> {
    ProjectPackageLock::decode_canonical(bytes).map_err(|error| match error {
        ProjectPackageLockError::Capacity
        | ProjectPackageLockError::Encoding
        | ProjectPackageLockError::NonCanonicalWire
        | ProjectPackageLockError::InvalidIdentity
        | ProjectPackageLockError::InvalidText
        | ProjectPackageLockError::DuplicatePlugin
        | ProjectPackageLockError::InvalidVersion
        | ProjectPackageLockError::InvalidAuthority
        | ProjectPackageLockError::UnsupportedSchema => PackageLockProviderError::Invalid,
    })
}

#[cfg(test)]
#[path = "tests/package_lock.rs"]
mod tests;
