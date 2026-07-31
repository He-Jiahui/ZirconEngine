use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::foundation::persistence::atomic_file::{atomic_write_with_fault, AtomicWriteFault};

use super::{AssetRegistryDiagnostic, AssetRegistryEntry, AssetRegistryError, AssetRegistryIndex};

const REGISTRY_FORMAT_VERSION: u32 = 1;
const REGISTRY_FILE_NAME: &str = "asset-registry.json";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedAssetRegistry {
    format_version: u32,
    entries: Vec<AssetRegistryEntry>,
}

pub(crate) struct PreparedAssetRegistryWrite {
    pub(crate) path: PathBuf,
    pub(crate) bytes: Vec<u8>,
}

impl AssetRegistryIndex {
    pub fn load_or_rebuild(
        asset_roots: &[PathBuf],
        registry_root: impl AsRef<Path>,
    ) -> Result<Self, AssetRegistryError> {
        let registry_root = registry_root.as_ref();
        let path = registry_path(registry_root);
        if !path.exists() {
            return Self::rebuild_from_project(asset_roots, registry_root);
        }
        match load(&path) {
            Ok(index) => Ok(index),
            Err(error) => {
                let reason = error.to_string();
                let mut rebuilt = Self::rebuild_from_project(asset_roots, registry_root)?;
                rebuilt.push_diagnostic(AssetRegistryDiagnostic::CorruptPersistenceRebuilt {
                    path,
                    reason,
                });
                Ok(rebuilt)
            }
        }
    }

    pub(super) fn persist(&self, registry_root: &Path) -> Result<(), AssetRegistryError> {
        self.persist_with_atomic_fault(registry_root, AtomicWriteFault::None)
    }

    pub(crate) fn persist_with_atomic_fault(
        &self,
        registry_root: &Path,
        fault: AtomicWriteFault,
    ) -> Result<(), AssetRegistryError> {
        let prepared = self.prepare_persistence(registry_root)?;
        atomic_write_with_fault(&prepared.path, &prepared.bytes, fault)
            .map_err(|source| AssetRegistryError::io(&prepared.path, source))
    }

    pub(crate) fn prepare_persistence(
        &self,
        registry_root: &Path,
    ) -> Result<PreparedAssetRegistryWrite, AssetRegistryError> {
        fs::create_dir_all(registry_root)
            .map_err(|source| AssetRegistryError::io(registry_root, source))?;
        let path = registry_path(registry_root);
        let document = PersistedAssetRegistry {
            format_version: REGISTRY_FORMAT_VERSION,
            entries: self.entries().into_iter().cloned().collect(),
        };
        let bytes = serde_json::to_vec_pretty(&document).map_err(|source| {
            AssetRegistryError::EncodePersistence {
                path: path.clone(),
                source,
            }
        })?;
        Ok(PreparedAssetRegistryWrite { path, bytes })
    }
}

pub(crate) fn load(path: &Path) -> Result<AssetRegistryIndex, AssetRegistryError> {
    let bytes = fs::read(path).map_err(|source| AssetRegistryError::io(path, source))?;
    let document: PersistedAssetRegistry =
        serde_json::from_slice(&bytes).map_err(|source| AssetRegistryError::DecodePersistence {
            path: path.to_path_buf(),
            source,
        })?;
    if document.format_version != REGISTRY_FORMAT_VERSION {
        return Err(AssetRegistryError::UnsupportedPersistenceVersion {
            path: path.to_path_buf(),
            found: document.format_version,
            supported: REGISTRY_FORMAT_VERSION,
        });
    }
    AssetRegistryIndex::from_entries(document.entries)
}

pub(super) fn registry_path(registry_root: &Path) -> PathBuf {
    registry_root.join(REGISTRY_FILE_NAME)
}
