use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::project::AssetMetaDocument;
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetUri, AssetUuid};

use super::{AssetRegistryDiagnostic, AssetRegistryEntry, AssetRegistryError, AssetRegistryIndex};

pub(super) struct ScannedMeta {
    pub(super) path: PathBuf,
    pub(super) document: AssetMetaDocument,
}

impl AssetRegistryIndex {
    pub fn rebuild_from_project(
        asset_roots: &[PathBuf],
        registry_root: impl AsRef<Path>,
    ) -> Result<Self, AssetRegistryError> {
        let registry_root = registry_root.as_ref();
        let index = Self::build_from_project(asset_roots)?;
        index.persist(registry_root)?;
        Ok(index)
    }

    fn build_from_project(asset_roots: &[PathBuf]) -> Result<Self, AssetRegistryError> {
        let mut metas = scan_project_metas(asset_roots)?;
        let mut diagnostics = Vec::new();
        normalize_duplicate_guids(&mut metas, &mut diagnostics, &HashMap::new())?;
        let mut index = build_index(&metas, diagnostics)?;
        refresh_dependency_edges(&mut index, &metas);
        Ok(index)
    }

    pub(crate) fn rebuild_after_import(
        &mut self,
        asset_roots: &[PathBuf],
    ) -> Result<(), AssetRegistryError> {
        let retained = self
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                matches!(
                    diagnostic,
                    AssetRegistryDiagnostic::DuplicateGuidReminted { .. }
                        | AssetRegistryDiagnostic::CorruptPersistenceRebuilt { .. }
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut rebuilt = Self::build_from_project(asset_roots)?;
        rebuilt.diagnostics.splice(0..0, retained);
        *self = rebuilt;
        Ok(())
    }

    pub(crate) fn prepare_duplicate_guids(
        &self,
        asset_roots: &[PathBuf],
        changes: Option<&[AssetChange]>,
    ) -> Result<Vec<AssetRegistryDiagnostic>, AssetRegistryError> {
        let mut metas = scan_project_metas(asset_roots)?;
        let mut diagnostics = Vec::new();
        let owners = changes
            .map(|changes| identity_owners_for_changes(self, changes))
            .unwrap_or_else(|| identity_owners(self));
        normalize_duplicate_guids(&mut metas, &mut diagnostics, &owners)?;
        Ok(diagnostics)
    }

    pub(crate) fn prepare_duplicate_guids_from_loaded(
        &self,
        documents_by_path: &mut BTreeMap<PathBuf, AssetMetaDocument>,
        changes: Option<&[AssetChange]>,
    ) -> Vec<AssetRegistryDiagnostic> {
        let mut diagnostics = Vec::new();
        let mut owners = changes
            .map(|changes| identity_owners_for_changes(self, changes))
            .unwrap_or_else(|| identity_owners(self));
        for document in documents_by_path.values_mut() {
            normalize_duplicate_guid_document(document, &mut diagnostics, &mut owners);
        }
        diagnostics
    }
}

pub(super) fn scan_project_metas(
    asset_roots: &[PathBuf],
) -> Result<Vec<ScannedMeta>, AssetRegistryError> {
    let mut meta_paths = Vec::new();
    for root in asset_roots {
        collect_meta_paths_for_root(root, &mut meta_paths)?;
    }
    scan_meta_paths(&meta_paths)
}

pub(super) fn scan_meta_paths(
    meta_paths: &[PathBuf],
) -> Result<Vec<ScannedMeta>, AssetRegistryError> {
    let mut metas = Vec::new();
    for path in meta_paths {
        if !source_path_for_meta(path).is_some_and(|source| source.exists()) {
            continue;
        }
        let document =
            AssetMetaDocument::load(path).map_err(|source| AssetRegistryError::io(path, source))?;
        metas.push(ScannedMeta {
            path: path.clone(),
            document,
        });
    }
    Ok(metas)
}

fn source_path_for_meta(meta_path: &Path) -> Option<PathBuf> {
    let file_name = meta_path.file_name()?.to_str()?;
    let source_name = file_name.strip_suffix(".zmeta")?;
    Some(meta_path.with_file_name(source_name))
}

pub(super) fn normalize_duplicate_guids(
    metas: &mut [ScannedMeta],
    diagnostics: &mut Vec<AssetRegistryDiagnostic>,
    preferred_owners: &HashMap<AssetUuid, AssetUri>,
) -> Result<Vec<AssetUri>, AssetRegistryError> {
    let mut first_path_by_uuid = preferred_owners.clone();
    let mut reminted_paths = Vec::new();
    for scanned in metas {
        let changed = normalize_duplicate_guid_document(
            &mut scanned.document,
            diagnostics,
            &mut first_path_by_uuid,
        );
        if changed {
            reminted_paths.push(scanned.document.url.clone());
            scanned
                .document
                .save(&scanned.path)
                .map_err(|source| AssetRegistryError::io(&scanned.path, source))?;
        }
    }
    Ok(reminted_paths)
}

fn normalize_duplicate_guid_document(
    document: &mut AssetMetaDocument,
    diagnostics: &mut Vec<AssetRegistryDiagnostic>,
    owners: &mut HashMap<AssetUuid, AssetUri>,
) -> bool {
    let mut changed = false;
    let original_root = document.uuid;
    if let Some(first_path) = duplicate_owner(owners, original_root, &document.url) {
        let replacement = unique_uuid(owners);
        diagnostics.push(AssetRegistryDiagnostic::DuplicateGuidReminted {
            original: original_root,
            first_path,
            path: document.url.clone(),
            replacement,
        });
        document.uuid = replacement;
        for entry in &mut document.entries {
            if entry.url.label().is_none() && entry.uuid == original_root {
                entry.uuid = replacement;
            }
        }
        changed = true;
    }
    owners.insert(document.uuid, document.url.clone());

    for entry in &mut document.entries {
        if entry.url.label().is_none() {
            entry.uuid = document.uuid;
            continue;
        }
        if let Some(first_path) = duplicate_owner(owners, entry.uuid, &entry.url) {
            let original = entry.uuid;
            let replacement = unique_uuid(owners);
            diagnostics.push(AssetRegistryDiagnostic::DuplicateGuidReminted {
                original,
                first_path,
                path: entry.url.clone(),
                replacement,
            });
            entry.uuid = replacement;
            changed = true;
        }
        owners.insert(entry.uuid, entry.url.clone());
    }
    changed
}

pub(super) fn identity_owners(index: &AssetRegistryIndex) -> HashMap<AssetUuid, AssetUri> {
    index
        .entries_by_uuid
        .values()
        .map(|entry| (entry.uuid(), entry.path().clone()))
        .collect()
}

pub(super) fn identity_owners_for_changes(
    index: &AssetRegistryIndex,
    changes: &[AssetChange],
) -> HashMap<AssetUuid, AssetUri> {
    let mut owners = identity_owners(index);
    for change in changes {
        match change.kind {
            AssetChangeKind::Added | AssetChangeKind::Modified => {}
            AssetChangeKind::Removed => {
                owners.retain(|_, owner| !same_source_path(owner, &change.uri));
            }
            AssetChangeKind::Renamed => {
                let Some(previous) = &change.previous_uri else {
                    continue;
                };
                for owner in owners.values_mut() {
                    if same_source_path(owner, previous) {
                        *owner = AssetUri::new(
                            change.uri.scheme(),
                            change.uri.path().to_string(),
                            owner.label().map(ToOwned::to_owned),
                        )
                        .expect("renaming an existing asset URI must preserve a valid URI");
                    }
                }
            }
        }
    }
    owners
}

fn duplicate_owner(
    owners: &HashMap<AssetUuid, AssetUri>,
    uuid: AssetUuid,
    candidate: &AssetUri,
) -> Option<AssetUri> {
    owners
        .get(&uuid)
        .filter(|owner| *owner != candidate)
        .cloned()
}

pub(super) fn replace_entries_for_paths(
    index: &mut AssetRegistryIndex,
    metas: &[ScannedMeta],
    paths: &[AssetUri],
) -> Result<(), AssetRegistryError> {
    for path in paths {
        index.remove_source_path(path);
    }
    for scanned in metas {
        if paths
            .iter()
            .any(|path| same_source_path(path, &scanned.document.url))
        {
            for entry in registry_entries(&scanned.document) {
                index.insert_checked(entry)?;
            }
        }
    }
    Ok(())
}

fn same_source_path(left: &AssetUri, right: &AssetUri) -> bool {
    left.scheme() == right.scheme() && left.path() == right.path()
}

pub(super) fn build_index(
    metas: &[ScannedMeta],
    diagnostics: Vec<AssetRegistryDiagnostic>,
) -> Result<AssetRegistryIndex, AssetRegistryError> {
    build_index_from_documents(metas.iter().map(|scanned| &scanned.document), diagnostics)
}

/// Builds registry entries from already-owned metadata without creating a
/// second metadata owner. Asset inventories use this during one-pass scans.
pub(super) fn build_index_from_documents<'a>(
    documents: impl IntoIterator<Item = &'a AssetMetaDocument>,
    diagnostics: Vec<AssetRegistryDiagnostic>,
) -> Result<AssetRegistryIndex, AssetRegistryError> {
    let mut index = AssetRegistryIndex::default();
    index.diagnostics = diagnostics;
    for document in documents {
        for entry in registry_entries(document) {
            index.insert_checked(entry)?;
        }
    }
    Ok(index)
}

