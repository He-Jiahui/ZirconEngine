use std::fs;
use std::path::Path;

use zircon_runtime_interface::project::load_project_manifest_value_from_toml_str;
use zircon_runtime_interface::serialization::Loaded;

use super::{ProjectManifest, ProjectManifestError};

impl ProjectManifest {
    pub fn from_toml_str(document: &str) -> Result<Loaded<Self>, ProjectManifestError> {
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
        let document =
            fs::read_to_string(path).map_err(|source| ProjectManifestError::Read { source })?;
        Self::from_toml_str(&document)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ProjectManifestError> {
        Self::load_with_report(path).map(|loaded| loaded.value)
    }
}
