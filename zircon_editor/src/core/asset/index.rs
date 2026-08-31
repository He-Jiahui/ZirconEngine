use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use thiserror::Error;
use zircon_runtime::asset::project::{
    AssetMetaDocument, ProjectCatalogInputGeneration, ProjectManager,
};
use zircon_runtime::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use zircon_runtime::asset::watch::AssetWatchEvent;
use zircon_runtime::asset::{AssetKind, AssetUri, AssetUuid};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorAssetImportState {
    Ready,
    Stale,
    Importing,
    Broken,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EditorAssetIndexError {
    #[error("runtime asset registry has no entry for metadata UUID {uuid}")]
    RuntimeEntryMissing { uuid: AssetUuid },
    #[error(
        "metadata path for {uuid} does not match runtime registry: metadata={metadata}, runtime={runtime}"
    )]
    MetadataPathMismatch {
        uuid: AssetUuid,
        metadata: AssetUri,
        runtime: AssetUri,
    },
    #[error(
        "metadata type for {uuid} does not match runtime registry: metadata={metadata:?}, runtime={runtime:?}"
    )]
    MetadataTypeMismatch {
        uuid: AssetUuid,
        metadata: AssetKind,
        runtime: AssetKind,
    },
    #[error("metadata digest for {uuid} does not match the runtime registry")]
    MetadataDigestMismatch { uuid: AssetUuid },
    #[error("metadata tags for {uuid} do not match the runtime registry")]
    MetadataTagsMismatch { uuid: AssetUuid },
    #[error("metadata document contains duplicate projected UUID {uuid}")]
    DuplicateMetadataUuid { uuid: AssetUuid },
    #[error(
        "metadata UUID {uuid} is already projected by document {existing_document}, not incoming document {incoming_document}"
    )]
    MetadataUuidOwnedByOtherDocument {
        uuid: AssetUuid,
        existing_document: AssetUuid,
        incoming_document: AssetUuid,
    },
    #[error("asset UUID {uuid} is not present in the runtime registry")]
    AssetNotIndexed { uuid: AssetUuid },
}

#[derive(Clone, Debug)]
struct AssetMetaProjection {
    document: Arc<AssetMetaDocument>,
    entry_index: Option<usize>,
}

impl AssetMetaProjection {
    fn uuid(&self) -> AssetUuid {
        self.entry()
            .map(|entry| entry.uuid)
            .unwrap_or(self.document.uuid)
    }

    fn path(&self) -> &AssetUri {
        self.entry()
            .map(|entry| &entry.url)
            .unwrap_or(&self.document.url)
    }

    fn type_marker(&self) -> AssetKind {
        self.entry()
            .map(|entry| entry.asset_kind)
            .unwrap_or(self.document.asset_kind)
    }

    fn tags(&self) -> &std::collections::BTreeSet<String> {
        match self.entry() {
            Some(entry) if entry.url.label().is_some() => &entry.tags,
            _ => &self.document.tags,
        }
    }

    fn artifact_locator(&self) -> Option<&AssetUri> {
        self.entry()
            .and_then(|entry| entry.artifact_locator.as_ref())
            .or_else(|| {
                self.entry_index
                    .is_none()
                    .then_some(self.document.artifact_locator.as_ref())
                    .flatten()
            })
    }

    fn entry(&self) -> Option<&zircon_runtime::asset::project::AssetMetaEntry> {
        self.entry_index
            .and_then(|index| self.document.entries.get(index))
    }
}

#[derive(Clone, Debug)]
pub struct EditorAssetIndex {
    runtime_registry: Arc<AssetRegistryIndex>,
    runtime_registry_revision: u64,
    metadata_by_uuid: HashMap<AssetUuid, AssetMetaProjection>,
    document_members: HashMap<AssetUuid, HashSet<AssetUuid>>,
    dirty_uuids: HashSet<AssetUuid>,
    importing_uuids: HashSet<AssetUuid>,
    pending_dirty_paths: HashSet<AssetUri>,
    catalog_input_generation: Option<Arc<ProjectCatalogInputGeneration>>,
}

impl EditorAssetIndex {
    pub fn new(runtime_registry: Arc<AssetRegistryIndex>) -> Self {
        Self {
            runtime_registry,
            runtime_registry_revision: 0,
            metadata_by_uuid: HashMap::new(),
            document_members: HashMap::new(),
            dirty_uuids: HashSet::new(),
            importing_uuids: HashSet::new(),
            pending_dirty_paths: HashSet::new(),
            catalog_input_generation: None,
        }
    }

