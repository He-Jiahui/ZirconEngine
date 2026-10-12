use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use zircon_runtime::asset::project::AssetMetaDocument;

use super::ShaderPrewarmAssetInventory;
use crate::error::{ShaderPrewarmAssetScanError, ShaderPrewarmAssetScanResult};

impl ShaderPrewarmAssetInventory {
    pub(super) fn load_snapshot(
        root: &Path,
        snapshot_root: &Path,
        excluded_root_identity: Option<&str>,
    ) -> Option<ShaderPrewarmAssetInventorySnapshot> {
        let snapshot_path = snapshot_path_for(root, snapshot_root)?;
        let bytes = fs::read(snapshot_path).ok()?;
        let snapshot =
            serde_json::from_slice::<ShaderPrewarmAssetInventorySnapshot>(&bytes).ok()?;
        if snapshot.schema_version != ShaderPrewarmAssetInventorySnapshot::SCHEMA_VERSION
            || snapshot.root_identity != root_identity(root)?
            || snapshot.excluded_root_identity.as_deref() != excluded_root_identity
            || !snapshot.has_safe_relative_paths()
        {
            return None;
        }
        Some(snapshot)
    }

    pub(super) fn load_snapshot_index(
        root: &Path,
        snapshot_root: &Path,
        excluded_root_identity: Option<&str>,
    ) -> Option<ShaderPrewarmAssetInventorySnapshotIndex> {
        let snapshot_index_path = snapshot_index_path_for(root, snapshot_root)?;
        let file = fs::File::open(snapshot_index_path).ok()?;
        let reader = BufReader::new(file);
        let snapshot =
            serde_json::from_reader::<_, ShaderPrewarmAssetInventorySnapshotIndex>(reader).ok()?;
        if snapshot.schema_version != ShaderPrewarmAssetInventorySnapshot::SCHEMA_VERSION
            || snapshot.root_identity != root_identity(root)?
            || snapshot.excluded_root_identity.as_deref() != excluded_root_identity
            || !snapshot.has_safe_relative_paths()
        {
            return None;
        }
        Some(snapshot)
    }

