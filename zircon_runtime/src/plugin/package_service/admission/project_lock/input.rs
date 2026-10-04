use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::Path,
};

use crate::asset::project::ProjectManifest;
use zircon_runtime_interface::project::{
    ProjectManifestDigest, ProjectPackageLockProject, MAX_PROJECT_MANIFEST_BYTES,
};

use super::ProjectPackageLockProviderError;

pub(super) struct ProjectInput {
    pub(super) manifest: ProjectManifest,
    // The same source object remains pinned through policy/inventory capture.
    _file: File,
}

pub(super) fn read_project_manifest(
    root: &Path,
    expected: &ProjectPackageLockProject,
) -> Result<ProjectInput, ProjectPackageLockProviderError> {
    let path = root.join("zircon-project.toml");
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        // Source admission permits readers and keeps this exact manifest immutable.
        options
            .share_mode(1)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = options
        .open(&path)
        .map_err(|_| ProjectPackageLockProviderError::InvalidProjectIdentity)?;
    let metadata = file
        .metadata()
        .map_err(|_| ProjectPackageLockProviderError::InvalidProjectIdentity)?;
    if !metadata.is_file() || metadata.len() > MAX_PROJECT_MANIFEST_BYTES as u64 {
        return Err(ProjectPackageLockProviderError::InvalidProjectIdentity);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(ProjectPackageLockProviderError::InvalidProjectIdentity);
        }
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(MAX_PROJECT_MANIFEST_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ProjectPackageLockProviderError::InvalidProjectIdentity)?;
    if bytes.len() > MAX_PROJECT_MANIFEST_BYTES
        || bytes.len() as u64 != metadata.len()
        || ProjectManifestDigest::from_bytes(&bytes) != expected.manifest_digest
    {
        return Err(ProjectPackageLockProviderError::InvalidProjectIdentity);
    }
    let document = std::str::from_utf8(&bytes)
        .map_err(|_| ProjectPackageLockProviderError::InvalidProjectIdentity)?;
    let manifest = ProjectManifest::from_toml_str(document)
        .map_err(|_| ProjectPackageLockProviderError::InvalidProjectIdentity)?
        .value;
    if manifest.project_guid != expected.project_guid || manifest.plugins.selections.len() > 512 {
        return Err(ProjectPackageLockProviderError::InvalidProjectIdentity);
    }
    Ok(ProjectInput {
        manifest,
        _file: file,
    })
}

#[cfg(test)]
#[path = "tests/input.rs"]
mod tests;