    /// Builds the editor projection from the active Runtime project snapshot.
    /// Runtime retains registry and metadata authority; this index owns only editor-local state.
    pub fn from_runtime_project(project: &ProjectManager) -> Result<Self, EditorAssetIndexError> {
        let catalog_input = project.catalog_input_generation();
        let mut index = Self::from_runtime_snapshot(
            project.asset_registry_shared(),
            catalog_input
                .records()
                .map(|record| Arc::new(record.meta().clone())),
        )?;
        index.catalog_input_generation = Some(catalog_input);
        Ok(index)
    }

    pub fn catalog_input_generation(&self) -> Option<&Arc<ProjectCatalogInputGeneration>> {
        self.catalog_input_generation.as_ref()
    }

    fn from_runtime_snapshot(
        runtime_registry: Arc<AssetRegistryIndex>,
        metadata: impl IntoIterator<Item = Arc<AssetMetaDocument>>,
    ) -> Result<Self, EditorAssetIndexError> {
        let mut index = Self::new(runtime_registry);
        for document in metadata {
            index.ingest_meta_document(document)?;
        }
        Ok(index)
    }

    pub fn runtime_registry(&self) -> &Arc<AssetRegistryIndex> {
        &self.runtime_registry
    }

    pub fn len(&self) -> usize {
        self.runtime_registry.len()
    }

    pub fn is_empty(&self) -> bool {
        self.runtime_registry.is_empty()
    }

