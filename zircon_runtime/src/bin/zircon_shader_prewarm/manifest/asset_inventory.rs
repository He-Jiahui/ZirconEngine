#[path = "asset_inventory/snapshot.rs"]
mod snapshot;
#[path = "asset_inventory/traversal.rs"]
mod traversal;

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::AssetMetaDocument;

use super::paths::is_zmeta;
use crate::error::{ShaderPrewarmAssetScanError, ShaderPrewarmAssetScanResult};
#[cfg(test)]
use snapshot::{snapshot_index_path_for, snapshot_path_for, temporary_snapshot_path};
use traversal::{
    collect_file_paths, inventory_text_read_error, is_inventory_text_path,
    nested_excluded_root_identity, reject_link_or_reparse,
};

/// One deterministic directory walk shared by the prewarm manifest scanners.
#[derive(Clone, Debug)]
pub(crate) struct ShaderPrewarmAssetInventory {
    paths: Vec<PathBuf>,
    directories: Vec<PathBuf>,
    meta_paths: Vec<PathBuf>,
    metadata_by_path: BTreeMap<PathBuf, AssetMetaDocument>,
    text_by_path: BTreeMap<PathBuf, String>,
    changed_paths: BTreeSet<PathBuf>,
}

impl ShaderPrewarmAssetInventory {
    pub(crate) fn collect(root: &Path) -> ShaderPrewarmAssetScanResult<Self> {
        Self::collect_fresh(root)
    }

    pub(crate) fn collect_with_warm_snapshot(
        root: &Path,
        snapshot_root: &Path,
        max_resident_text_bytes: usize,
    ) -> ShaderPrewarmAssetScanResult<Self> {
        Self::collect_with_warm_snapshot_excluding(
            root,
            snapshot_root,
            None,
            max_resident_text_bytes,
        )
    }

    pub(crate) fn collect_with_warm_snapshot_excluding(
        root: &Path,
        snapshot_root: &Path,
        excluded_root: Option<&Path>,
        max_resident_text_bytes: usize,
    ) -> ShaderPrewarmAssetScanResult<Self> {
        let excluded_root_identity = nested_excluded_root_identity(root, excluded_root);
        if let Some(snapshot) =
            Self::load_snapshot(root, snapshot_root, excluded_root_identity.as_deref())
        {
            if snapshot.is_current(root, max_resident_text_bytes) {
                return Ok(snapshot.into_inventory(root));
            }
            let mut inventory = Self::collect_fresh_with_text_budget_excluding(
                root,
                max_resident_text_bytes,
                excluded_root,
            )?;
            inventory.changed_paths = snapshot.changed_file_paths(root, &inventory);
            inventory.write_snapshot(root, snapshot_root, excluded_root_identity.as_deref())?;
            return Ok(inventory);
        }
        let inventory = Self::collect_fresh_with_text_budget_excluding(
            root,
            max_resident_text_bytes,
            excluded_root,
        )?;
        inventory.write_snapshot(root, snapshot_root, excluded_root_identity.as_deref())?;
        Ok(inventory)
    }

    /// Checks only the compact warm-snapshot index. The command-line path
    /// uses this before deciding whether an unchanged root needs its bounded
    /// metadata and source payload hydrated at all.
    pub(crate) fn warm_snapshot_is_current_excluding(
        root: &Path,
        snapshot_root: &Path,
        excluded_root: Option<&Path>,
        max_resident_text_bytes: usize,
    ) -> bool {
        let excluded_root_identity = nested_excluded_root_identity(root, excluded_root);
        Self::load_snapshot_index(root, snapshot_root, excluded_root_identity.as_deref())
            .is_some_and(|snapshot| snapshot.is_current(root, max_resident_text_bytes))
    }

    fn collect_fresh(root: &Path) -> ShaderPrewarmAssetScanResult<Self> {
        Self::collect_fresh_with_text_budget_excluding(root, 64 * 1024 * 1024, None)
    }

