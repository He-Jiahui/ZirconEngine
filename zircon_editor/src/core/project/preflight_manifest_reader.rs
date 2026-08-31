use std::fs::File;
use std::io::Read;
use std::path::Path;

use zircon_runtime::asset::project::{ProjectManifest, ProjectManifestError};
use zircon_runtime_interface::project::{ProjectManifestDigest, ProjectManifestSummary};

use super::ProjectAuthorityError;

/// Editor admission only needs a compact descriptor; large payloads belong in asset pipelines.
pub(super) const MAX_PROJECT_PREFLIGHT_MANIFEST_BYTES: usize = 4 * 1024 * 1024;

pub(super) struct PreflightManifestInspection {
    pub(super) digest: ProjectManifestDigest,
    pub(super) summary: ProjectManifestSummary,
    pub(super) migrated_from: Option<u32>,
    pub(super) manifest: Option<ProjectManifest>,
}

pub(super) fn inspect_project_manifest(
    path: &Path,
) -> Result<PreflightManifestInspection, ProjectAuthorityError> {
    let file = File::open(path).map_err(|source| {
        ProjectAuthorityError::io("open project manifest for preflight", path, source)
    })?;
    let max_bytes = MAX_PROJECT_PREFLIGHT_MANIFEST_BYTES as u64;
    if file
        .metadata()
        .map_err(|source| {
            ProjectAuthorityError::io("inspect project manifest for preflight", path, source)
        })?
        .len()
        > max_bytes
    {
        return Err(ProjectAuthorityError::ManifestPreflightTooLarge {
            path: path.to_path_buf(),
            max_bytes: MAX_PROJECT_PREFLIGHT_MANIFEST_BYTES,
        });
    }

    let mut bytes = Vec::with_capacity(MAX_PROJECT_PREFLIGHT_MANIFEST_BYTES.min(4096));
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| {
            ProjectAuthorityError::io("read project manifest for preflight", path, source)
        })?;
    if bytes.len() > MAX_PROJECT_PREFLIGHT_MANIFEST_BYTES {
        return Err(ProjectAuthorityError::ManifestPreflightTooLarge {
            path: path.to_path_buf(),
            max_bytes: MAX_PROJECT_PREFLIGHT_MANIFEST_BYTES,
        });
    }

    let source = String::from_utf8(bytes).map_err(|source| {
        ProjectAuthorityError::io(
            "decode project manifest for preflight",
            path,
            std::io::Error::new(std::io::ErrorKind::InvalidData, source),
        )
    })?;
    let digest = ProjectManifestDigest::from_bytes(source.as_bytes());
    let summary = ProjectManifestSummary::parse_toml_str(&source)
        .map_err(ProjectManifestError::Summary)
        .map_err(|source| ProjectAuthorityError::Manifest { source })?;
    let manifest = if summary.migrated_from.is_none() {
        Some(ProjectManifest::from_toml_str(&source)?.value)
    } else {
        None
    };
    Ok(PreflightManifestInspection {
        digest,
        migrated_from: summary.migrated_from,
        summary: summary.value,
        manifest,
    })
}