    pub fn rows(&self) -> Vec<EditorAssetRow<'_>> {
        self.runtime_registry
            .entries()
            .into_iter()
            .map(|runtime_entry| self.project_row(runtime_entry))
            .collect()
    }

    pub fn row_by_uuid(&self, uuid: AssetUuid) -> Option<EditorAssetRow<'_>> {
        self.runtime_registry
            .entry_by_uuid(uuid)
            .map(|runtime_entry| self.project_row(runtime_entry))
    }

    pub fn row_by_path(&self, path: &AssetUri) -> Option<EditorAssetRow<'_>> {
        self.runtime_registry
            .entry_by_path(path)
            .map(|runtime_entry| self.project_row(runtime_entry))
    }

    pub fn ingest_meta_document(
        &mut self,
        document: Arc<AssetMetaDocument>,
    ) -> Result<(), EditorAssetIndexError> {
        let entry_indices = projected_entry_indices(&document)?;
        let projections = entry_indices
            .into_iter()
            .map(|entry_index| AssetMetaProjection {
                document: Arc::clone(&document),
                entry_index,
            })
            .collect::<Vec<_>>();

        for projection in &projections {
            let runtime_entry = self
                .runtime_registry
                .entry_by_uuid(projection.uuid())
                .ok_or(EditorAssetIndexError::RuntimeEntryMissing {
                    uuid: projection.uuid(),
                })?;
            validate_projection(runtime_entry, projection)?;
        }

        let document_uuid = document.uuid;
        let projected_uuids = projections
            .iter()
            .map(AssetMetaProjection::uuid)
            .collect::<HashSet<_>>();
        for uuid in &projected_uuids {
            if let Some(existing) = self
                .metadata_by_uuid
                .get(uuid)
                .filter(|existing| existing.document.uuid != document_uuid)
            {
                return Err(EditorAssetIndexError::MetadataUuidOwnedByOtherDocument {
                    uuid: *uuid,
                    existing_document: existing.document.uuid,
                    incoming_document: document_uuid,
                });
            }
        }

        if let Some(previous_members) = self.document_members.get(&document_uuid) {
            for removed_uuid in previous_members.difference(&projected_uuids) {
                self.metadata_by_uuid.remove(removed_uuid);
            }
        }
        for projection in projections {
            let uuid = projection.uuid();
            self.pending_dirty_paths.remove(projection.path());
            self.dirty_uuids.remove(&uuid);
            self.metadata_by_uuid.insert(uuid, projection);
        }
        self.document_members.insert(document_uuid, projected_uuids);
        Ok(())
    }

    pub fn apply_watch_events(&mut self, events: &[AssetWatchEvent]) {
        for event in events {
            match event {
                AssetWatchEvent::Added(path) | AssetWatchEvent::Modified(path) => {
                    self.mark_or_queue_path(path);
                }
                AssetWatchEvent::Removed(path) => {
                    self.pending_dirty_paths.remove(path);
                    self.mark_existing_path(path);
                }
                AssetWatchEvent::Renamed { from, to } => {
                    self.pending_dirty_paths.remove(from);
                    self.mark_existing_path(from);
                    self.mark_or_queue_path(to);
                }
            }
        }
    }

    pub fn replace_runtime_registry(&mut self, runtime_registry: Arc<AssetRegistryIndex>) {
        self.runtime_registry = runtime_registry;
        self.runtime_registry_revision = self.runtime_registry_revision.wrapping_add(1);
        self.metadata_by_uuid.retain(|uuid, projection| {
            self.runtime_registry
                .entry_by_uuid(*uuid)
                .is_some_and(|entry| validate_projection(entry, projection).is_ok())
        });
        self.document_members.retain(|_, members| {
            members.retain(|uuid| self.metadata_by_uuid.contains_key(uuid));
            !members.is_empty()
        });

        self.retain_transient_state_for_current_registry();
    }

    /// Replaces Runtime-authoritative catalog data while retaining valid editor-local work.
    ///
    /// A full Runtime catalog projection is deliberately rebuilt rather than incrementally
    /// mutated. Import jobs and watcher events can overlap that rebuild, so their transient
    /// state must move to the replacement only when its UUIDs remain authoritative.
    pub fn replace_authoritative_projection(&mut self, mut projection: Self) {
        projection.runtime_registry_revision = self.runtime_registry_revision.wrapping_add(1);
        projection.dirty_uuids = std::mem::take(&mut self.dirty_uuids);
        projection.importing_uuids = std::mem::take(&mut self.importing_uuids);
        projection.pending_dirty_paths = std::mem::take(&mut self.pending_dirty_paths);
        projection.retain_transient_state_for_current_registry();
        *self = projection;
    }

    /// Copies valid editor-local state into a speculative Runtime catalog projection.
    ///
    /// Call this before building catalog rows from the candidate. The final commit should still
    /// use `replace_authoritative_projection` so the live transient collections move atomically.
    pub fn inherit_transient_state_from(&mut self, current: &Self) {
        self.runtime_registry_revision = current.runtime_registry_revision.wrapping_add(1);
        self.dirty_uuids.clone_from(&current.dirty_uuids);
        self.importing_uuids.clone_from(&current.importing_uuids);
        self.pending_dirty_paths
            .clone_from(&current.pending_dirty_paths);
        self.retain_transient_state_for_current_registry();
    }

    fn retain_transient_state_for_current_registry(&mut self) {
        self.dirty_uuids
            .retain(|uuid| self.runtime_registry.entry_by_uuid(*uuid).is_some());
        self.importing_uuids
            .retain(|uuid| self.runtime_registry.entry_by_uuid(*uuid).is_some());

        let runtime_registry = &self.runtime_registry;
        let dirty_uuids = &mut self.dirty_uuids;
        let pending_dirty_paths = &mut self.pending_dirty_paths;
        pending_dirty_paths.retain(|path| {
            let Some(entry) = runtime_registry.entry_by_path(path) else {
                return true;
            };
            dirty_uuids.insert(entry.uuid());
            false
        });
    }

    pub fn begin_import(&mut self, uuid: AssetUuid) -> Result<(), EditorAssetIndexError> {
        if self.runtime_registry.entry_by_uuid(uuid).is_none() {
            return Err(EditorAssetIndexError::AssetNotIndexed { uuid });
        }
        self.importing_uuids.insert(uuid);
        Ok(())
    }

    pub fn clear_import(&mut self, uuid: AssetUuid) {
        self.importing_uuids.remove(&uuid);
    }

    pub(super) fn import_generation(&self, path: &AssetUri) -> Option<EditorAssetImportGeneration> {
        let entry = self.runtime_registry.entry_by_path(path)?;
        Some(EditorAssetImportGeneration {
            registry_revision: self.runtime_registry_revision,
            uuid: entry.uuid(),
            uri: Arc::new(entry.path().clone()),
            source_digest: Arc::from(entry.source_digest()),
        })
    }

    pub(super) fn is_current_import_generation(
        &self,
        generation: &EditorAssetImportGeneration,
    ) -> bool {
        self.runtime_registry_revision == generation.registry_revision
            && self
                .runtime_registry
                .entry_by_path(generation.uri())
                .is_some_and(|entry| {
                    entry.uuid() == generation.uuid()
                        && entry.source_digest() == generation.source_digest().as_ref()
                })
    }

    pub(super) fn begin_import_generation(
        &mut self,
        generation: &EditorAssetImportGeneration,
    ) -> bool {
        if !self.is_current_import_generation(generation) {
            return false;
        }
        self.importing_uuids.insert(generation.uuid());
        true
    }

    pub fn pending_dirty_path_count(&self) -> usize {
        self.pending_dirty_paths.len()
    }

    pub(crate) fn dirty_count(&self) -> usize {
        self.dirty_uuids.len()
    }

    fn project_row<'a>(&'a self, runtime_entry: &'a AssetRegistryEntry) -> EditorAssetRow<'a> {
        let uuid = runtime_entry.uuid();
        EditorAssetRow {
            runtime_entry,
            metadata: self.metadata_by_uuid.get(&uuid),
            dirty: self.dirty_uuids.contains(&uuid),
            importing: self.importing_uuids.contains(&uuid),
        }
    }

    fn mark_or_queue_path(&mut self, path: &AssetUri) {
        if !self.mark_existing_path(path) {
            self.pending_dirty_paths.insert(path.clone());
        }
    }

    fn mark_existing_path(&mut self, path: &AssetUri) -> bool {
        let Some(entry) = self.runtime_registry.entry_by_path(path) else {
            return false;
        };
        self.dirty_uuids.insert(entry.uuid());
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EditorAssetImportGeneration {
    registry_revision: u64,
    uuid: AssetUuid,
    uri: Arc<AssetUri>,
    source_digest: Arc<str>,
}

impl EditorAssetImportGeneration {
    pub(super) fn uuid(&self) -> AssetUuid {
        self.uuid
    }

    pub(super) fn uri(&self) -> &Arc<AssetUri> {
        &self.uri
    }

    pub(super) fn source_digest(&self) -> &Arc<str> {
        &self.source_digest
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EditorAssetRow<'a> {
    runtime_entry: &'a AssetRegistryEntry,
    metadata: Option<&'a AssetMetaProjection>,
    dirty: bool,
    importing: bool,
}

impl<'a> EditorAssetRow<'a> {
    pub fn runtime_entry(&self) -> &'a AssetRegistryEntry {
        self.runtime_entry
    }

    pub fn uuid(&self) -> AssetUuid {
        self.runtime_entry.uuid()
    }

    pub fn path(&self) -> &'a AssetUri {
        self.runtime_entry.path()
    }

    pub fn type_marker(&self) -> AssetKind {
        self.runtime_entry.type_marker()
    }

    pub fn tags(&self) -> &'a std::collections::BTreeSet<String> {
        self.runtime_entry.tags()
    }

    pub fn dependencies(&self) -> &'a [AssetUuid] {
        self.runtime_entry.dependencies()
    }

    pub fn source_digest(&self) -> &'a str {
        self.runtime_entry.source_digest()
    }

    pub fn source_mtime_unix_ms(&self) -> Option<u64> {
        self.metadata
            .map(|metadata| metadata.document.source_mtime_unix_ms)
    }

    pub fn import_products(&self) -> impl Iterator<Item = &'a AssetUri> + 'a {
        self.metadata
            .and_then(AssetMetaProjection::artifact_locator)
            .into_iter()
    }

    pub fn import_valid(&self) -> bool {
        self.metadata
            .and_then(AssetMetaProjection::artifact_locator)
            .is_some()
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn import_state(&self) -> EditorAssetImportState {
        if self.importing {
            return EditorAssetImportState::Importing;
        }
        if self.dirty {
            return EditorAssetImportState::Stale;
        }
        match self.metadata {
            Some(metadata) if metadata.artifact_locator().is_some() => {
                EditorAssetImportState::Ready
            }
            Some(_) => EditorAssetImportState::Broken,
            None => EditorAssetImportState::Stale,
        }
    }
}