    fn collect_fresh_with_text_budget_excluding(
        root: &Path,
        max_resident_text_bytes: usize,
        excluded_root: Option<&Path>,
    ) -> ShaderPrewarmAssetScanResult<Self> {
        let metadata = match fs::symlink_metadata(root) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self {
                    paths: Vec::new(),
                    directories: Vec::new(),
                    meta_paths: Vec::new(),
                    metadata_by_path: BTreeMap::new(),
                    text_by_path: BTreeMap::new(),
                    changed_paths: BTreeSet::new(),
                });
            }
            Err(source) => {
                return Err(ShaderPrewarmAssetScanError::ReadAssetRoot {
                    path: root.to_path_buf(),
                    source,
                });
            }
        };
        reject_link_or_reparse(root, root, &metadata)?;
        let canonical_root = fs::canonicalize(root).map_err(|source| {
            ShaderPrewarmAssetScanError::ReadAssetRoot {
                path: root.to_path_buf(),
                source,
            }
        })?;
        let canonical_excluded_root = excluded_root
            .and_then(|path| fs::canonicalize(path).ok())
            .filter(|path| path.starts_with(&canonical_root) && path != &canonical_root);
        let mut paths = Vec::new();
        let mut directories = Vec::new();
        let mut visited = HashSet::new();
        collect_file_paths(
            root,
            &canonical_root,
            canonical_excluded_root.as_deref(),
            &mut visited,
            &mut paths,
            &mut directories,
        )?;
        paths.sort();
        directories.sort();
        let meta_paths: Vec<PathBuf> = paths
            .iter()
            .filter(|path| is_zmeta(path))
            .cloned()
            .collect();
        let mut metadata_by_path = BTreeMap::new();
        for meta_path in &meta_paths {
            let metadata = AssetMetaDocument::load(meta_path).map_err(|source| {
                ShaderPrewarmAssetScanError::LoadShaderMetadata {
                    path: meta_path.clone(),
                    source,
                }
            })?;
            metadata_by_path.insert(meta_path.clone(), metadata);
        }
        let mut text_by_path = BTreeMap::new();
        let mut resident_text_bytes = 0usize;
        for path in &paths {
            if !is_inventory_text_path(path) {
                continue;
            }
            let text = fs::read_to_string(path)
                .map_err(|source| inventory_text_read_error(path, source))?;
            resident_text_bytes = resident_text_bytes.checked_add(text.len()).ok_or_else(|| {
                ShaderPrewarmAssetScanError::AssetInventoryTextBudgetExceeded {
                    requested_bytes: usize::MAX,
                    max_bytes: max_resident_text_bytes,
                }
            })?;
            if resident_text_bytes > max_resident_text_bytes {
                return Err(
                    ShaderPrewarmAssetScanError::AssetInventoryTextBudgetExceeded {
                        requested_bytes: resident_text_bytes,
                        max_bytes: max_resident_text_bytes,
                    },
                );
            }
            text_by_path.insert(path.clone(), text);
        }
        Ok(Self {
            changed_paths: paths.iter().cloned().collect(),
            paths,
            directories,
            meta_paths,
            metadata_by_path,
            text_by_path,
        })
    }

    pub(crate) fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    pub(crate) fn meta_paths(&self) -> &[PathBuf] {
        &self.meta_paths
    }

    pub(crate) fn metadata(&self, path: &Path) -> Option<&AssetMetaDocument> {
        self.metadata_by_path.get(path)
    }

    pub(crate) fn metadata_by_path(&self) -> &BTreeMap<PathBuf, AssetMetaDocument> {
        &self.metadata_by_path
    }

    pub(crate) fn text(&self, path: &Path) -> Option<&str> {
        self.text_by_path.get(path).map(String::as_str)
    }

    /// Files are reported only when a fresh scan differs from the preceding
    /// snapshot. A cold scan intentionally reports every discovered file.
    pub(crate) fn changed_paths(&self) -> &BTreeSet<PathBuf> {
        &self.changed_paths
    }
}

#[cfg(test)]
#[path = "asset_inventory/tests.rs"]
mod tests;
