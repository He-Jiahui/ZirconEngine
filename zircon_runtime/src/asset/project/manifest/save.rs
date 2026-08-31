use std::path::Path;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use zircon_runtime_interface::project::PROJECT_MANIFEST_FORMAT_VERSION;

use crate::core::resource::io::{atomic_write_with_fault, AtomicWriteFault};

use super::{ProjectManifest, ProjectManifestError};

impl ProjectManifest {
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ProjectManifestError> {
        self.save_with_atomic_fault(path, AtomicWriteFault::None)
    }

    pub(crate) fn save_with_atomic_fault(
        &self,
        path: impl AsRef<Path>,
        fault: AtomicWriteFault,
    ) -> Result<(), ProjectManifestError> {
        self.validate()?;
        let path = path.as_ref();
        let document = serialize_current_project_manifest(self)
            .map_err(|source| ProjectManifestError::Encode { source })?;
        atomic_write_with_fault(path, document.as_bytes(), fault)
            .map_err(|source| ProjectManifestError::Write { source })
    }
}

struct CurrentProjectManifest<'a>(&'a ProjectManifest);

impl Serialize for CurrentProjectManifest<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let manifest = self.0;
        let mut state = serializer.serialize_struct("ProjectManifest", 13)?;
        state.serialize_field("name", &manifest.name)?;
        state.serialize_field("format_version", &PROJECT_MANIFEST_FORMAT_VERSION)?;
        state.serialize_field("project_guid", &manifest.project_guid)?;
        if let Some(engine_version_req) = &manifest.engine_version_req {
            state.serialize_field("engine_version_req", engine_version_req)?;
        }
        state.serialize_field("default_scene", &manifest.default_scene)?;
        if !manifest.ui_roots.is_empty() {
            state.serialize_field("ui_roots", &manifest.ui_roots)?;
        }
        state.serialize_field("asset_roots", &manifest.asset_roots)?;
        if let Some(settings) = &manifest.settings {
            state.serialize_field("settings", settings)?;
        }
        if let Some(asset_manifest) = &manifest.asset_manifest {
            state.serialize_field("asset_manifest", asset_manifest)?;
        }
        state.serialize_field("library_version", &manifest.library_version)?;
        if !manifest.plugins.is_empty() {
            state.serialize_field("plugins", &manifest.plugins)?;
        }
        if !manifest.scripts.is_empty() {
            state.serialize_field("scripts", &manifest.scripts)?;
        }
        if !manifest.export_profiles.is_empty() {
            state.serialize_field("export_profiles", &manifest.export_profiles)?;
        }
        state.end()
    }
}

fn serialize_current_project_manifest(
    manifest: &ProjectManifest,
) -> Result<String, toml::ser::Error> {
    toml::to_string_pretty(&CurrentProjectManifest(manifest))
}

#[cfg(test)]
#[path = "save/borrowed_serialization_tests.rs"]
mod borrowed_serialization_tests;