fn projected_entry_indices(
    document: &AssetMetaDocument,
) -> Result<Vec<Option<usize>>, EditorAssetIndexError> {
    let mut entry_indices = Vec::with_capacity(document.entries.len() + 1);
    if !document
        .entries
        .iter()
        .any(|entry| entry.url.label().is_none())
    {
        entry_indices.push(None);
    }
    entry_indices.extend((0..document.entries.len()).map(Some));

    let mut seen = HashSet::with_capacity(entry_indices.len());
    for entry_index in &entry_indices {
        let uuid = entry_index
            .and_then(|index| document.entries.get(index))
            .map(|entry| entry.uuid)
            .unwrap_or(document.uuid);
        if !seen.insert(uuid) {
            return Err(EditorAssetIndexError::DuplicateMetadataUuid { uuid });
        }
    }
    Ok(entry_indices)
}

fn validate_projection(
    runtime_entry: &AssetRegistryEntry,
    metadata: &AssetMetaProjection,
) -> Result<(), EditorAssetIndexError> {
    let uuid = metadata.uuid();
    if runtime_entry.path() != metadata.path() {
        return Err(EditorAssetIndexError::MetadataPathMismatch {
            uuid,
            metadata: metadata.path().clone(),
            runtime: runtime_entry.path().clone(),
        });
    }
    if runtime_entry.type_marker() != metadata.type_marker() {
        return Err(EditorAssetIndexError::MetadataTypeMismatch {
            uuid,
            metadata: metadata.type_marker(),
            runtime: runtime_entry.type_marker(),
        });
    }
    if runtime_entry.source_digest() != metadata.document.source_digest {
        return Err(EditorAssetIndexError::MetadataDigestMismatch { uuid });
    }
    if runtime_entry.tags() != metadata.tags() {
        return Err(EditorAssetIndexError::MetadataTagsMismatch { uuid });
    }
    Ok(())
}

#[cfg(test)]
#[path = "index/tests.rs"]
mod tests;