pub(super) fn refresh_dependency_edges(index: &mut AssetRegistryIndex, metas: &[ScannedMeta]) {
    refresh_dependency_edges_from_documents(index, metas.iter().map(|scanned| &scanned.document));
}

/// Resolves dependency edges from borrowed metadata. The resulting index owns
/// only its compact query records and dependency paths.
pub(super) fn refresh_dependency_edges_from_documents<'a>(
    index: &mut AssetRegistryIndex,
    documents: impl IntoIterator<Item = &'a AssetMetaDocument>,
) {
    let mut dependency_paths: HashMap<AssetUuid, &[AssetUri]> = HashMap::new();
    for meta in documents {
        if !meta.entries.iter().any(|entry| entry.url.label().is_none()) {
            dependency_paths.insert(meta.uuid, &meta.dependencies);
        }
        for entry in &meta.entries {
            dependency_paths.insert(entry.uuid, &entry.dependencies);
        }
    }
    let uuids_by_path = &index.uuids_by_path;
    let mut unresolved = Vec::new();
    let mut resolved_by_uuid = Vec::with_capacity(index.entries_by_uuid.len());
    for entry in index.entries_by_uuid.values() {
        let paths = dependency_paths
            .get(&entry.uuid())
            .copied()
            .unwrap_or_default();
        let mut dependencies = Vec::new();
        for path in paths {
            if let Some(uuid) = uuids_by_path.get(path).copied() {
                dependencies.push(uuid);
            } else {
                unresolved.push(AssetRegistryDiagnostic::UnresolvedDependency {
                    owner: entry.uuid(),
                    path: path.clone(),
                });
            }
        }
        resolved_by_uuid.push((entry.uuid(), dependencies));
    }
    for (uuid, paths) in dependency_paths {
        index.replace_dependency_paths(uuid, paths.to_vec());
    }
    for (uuid, dependencies) in resolved_by_uuid {
        index.replace_dependencies(uuid, dependencies);
    }
    index.diagnostics.retain(|diagnostic| {
        !matches!(
            diagnostic,
            AssetRegistryDiagnostic::UnresolvedDependency { .. }
        )
    });
    index.diagnostics.extend(unresolved);
}

