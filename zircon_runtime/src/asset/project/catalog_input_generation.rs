use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::asset::{AssetId, AssetReference, AssetUri, ReferenceRepair};
use crate::core::resource::{ResourceLocator, ResourceRecord};

use super::{AssetMetaDocument, PackageAssetRegistry, ProjectManifest};

static NEXT_CATALOG_INPUT_GENERATION: AtomicU64 = AtomicU64::new(1);

const PROJECT_CATALOG_INPUT_SHARD_COUNT: usize = 64;

type ProjectCatalogInputShard = HashMap<AssetId, Arc<ProjectCatalogInputRecord>>;

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectCatalogInputRecord {
    resource: ResourceRecord,
    source_path: PathBuf,
    meta_path: PathBuf,
    meta: AssetMetaDocument,
    source_mtime_unix_ms: u64,
    artifact_reference_revision: u64,
    direct_references: Arc<[AssetReference]>,
    reference_repairs: Arc<[ReferenceRepair]>,
}

impl ProjectCatalogInputRecord {
    pub fn resource(&self) -> &ResourceRecord {
        &self.resource
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn meta_path(&self) -> &Path {
        &self.meta_path
    }

    pub fn meta(&self) -> &AssetMetaDocument {
        &self.meta
    }

    pub fn source_mtime_unix_ms(&self) -> u64 {
        self.source_mtime_unix_ms
    }

    pub fn artifact_reference_revision(&self) -> u64 {
        self.artifact_reference_revision
    }

    pub fn direct_references(&self) -> &[AssetReference] {
        &self.direct_references
    }

    /// Observations requiring an explicit authoring-document fix-up.
    pub fn reference_repairs(&self) -> &[ReferenceRepair] {
        &self.reference_repairs
    }

    fn from_source(resource: ResourceRecord, source: ProjectCatalogInputSource) -> Self {
        let mut direct_references = source.direct_references;
        direct_references.sort_by(|left, right| {
            left.locator
                .to_string()
                .cmp(&right.locator.to_string())
                .then_with(|| left.uuid.to_string().cmp(&right.uuid.to_string()))
        });
        direct_references.dedup();
        Self {
            artifact_reference_revision: resource.revision,
            resource,
            source_path: source.source_path,
            meta_path: source.meta_path,
            meta: source.meta,
            source_mtime_unix_ms: source.source_mtime_unix_ms,
            direct_references: direct_references.into(),
            reference_repairs: source.reference_repairs.into(),
        }
    }

    fn with_resource(&self, resource: ResourceRecord) -> Self {
        let artifact_reference_revision = resource.revision;
        Self {
            resource,
            source_path: self.source_path.clone(),
            meta_path: self.meta_path.clone(),
            meta: self.meta.clone(),
            source_mtime_unix_ms: self.source_mtime_unix_ms,
            artifact_reference_revision,
            direct_references: Arc::clone(&self.direct_references),
            reference_repairs: Arc::clone(&self.reference_repairs),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(
        resource: ResourceRecord,
        source_path: PathBuf,
        meta_path: PathBuf,
        meta: AssetMetaDocument,
        source_mtime_unix_ms: u64,
        direct_references: Vec<AssetReference>,
    ) -> Self {
        Self::from_source(
            resource,
            ProjectCatalogInputSource::new(
                source_path,
                meta_path,
                meta,
                source_mtime_unix_ms,
                direct_references,
                Vec::new(),
            ),
        )
    }

    #[cfg(test)]
    pub(crate) fn set_locator_for_test(&mut self, locator: AssetUri) {
        self.resource.primary_locator = locator.clone();
        self.meta.url = locator;
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProjectCatalogInputSource {
    source_path: PathBuf,
    meta_path: PathBuf,
    meta: AssetMetaDocument,
    source_mtime_unix_ms: u64,
    direct_references: Vec<AssetReference>,
    reference_repairs: Vec<ReferenceRepair>,
}

impl ProjectCatalogInputSource {
    pub(crate) fn new(
        source_path: PathBuf,
        meta_path: PathBuf,
        meta: AssetMetaDocument,
        source_mtime_unix_ms: u64,
        direct_references: Vec<AssetReference>,
        reference_repairs: Vec<ReferenceRepair>,
    ) -> Self {
        Self {
            source_path,
            meta_path,
            meta,
            source_mtime_unix_ms,
            direct_references,
            reference_repairs,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProjectCatalogInputGeneration {
    sequence: u64,
    predecessor_sequence: Option<u64>,
    delta_from_predecessor: ProjectCatalogInputDelta,
    project_root: PathBuf,
    manifest: ProjectManifest,
    package_assets: PackageAssetRegistry,
    records: Vec<Arc<ProjectCatalogInputShard>>,
}

impl ProjectCatalogInputGeneration {
    pub(crate) fn initial(
        project_root: &Path,
        manifest: ProjectManifest,
        package_assets: PackageAssetRegistry,
    ) -> Arc<Self> {
        Arc::new(Self {
            sequence: next_sequence(),
            predecessor_sequence: None,
            delta_from_predecessor: ProjectCatalogInputDelta::default(),
            project_root: project_root.to_path_buf(),
            manifest,
            package_assets,
            records: empty_catalog_input_shards(),
        })
    }

    pub(crate) fn publish(
        previous: &Arc<Self>,
        project_root: &Path,
        manifest: &ProjectManifest,
        package_assets: &PackageAssetRegistry,
        records: impl IntoIterator<Item = ResourceRecord>,
        sources: HashMap<AssetId, ProjectCatalogInputSource>,
    ) -> Arc<Self> {
        let records = catalog_input_shards_from_full_scan(records, sources);
        if previous.project_root == project_root
            && previous.manifest == *manifest
            && previous.package_assets == *package_assets
            && previous.records == records
        {
            return Arc::clone(previous);
        }
        let mut current = Self {
            sequence: next_sequence(),
            predecessor_sequence: Some(previous.sequence),
            delta_from_predecessor: ProjectCatalogInputDelta::default(),
            project_root: project_root.to_path_buf(),
            manifest: manifest.clone(),
            package_assets: package_assets.clone(),
            records,
        };
        current.delta_from_predecessor = catalog_delta_from_full_records(&current, previous);
        Arc::new(current)
    }

    /// Publishes a mutation without rebuilding unchanged catalog-input records.
    ///
    /// Full scans still use [`Self::publish`]. Targeted imports provide only records whose
    /// resource state can have changed plus source data for roots whose catalog input changed.
    pub(crate) fn publish_targeted(
        previous: &Arc<Self>,
        project_root: &Path,
        manifest: &ProjectManifest,
        package_assets: &PackageAssetRegistry,
        updated_records: impl IntoIterator<Item = ResourceRecord>,
        mut updated_sources: HashMap<AssetId, ProjectCatalogInputSource>,
        removed_ids: impl IntoIterator<Item = AssetId>,
    ) -> Arc<Self> {
        let mut records = previous.records.clone();
        let mut records_changed = false;
        let mut updated_ids = HashSet::new();
        let mut touched_ids = HashSet::new();

        for id in removed_ids {
            touched_ids.insert(id);
            let shard = Arc::make_mut(&mut records[catalog_input_shard_index(&id)]);
            records_changed |= shard.remove(&id).is_some();
        }

        for resource in updated_records {
            if resource.primary_locator.label().is_some() {
                continue;
            }
            let id = resource.id;
            touched_ids.insert(id);
            if !updated_ids.insert(id) {
                continue;
            }
            let record = match updated_sources.remove(&id) {
                Some(source) => ProjectCatalogInputRecord::from_source(resource, source),
                None => match previous.record(id) {
                    Some(previous_record) => previous_record.with_resource(resource),
                    None => continue,
                },
            };
            let shard = Arc::make_mut(&mut records[catalog_input_shard_index(&id)]);
            if shard
                .get(&id)
                .is_some_and(|existing| existing.as_ref() == &record)
            {
                continue;
            }
            shard.insert(id, Arc::new(record));
            records_changed = true;
        }

        if !records_changed
            && previous.project_root == project_root
            && previous.manifest == *manifest
            && previous.package_assets == *package_assets
        {
            return Arc::clone(previous);
        }

        let mut current = Self {
            sequence: next_sequence(),
            predecessor_sequence: Some(previous.sequence),
            delta_from_predecessor: ProjectCatalogInputDelta::default(),
            project_root: project_root.to_path_buf(),
            manifest: manifest.clone(),
            package_assets: package_assets.clone(),
            records,
        };
        current.delta_from_predecessor =
            catalog_delta_from_touched_records(&current, previous, &touched_ids);
        Arc::new(current)
    }

    pub(crate) fn publish_metadata(
        previous: &Arc<Self>,
        project_root: &Path,
        manifest: &ProjectManifest,
        package_assets: &PackageAssetRegistry,
    ) -> Arc<Self> {
        if previous.project_root == project_root
            && previous.manifest == *manifest
            && previous.package_assets == *package_assets
        {
            return Arc::clone(previous);
        }
        let mut current = Self {
            sequence: next_sequence(),
            predecessor_sequence: Some(previous.sequence),
            delta_from_predecessor: ProjectCatalogInputDelta::default(),
            project_root: project_root.to_path_buf(),
            manifest: manifest.clone(),
            package_assets: package_assets.clone(),
            records: previous.records.clone(),
        };
        current.delta_from_predecessor =
            catalog_delta_from_touched_records(&current, previous, &HashSet::new());
        Arc::new(current)
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn manifest(&self) -> &ProjectManifest {
        &self.manifest
    }

    pub fn package_assets(&self) -> &PackageAssetRegistry {
        &self.package_assets
    }

    pub fn records(&self) -> impl Iterator<Item = &ProjectCatalogInputRecord> {
        self.records
            .iter()
            .flat_map(|shard| shard.values().map(Arc::as_ref))
    }

    pub fn record(&self, id: AssetId) -> Option<&ProjectCatalogInputRecord> {
        self.records[catalog_input_shard_index(&id)]
            .get(&id)
            .map(Arc::as_ref)
    }

    pub fn delta_since(&self, previous: &Self) -> ProjectCatalogInputDelta {
        if self.sequence == previous.sequence {
            return ProjectCatalogInputDelta::default();
        }
        if self.predecessor_sequence == Some(previous.sequence) {
            return self.delta_from_predecessor.clone();
        }
        catalog_delta_from_full_records(self, previous)
    }

    #[cfg(test)]
    pub(crate) fn from_test_records(
        project_root: &Path,
        manifest: ProjectManifest,
        package_assets: PackageAssetRegistry,
        records: impl IntoIterator<Item = ProjectCatalogInputRecord>,
    ) -> Arc<Self> {
        Arc::new(Self {
            sequence: next_sequence(),
            predecessor_sequence: None,
            delta_from_predecessor: ProjectCatalogInputDelta::default(),
            project_root: project_root.to_path_buf(),
            manifest,
            package_assets,
            records: catalog_input_shards_from_records(records),
        })
    }
}

fn empty_catalog_input_shards() -> Vec<Arc<ProjectCatalogInputShard>> {
    (0..PROJECT_CATALOG_INPUT_SHARD_COUNT)
        .map(|_| Arc::new(ProjectCatalogInputShard::new()))
        .collect()
}

fn catalog_input_shards_from_full_scan(
    records: impl IntoIterator<Item = ResourceRecord>,
    mut sources: HashMap<AssetId, ProjectCatalogInputSource>,
) -> Vec<Arc<ProjectCatalogInputShard>> {
    let mut shards = empty_catalog_input_shards();
    for resource in records {
        if resource.primary_locator.label().is_some() {
            continue;
        }
        let id = resource.id;
        let Some(source) = sources.remove(&id) else {
            continue;
        };
        Arc::make_mut(&mut shards[catalog_input_shard_index(&id)]).insert(
            id,
            Arc::new(ProjectCatalogInputRecord::from_source(resource, source)),
        );
    }
    shards
}

fn catalog_input_shards_from_records(
    records: impl IntoIterator<Item = ProjectCatalogInputRecord>,
) -> Vec<Arc<ProjectCatalogInputShard>> {
    let mut shards = empty_catalog_input_shards();
    for record in records {
        let id = record.resource.id;
        Arc::make_mut(&mut shards[catalog_input_shard_index(&id)]).insert(id, Arc::new(record));
    }
    shards
}

fn catalog_input_shard_index(id: &AssetId) -> usize {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    (hasher.finish() as usize) % PROJECT_CATALOG_INPUT_SHARD_COUNT
}

fn catalog_delta_from_full_records(
    current: &ProjectCatalogInputGeneration,
    previous: &ProjectCatalogInputGeneration,
) -> ProjectCatalogInputDelta {
    if current.project_root != previous.project_root {
        let mut delta = ProjectCatalogInputDelta {
            project_metadata_changed: true,
            added: current.records().cloned().collect(),
            removed: previous.records().cloned().collect(),
            ..ProjectCatalogInputDelta::default()
        };
        sort_delta(&mut delta);
        return delta;
    }

    let mut delta = ProjectCatalogInputDelta {
        project_metadata_changed: catalog_project_metadata_changed(current, previous),
        ..ProjectCatalogInputDelta::default()
    };
    for current_record in current.records() {
        append_catalog_record_delta(
            &mut delta,
            previous.record(current_record.resource.id),
            Some(current_record),
        );
    }
    for previous_record in previous.records() {
        if current.record(previous_record.resource.id).is_none() {
            append_catalog_record_delta(&mut delta, Some(previous_record), None);
        }
    }
    sort_delta(&mut delta);
    delta
}

fn catalog_delta_from_touched_records(
    current: &ProjectCatalogInputGeneration,
    previous: &ProjectCatalogInputGeneration,
    touched_ids: &HashSet<AssetId>,
) -> ProjectCatalogInputDelta {
    if current.project_root != previous.project_root {
        return catalog_delta_from_full_records(current, previous);
    }

    let mut delta = ProjectCatalogInputDelta {
        project_metadata_changed: catalog_project_metadata_changed(current, previous),
        ..ProjectCatalogInputDelta::default()
    };
    for id in touched_ids {
        append_catalog_record_delta(&mut delta, previous.record(*id), current.record(*id));
    }
    sort_delta(&mut delta);
    delta
}

fn catalog_project_metadata_changed(
    current: &ProjectCatalogInputGeneration,
    previous: &ProjectCatalogInputGeneration,
) -> bool {
    current.manifest != previous.manifest || current.package_assets != previous.package_assets
}

fn append_catalog_record_delta(
    delta: &mut ProjectCatalogInputDelta,
    previous: Option<&ProjectCatalogInputRecord>,
    current: Option<&ProjectCatalogInputRecord>,
) {
    match (previous, current) {
        (None, Some(current)) => delta.added.push(current.clone()),
        (Some(previous), None) => delta.removed.push(previous.clone()),
        (Some(previous), Some(current))
            if current.resource.primary_locator != previous.resource.primary_locator =>
        {
            delta.renamed.push(ProjectCatalogInputRename {
                previous_locator: previous.resource.primary_locator.clone(),
                current_locator: current.resource.primary_locator.clone(),
                previous: previous.clone(),
                current: current.clone(),
            });
        }
        (Some(previous), Some(current)) if current != previous => {
            delta.modified.push(current.clone());
        }
        _ => {}
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectCatalogInputRename {
    pub previous_locator: ResourceLocator,
    pub current_locator: ResourceLocator,
    pub previous: ProjectCatalogInputRecord,
    pub current: ProjectCatalogInputRecord,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectCatalogInputDelta {
    pub project_metadata_changed: bool,
    pub added: Vec<ProjectCatalogInputRecord>,
    pub modified: Vec<ProjectCatalogInputRecord>,
    pub removed: Vec<ProjectCatalogInputRecord>,
    pub renamed: Vec<ProjectCatalogInputRename>,
}

impl ProjectCatalogInputDelta {
    pub fn is_unchanged(&self) -> bool {
        !self.project_metadata_changed
            && self.added.is_empty()
            && self.modified.is_empty()
            && self.removed.is_empty()
            && self.renamed.is_empty()
    }
}

fn sort_delta(delta: &mut ProjectCatalogInputDelta) {
    delta.added.sort_by_key(record_locator);
    delta.modified.sort_by_key(record_locator);
    delta.removed.sort_by_key(record_locator);
    delta
        .renamed
        .sort_by_key(|rename| rename.current_locator.to_string());
}

fn record_locator(record: &ProjectCatalogInputRecord) -> String {
    record.resource.primary_locator.to_string()
}

fn next_sequence() -> u64 {
    advance_sequence(&NEXT_CATALOG_INPUT_GENERATION)
}

fn advance_sequence(sequence: &AtomicU64) -> u64 {
    sequence
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .expect("project catalog input generation sequence exhausted")
}

#[cfg(test)]
mod sequence_tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::advance_sequence;

    #[test]
    fn final_catalog_input_sequence_is_published_once_without_wrapping() {
        let sequence = AtomicU64::new(u64::MAX - 1);

        assert_eq!(advance_sequence(&sequence), u64::MAX - 1);
        assert_eq!(sequence.load(Ordering::Relaxed), u64::MAX);
    }

    #[test]
    #[should_panic(expected = "project catalog input generation sequence exhausted")]
    fn exhausted_catalog_input_sequence_never_reuses_an_old_generation() {
        let sequence = AtomicU64::new(u64::MAX);

        let _ = advance_sequence(&sequence);
    }
}
