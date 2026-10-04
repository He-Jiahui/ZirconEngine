use std::fs::File;
use std::io::Read;
use std::path::Path;

use zircon_runtime_interface::project::{
    load_project_manifest_value_from_toml_str, ProjectManifestSummaryError,
    MAX_PROJECT_MANIFEST_BYTES,
};
use zircon_runtime_interface::serialization::Loaded;

use super::{ProjectManifest, ProjectManifestError};

impl ProjectManifest {
    pub fn from_toml_str(document: &str) -> Result<Loaded<Self>, ProjectManifestError> {
        if document.len() > MAX_PROJECT_MANIFEST_BYTES {
            return Err(document_too_large(
                u64::try_from(document.len()).unwrap_or(u64::MAX),
            ));
        }
        let loaded = load_project_manifest_value_from_toml_str(document)?;
        if let Some(source_format_version) = loaded.migrated_from {
            return Err(ProjectManifestError::MigrationRequired {
                source_format_version,
            });
        }
        let manifest: ProjectManifest = serde_json::from_value(loaded.value)
            .map_err(|source| ProjectManifestError::Decode { source })?;
        let result = Loaded {
            value: manifest,
            migrated_from: loaded.migrated_from,
        };
        result.value.validate()?;
        Ok(result)
    }

    pub fn load_with_report(path: impl AsRef<Path>) -> Result<Loaded<Self>, ProjectManifestError> {
        let document = read_bounded_manifest(path.as_ref())?;
        Self::from_toml_str(&document)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ProjectManifestError> {
        Self::load_with_report(path).map(|loaded| loaded.value)
    }
}

fn read_bounded_manifest(path: &Path) -> Result<String, ProjectManifestError> {
    let file = File::open(path).map_err(|source| ProjectManifestError::Read { source })?;
    let metadata_len = file
        .metadata()
        .map_err(|source| ProjectManifestError::Read { source })?
        .len();
    if metadata_len > MAX_PROJECT_MANIFEST_BYTES as u64 {
        return Err(document_too_large(metadata_len));
    }

    let capacity_hint = usize::try_from(metadata_len).unwrap_or(MAX_PROJECT_MANIFEST_BYTES + 1);
    read_bounded_manifest_from_reader(file, capacity_hint)
}

fn read_bounded_manifest_from_reader(
    reader: impl Read,
    capacity_hint: usize,
) -> Result<String, ProjectManifestError> {
    let mut bytes = Vec::with_capacity(capacity_hint.min(MAX_PROJECT_MANIFEST_BYTES));
    reader
        .take((MAX_PROJECT_MANIFEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|source| ProjectManifestError::Read { source })?;
    if bytes.len() > MAX_PROJECT_MANIFEST_BYTES {
        return Err(document_too_large(bytes.len() as u64));
    }
    String::from_utf8(bytes).map_err(|source| {
        ProjectManifestError::Summary(ProjectManifestSummaryError::InvalidUtf8 {
            source: source.utf8_error(),
        })
    })
}

fn document_too_large(found: u64) -> ProjectManifestError {
    ProjectManifestError::Summary(ProjectManifestSummaryError::DocumentTooLarge {
        max: MAX_PROJECT_MANIFEST_BYTES,
        found: usize::try_from(found).unwrap_or(usize::MAX),
    })
}

#[cfg(test)]
#[path = "tests/load.rs"]
mod tests;
