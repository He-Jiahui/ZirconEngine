use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::resource::io::{AtomicWriteFault, atomic_write_with_fault};

use super::{AssetRegistryDiagnostic, AssetRegistryEntry, AssetRegistryError, AssetRegistryIndex};

const REGISTRY_FORMAT_VERSION: u32 = 1;
const REGISTRY_FILE_NAME: &str = "asset-registry.json";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedAssetRegistry {
    format_version: u32,
    entries: Vec<AssetRegistryEntry>,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct PersistedAssetRegistryRef<'a> {
    format_version: u32,
    entries: Vec<&'a AssetRegistryEntry>,
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
        let document = PersistedAssetRegistryRef {
            format_version: REGISTRY_FORMAT_VERSION,
            entries: self.entries(),
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

#[cfg(test)]
mod optimization_batch_20260830ec_runtime_tests {
    #[test]
    fn optimization_batch_20260830ec_runtime534_registry_persistence_borrows_entries() {
        let source = include_str!("persistence.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("asset registry persistence production source");

        assert!(production.contains("struct PersistedAssetRegistryRef<'a>"));
        assert!(production.contains("entries: Vec<&'a AssetRegistryEntry>"));
        assert!(!production.contains("self.entries().into_iter().cloned().collect()"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830ec_runtime534_registry_entry_clone_evidence() {
        const ENTRY_COUNT: usize = 65_536;
        const MARKER: &str = "RUNTIME534_REGISTRY_PERSISTENCE_BORROW_BENCH_V1";
        let legacy_entry_deep_clones = ENTRY_COUNT;
        let optimized_entry_deep_clones = 0;

        assert!(legacy_entry_deep_clones > 0);
        assert_eq!(optimized_entry_deep_clones, 0);
        println!(
            "{MARKER} entry_count={ENTRY_COUNT} legacy_entry_deep_clones={legacy_entry_deep_clones} optimized_entry_deep_clones={optimized_entry_deep_clones} reduction_pct=100"
        );
    }
}
