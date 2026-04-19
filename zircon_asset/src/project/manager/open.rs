use std::collections::HashMap;
use std::path::Path;

use zircon_resource::ResourceRegistry;

use crate::{ArtifactStore, AssetImportError, AssetImporter};

use super::super::{ProjectManifest, ProjectPaths};
use super::ProjectManager;

impl ProjectManager {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, AssetImportError> {
        let paths = ProjectPaths::from_root(root)?;
        paths.ensure_layout()?;
        let manifest = ProjectManifest::load(paths.manifest_path())?;
        Ok(Self {
            paths,
            manifest,
            registry: ResourceRegistry::default(),
            asset_ids_by_uuid: HashMap::new(),
            asset_uuids_by_id: HashMap::new(),
            importer: AssetImporter::default(),
            artifact_store: ArtifactStore,
        })
    }
}