    pub(super) fn write_snapshot(
        &self,
        root: &Path,
        snapshot_root: &Path,
        excluded_root_identity: Option<&str>,
    ) -> ShaderPrewarmAssetScanResult<()> {
        let Some(snapshot_path) = snapshot_path_for(root, snapshot_root) else {
            return Ok(());
        };
        let Some(files) = self
            .paths
            .iter()
            .map(|path| SnapshotEntry::from_path(root, path))
            .collect::<Option<Vec<_>>>()
        else {
            return Ok(());
        };
        let Some(directories) = self
            .directories
            .iter()
            .map(|path| SnapshotEntry::from_path(root, path))
            .collect::<Option<Vec<_>>>()
        else {
            return Ok(());
        };
        let metadata_by_relative_path = self
            .metadata_by_path
            .iter()
            .filter_map(|(path, document)| {
                path.strip_prefix(root)
                    .ok()
                    .map(|relative| (relative.to_path_buf(), document))
            })
            .collect();
        let text_by_relative_path = self
            .text_by_path
            .iter()
            .filter_map(|(path, text)| {
                path.strip_prefix(root)
                    .ok()
                    .map(|relative| (relative.to_path_buf(), text))
            })
            .collect();
        let resident_text_bytes = self
            .text_by_path
            .values()
            .try_fold(0usize, |total, text| total.checked_add(text.len()))
            .unwrap_or(usize::MAX);
        let root_identity = root_identity(root).unwrap_or_default();
        let excluded_root_identity = excluded_root_identity.map(str::to_owned);
        let snapshot_index = ShaderPrewarmAssetInventorySnapshotIndex {
            schema_version: ShaderPrewarmAssetInventorySnapshot::SCHEMA_VERSION,
            root_identity: root_identity.clone(),
            excluded_root_identity: excluded_root_identity.clone(),
            files: files.clone(),
            directories: directories.clone(),
            resident_text_bytes,
        };
        let snapshot = ShaderPrewarmAssetInventorySnapshotRef {
            schema_version: ShaderPrewarmAssetInventorySnapshot::SCHEMA_VERSION,
            root_identity,
            excluded_root_identity,
            files,
            directories,
            resident_text_bytes,
            metadata_by_relative_path,
            text_by_relative_path,
        };
        if let Some(parent) = snapshot_path.parent() {
            fs::create_dir_all(parent).map_err(|source| {
                ShaderPrewarmAssetScanError::WriteWarmInventorySnapshot {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }
        write_snapshot_json(&snapshot_path, &snapshot)?;
        let Some(snapshot_index_path) = snapshot_index_path_for(root, snapshot_root) else {
            return Ok(());
        };
        // The index is the readiness marker: publish it after the full payload
        // so an external-input invocation can always hydrate a current index.
        write_snapshot_json(&snapshot_index_path, &snapshot_index)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct ShaderPrewarmAssetInventorySnapshot {
    schema_version: u32,
    root_identity: String,
    excluded_root_identity: Option<String>,
    files: Vec<SnapshotEntry>,
    directories: Vec<SnapshotEntry>,
    resident_text_bytes: usize,
    metadata_by_relative_path: BTreeMap<PathBuf, AssetMetaDocument>,
    text_by_relative_path: BTreeMap<PathBuf, String>,
}

/// Small warm-path payload: enough to prove source-tree freshness without
/// deserializing the cached metadata and WGSL bodies.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct ShaderPrewarmAssetInventorySnapshotIndex {
    schema_version: u32,
    root_identity: String,
    excluded_root_identity: Option<String>,
    files: Vec<SnapshotEntry>,
    directories: Vec<SnapshotEntry>,
    resident_text_bytes: usize,
}

/// Serialization borrows the bounded inventory so snapshot persistence never
/// creates a second in-memory copy of every WGSL, zshader, and material body.
#[derive(Serialize)]
struct ShaderPrewarmAssetInventorySnapshotRef<'a> {
    schema_version: u32,
    root_identity: String,
    excluded_root_identity: Option<String>,
    files: Vec<SnapshotEntry>,
    directories: Vec<SnapshotEntry>,
    resident_text_bytes: usize,
    metadata_by_relative_path: BTreeMap<PathBuf, &'a AssetMetaDocument>,
    text_by_relative_path: BTreeMap<PathBuf, &'a String>,
}

impl ShaderPrewarmAssetInventorySnapshot {
    const SCHEMA_VERSION: u32 = 4;

    pub(super) fn into_inventory(self, root: &Path) -> ShaderPrewarmAssetInventory {
        let paths = self
            .files
            .iter()
            .map(|entry| root.join(&entry.relative_path))
            .collect::<Vec<_>>();
        let directories = self
            .directories
            .iter()
            .map(|entry| root.join(&entry.relative_path))
            .collect::<Vec<_>>();
        let meta_paths = self
            .metadata_by_relative_path
            .keys()
            .map(|path| root.join(path))
            .collect::<Vec<_>>();
        let metadata_by_path = self
            .metadata_by_relative_path
            .into_iter()
            .map(|(path, document)| (root.join(path), document))
            .collect();
        let text_by_path = self
            .text_by_relative_path
            .into_iter()
            .map(|(path, text)| (root.join(path), text))
            .collect();
        ShaderPrewarmAssetInventory {
            paths,
            directories,
            meta_paths,
            metadata_by_path,
            text_by_path,
            changed_paths: BTreeSet::new(),
        }
    }

    fn has_safe_relative_paths(&self) -> bool {
        if !snapshot_entry_paths_are_safe(&self.files, &self.directories) {
            return false;
        }
        let file_paths = self
            .files
            .iter()
            .map(|entry| entry.relative_path.clone())
            .collect::<BTreeSet<_>>();
        self.metadata_by_relative_path
            .keys()
            .chain(self.text_by_relative_path.keys())
            .all(|path| is_safe_relative_child(path, false) && file_paths.contains(path))
    }

    pub(super) fn is_current(&self, root: &Path, max_resident_text_bytes: usize) -> bool {
        snapshot_entries_are_current(root, &self.files, &self.directories)
            && self.resident_text_bytes <= max_resident_text_bytes
    }

    pub(super) fn changed_file_paths(
        &self,
        root: &Path,
        current: &ShaderPrewarmAssetInventory,
    ) -> BTreeSet<PathBuf> {
        let previous_entries = self
            .files
            .iter()
            .map(|entry| (entry.relative_path.clone(), entry))
            .collect::<BTreeMap<_, _>>();
        let current_entries = current
            .paths
            .iter()
            .filter_map(|path| SnapshotEntry::from_path(root, path))
            .map(|entry| (entry.relative_path.clone(), entry))
            .collect::<BTreeMap<_, _>>();
        let mut relative_paths = BTreeSet::new();
        relative_paths.extend(previous_entries.keys().cloned());
        relative_paths.extend(current_entries.keys().cloned());
        relative_paths
            .into_iter()
            .filter(|path| previous_entries.get(path).copied() != current_entries.get(path))
            .map(|path| root.join(path))
            .collect()
    }
}

impl ShaderPrewarmAssetInventorySnapshotIndex {
    fn has_safe_relative_paths(&self) -> bool {
        snapshot_entry_paths_are_safe(&self.files, &self.directories)
    }

    pub(super) fn is_current(&self, root: &Path, max_resident_text_bytes: usize) -> bool {
        snapshot_entries_are_current(root, &self.files, &self.directories)
            && self.resident_text_bytes <= max_resident_text_bytes
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct SnapshotEntry {
    relative_path: PathBuf,
    byte_len: u64,
    modified_nanos: u64,
}

impl SnapshotEntry {
    fn from_path(root: &Path, path: &Path) -> Option<Self> {
        let metadata = fs::metadata(path).ok()?;
        Some(Self {
            relative_path: path.strip_prefix(root).ok()?.to_path_buf(),
            byte_len: metadata.len(),
            modified_nanos: modified_nanos(&metadata)?,
        })
    }

    fn matches(&self, root: &Path, allow_root: bool) -> bool {
        if !is_safe_relative_child(&self.relative_path, allow_root) {
            return false;
        }
        let path = root.join(&self.relative_path);
        let Ok(metadata) = fs::symlink_metadata(path) else {
            return false;
        };
        !metadata.file_type().is_symlink()
            && !super::traversal::is_reparse_point(&metadata)
            && metadata.len() == self.byte_len
            && modified_nanos(&metadata) == Some(self.modified_nanos)
    }
}

fn snapshot_entries_are_current(
    root: &Path,
    files: &[SnapshotEntry],
    directories: &[SnapshotEntry],
) -> bool {
    directories.iter().all(|entry| entry.matches(root, true))
        && files.iter().all(|entry| entry.matches(root, false))
}

fn snapshot_entry_paths_are_safe(files: &[SnapshotEntry], directories: &[SnapshotEntry]) -> bool {
    if !directories
        .iter()
        .all(|entry| is_safe_relative_child(&entry.relative_path, true))
        || !files
            .iter()
            .all(|entry| is_safe_relative_child(&entry.relative_path, false))
    {
        return false;
    }

    let directory_paths = directories
        .iter()
        .map(|entry| entry.relative_path.clone())
        .collect::<BTreeSet<_>>();
    directory_paths.contains(Path::new(""))
        && directories.iter().all(|entry| {
            snapshot_directory_and_ancestors_are_recorded(&entry.relative_path, &directory_paths)
        })
        && files.iter().all(|entry| {
            entry.relative_path.parent().is_some_and(|parent| {
                snapshot_directory_and_ancestors_are_recorded(parent, &directory_paths)
            })
        })
}

fn snapshot_directory_and_ancestors_are_recorded(
    directory: &Path,
    directory_paths: &BTreeSet<PathBuf>,
) -> bool {
    let mut current = Some(directory);
    while let Some(path) = current {
        if !directory_paths.contains(path) {
            return false;
        }
        current = path.parent();
    }
    true
}

/// Warm snapshots are untrusted persisted input. Only ordinary relative path
/// components can be joined to the scanned root; the root directory itself is
/// represented by an empty directory entry.
fn is_safe_relative_child(path: &Path, allow_root: bool) -> bool {
    (allow_root && path.as_os_str().is_empty())
        || (!path.as_os_str().is_empty()
            && path
                .components()
                .all(|component| matches!(component, Component::Normal(_))))
}

pub(super) fn snapshot_path_for(root: &Path, snapshot_root: &Path) -> Option<PathBuf> {
    let root_identity = root_identity(root)?;
    Some(snapshot_root.join(format!(
        "{}.json",
        blake3::hash(root_identity.as_bytes()).to_hex()
    )))
}

pub(super) fn snapshot_index_path_for(root: &Path, snapshot_root: &Path) -> Option<PathBuf> {
    Some(snapshot_path_for(root, snapshot_root)?.with_extension("index.json"))
}

fn write_snapshot_json(
    snapshot_path: &Path,
    value: &impl Serialize,
) -> ShaderPrewarmAssetScanResult<()> {
    let temporary_path = temporary_snapshot_path(snapshot_path);
    let file = fs::File::create(&temporary_path).map_err(|source| {
        ShaderPrewarmAssetScanError::WriteWarmInventorySnapshot {
            path: temporary_path.clone(),
            source,
        }
    })?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, value).map_err(|source| {
        ShaderPrewarmAssetScanError::EncodeWarmInventorySnapshot {
            path: temporary_path.clone(),
            source,
        }
    })?;
    writer.flush().map_err(
        |source| ShaderPrewarmAssetScanError::WriteWarmInventorySnapshot {
            path: temporary_path.clone(),
            source,
        },
    )?;
    drop(writer);
    fs::rename(&temporary_path, snapshot_path).map_err(|source| {
        ShaderPrewarmAssetScanError::WriteWarmInventorySnapshot {
            path: snapshot_path.to_path_buf(),
            source,
        }
    })
}

pub(super) fn temporary_snapshot_path(snapshot_path: &Path) -> PathBuf {
    let extension = snapshot_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("snapshot");
    snapshot_path.with_extension(format!("{extension}.{}.tmp", std::process::id()))
}

fn root_identity(root: &Path) -> Option<String> {
    fs::canonicalize(root)
        .ok()
        .map(|path| path.to_string_lossy().to_string())
}

fn modified_nanos(metadata: &fs::Metadata) -> Option<u64> {
    metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_nanos()).ok())
}