pub(super) fn registry_entries(meta: &AssetMetaDocument) -> Vec<AssetRegistryEntry> {
    let mut entries = Vec::new();
    if !meta.entries.iter().any(|entry| entry.url.label().is_none()) {
        entries.push(
            AssetRegistryEntry::new(
                meta.uuid,
                meta.url.clone(),
                meta.asset_kind,
                meta.source_digest.clone(),
            )
            .with_tags(meta.tags.clone()),
        );
    }
    entries.extend(meta.entries.iter().map(|entry| {
        let tags = if entry.url.label().is_none() {
            meta.tags.clone()
        } else {
            entry.tags.clone()
        };
        AssetRegistryEntry::new(
            entry.uuid,
            entry.url.clone(),
            entry.asset_kind,
            meta.source_digest.clone(),
        )
        .with_tags(tags)
    }));
    entries
}

fn unique_uuid(used: &HashMap<AssetUuid, AssetUri>) -> AssetUuid {
    loop {
        let candidate = AssetUuid::new();
        if !used.contains_key(&candidate) {
            return candidate;
        }
    }
}

fn collect_meta_paths_for_root(
    root: &Path,
    paths: &mut Vec<PathBuf>,
) -> Result<(), AssetRegistryError> {
    let metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(AssetRegistryError::io(root, error)),
    };
    reject_link_or_reparse(root, root, &metadata)?;
    let canonical_root =
        fs::canonicalize(root).map_err(|source| AssetRegistryError::io(root, source))?;
    let mut visited = HashSet::new();
    collect_meta_paths(root, &canonical_root, &mut visited, paths)
}

fn collect_meta_paths(
    directory: &Path,
    canonical_root: &Path,
    visited: &mut HashSet<PathBuf>,
    paths: &mut Vec<PathBuf>,
) -> Result<(), AssetRegistryError> {
    let canonical_directory =
        fs::canonicalize(directory).map_err(|source| AssetRegistryError::io(directory, source))?;
    ensure_below_root(canonical_root, &canonical_directory)?;
    if !visited.insert(canonical_directory.clone()) {
        return Err(AssetRegistryError::MetadataDirectoryCycle {
            root: canonical_root.to_path_buf(),
            path: canonical_directory,
        });
    }
    let entries =
        fs::read_dir(directory).map_err(|source| AssetRegistryError::io(directory, source))?;
    for entry in entries {
        let entry = entry.map_err(|source| AssetRegistryError::io(directory, source))?;
        let path = entry.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|source| AssetRegistryError::io(&path, source))?;
        reject_link_or_reparse(canonical_root, &path, &metadata)?;
        let canonical_path =
            fs::canonicalize(&path).map_err(|source| AssetRegistryError::io(&path, source))?;
        ensure_below_root(canonical_root, &canonical_path)?;
        if metadata.is_dir() {
            collect_meta_paths(&path, canonical_root, visited, paths)?;
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".zmeta"))
        {
            paths.push(path);
        }
    }
    Ok(())
}

fn ensure_below_root(root: &Path, path: &Path) -> Result<(), AssetRegistryError> {
    if path.starts_with(root) {
        Ok(())
    } else {
        Err(AssetRegistryError::MetadataPathEscapesRoot {
            root: root.to_path_buf(),
            path: path.to_path_buf(),
        })
    }
}

fn reject_link_or_reparse(
    root: &Path,
    path: &Path,
    metadata: &fs::Metadata,
) -> Result<(), AssetRegistryError> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) {
        Err(AssetRegistryError::UnsafeMetadataLink {
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
