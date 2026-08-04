use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use zircon_runtime::asset::project::AssetMetaDocument;

use super::paths::{has_extension, is_zmeta};
use crate::error::{ShaderPrewarmAssetScanError, ShaderPrewarmAssetScanResult};

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

    fn load_snapshot(
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

    fn load_snapshot_index(
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

    fn write_snapshot(
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
struct ShaderPrewarmAssetInventorySnapshot {
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
struct ShaderPrewarmAssetInventorySnapshotIndex {
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

    fn into_inventory(self, root: &Path) -> ShaderPrewarmAssetInventory {
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

    fn is_current(&self, root: &Path, max_resident_text_bytes: usize) -> bool {
        snapshot_entries_are_current(root, &self.files, &self.directories)
            && self.resident_text_bytes <= max_resident_text_bytes
    }

    fn changed_file_paths(
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

    fn is_current(&self, root: &Path, max_resident_text_bytes: usize) -> bool {
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
            && !is_reparse_point(&metadata)
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

fn snapshot_path_for(root: &Path, snapshot_root: &Path) -> Option<PathBuf> {
    let root_identity = root_identity(root)?;
    Some(snapshot_root.join(format!(
        "{}.json",
        blake3::hash(root_identity.as_bytes()).to_hex()
    )))
}

fn snapshot_index_path_for(root: &Path, snapshot_root: &Path) -> Option<PathBuf> {
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

fn temporary_snapshot_path(snapshot_path: &Path) -> PathBuf {
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

/// Only a strict child of the scanned root is eligible for exclusion. This
/// keeps a shared snapshot root from silently reusing an inventory with a
/// different scan shape.
fn nested_excluded_root_identity(root: &Path, excluded_root: Option<&Path>) -> Option<String> {
    let canonical_root = fs::canonicalize(root).ok()?;
    excluded_root
        .and_then(|path| fs::canonicalize(path).ok())
        .filter(|path| path.starts_with(&canonical_root) && path != &canonical_root)
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

fn is_inventory_text_path(path: &Path) -> bool {
    has_extension(path, "wgsl")
        || has_extension(path, "zshader")
        || has_extension(path, "zmaterial")
}

fn inventory_text_read_error(path: &Path, source: std::io::Error) -> ShaderPrewarmAssetScanError {
    if has_extension(path, "zshader") {
        ShaderPrewarmAssetScanError::ReadZShader {
            path: path.to_path_buf(),
            source,
        }
    } else if has_extension(path, "zmaterial") {
        ShaderPrewarmAssetScanError::ReadZMaterial {
            path: path.to_path_buf(),
            source,
        }
    } else {
        ShaderPrewarmAssetScanError::ReadWgsl {
            path: path.to_path_buf(),
            source,
        }
    }
}

fn collect_file_paths(
    directory: &Path,
    canonical_root: &Path,
    canonical_excluded_root: Option<&Path>,
    visited: &mut HashSet<PathBuf>,
    paths: &mut Vec<PathBuf>,
    directories: &mut Vec<PathBuf>,
) -> ShaderPrewarmAssetScanResult<()> {
    let canonical_directory = fs::canonicalize(directory).map_err(|source| {
        ShaderPrewarmAssetScanError::ReadAssetRoot {
            path: directory.to_path_buf(),
            source,
        }
    })?;
    ensure_below_root(canonical_root, &canonical_directory)?;
    if canonical_excluded_root
        .is_some_and(|excluded_root| canonical_directory.starts_with(excluded_root))
    {
        return Ok(());
    }
    if !visited.insert(canonical_directory.clone()) {
        return Err(ShaderPrewarmAssetScanError::AssetInventoryDirectoryCycle {
            root: canonical_root.to_path_buf(),
            path: canonical_directory,
        });
    }
    directories.push(directory.to_path_buf());
    let mut entries = fs::read_dir(directory)
        .map_err(|source| ShaderPrewarmAssetScanError::ReadAssetRoot {
            path: directory.to_path_buf(),
            source,
        })?
        .map(|entry| {
            entry.map_err(|source| ShaderPrewarmAssetScanError::ReadAssetRootEntry {
                path: directory.to_path_buf(),
                source,
            })
        })
        .collect::<ShaderPrewarmAssetScanResult<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| {
            ShaderPrewarmAssetScanError::ReadAssetRoot {
                path: path.clone(),
                source,
            }
        })?;
        reject_link_or_reparse(canonical_root, &path, &metadata)?;
        let canonical_path = fs::canonicalize(&path).map_err(|source| {
            ShaderPrewarmAssetScanError::ReadAssetRoot {
                path: path.clone(),
                source,
            }
        })?;
        ensure_below_root(canonical_root, &canonical_path)?;
        if metadata.is_dir() {
            collect_file_paths(
                &path,
                canonical_root,
                canonical_excluded_root,
                visited,
                paths,
                directories,
            )?;
        } else {
            paths.push(path);
        }
    }
    Ok(())
}

fn ensure_below_root(root: &Path, path: &Path) -> ShaderPrewarmAssetScanResult<()> {
    if path.starts_with(root) {
        Ok(())
    } else {
        Err(ShaderPrewarmAssetScanError::AssetInventoryPathEscapesRoot {
            root: root.to_path_buf(),
            path: path.to_path_buf(),
        })
    }
}

fn reject_link_or_reparse(
    root: &Path,
    path: &Path,
    metadata: &fs::Metadata,
) -> ShaderPrewarmAssetScanResult<()> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) {
        Err(ShaderPrewarmAssetScanError::UnsafeAssetInventoryLink {
            root: root.to_path_buf(),
            path: path.to_path_buf(),
        })
    } else {
        Ok(())
    }
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
#[path = "asset_inventory/tests.rs"]
mod tests;
