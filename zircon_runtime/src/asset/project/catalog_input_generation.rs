use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::asset::{AssetId, AssetReference, AssetUri};
use crate::core::resource::{ResourceLocator, ResourceRecord};

use super::{AssetMetaDocument, PackageAssetRegistry, ProjectManifest};

static NEXT_CATALOG_INPUT_GENERATION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectCatalogInputRecord {
    resource: ResourceRecord,
    source_path: PathBuf,
    meta_path: PathBuf,
    meta: AssetMetaDocument,
    source_mtime_unix_ms: u64,
    artifact_reference_revision: u64,
    direct_references: Arc<[AssetReference]>,
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
}

impl ProjectCatalogInputSource {
    pub(crate) fn new(
        source_path: PathBuf,
        meta_path: PathBuf,
        meta: AssetMetaDocument,
        source_mtime_unix_ms: u64,
        direct_references: Vec<AssetReference>,
    ) -> Self {
        Self {
            source_path,
            meta_path,
            meta,
            source_mtime_unix_ms,
            direct_references,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProjectCatalogInputGeneration {
    sequence: u64,
    project_root: PathBuf,
    manifest: ProjectManifest,
    package_assets: PackageAssetRegistry,
    records: HashMap<AssetId, ProjectCatalogInputRecord>,
}

impl ProjectCatalogInputGeneration {
    pub(crate) fn initial(
        project_root: &Path,
        manifest: ProjectManifest,
        package_assets: PackageAssetRegistry,
    ) -> Arc<Self> {
        Arc::new(Self {
            sequence: next_sequence(),
            project_root: project_root.to_path_buf(),
            manifest,
            package_assets,
            records: HashMap::new(),
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
        let records = records
            .into_iter()
            .filter(|record| record.primary_locator.label().is_none())
            .filter_map(|record| {
                let source = sources.get(&record.id).cloned()?;
                Some((
                    record.id,
                    ProjectCatalogInputRecord::from_source(record, source),
                ))
            })
            .collect::<HashMap<_, _>>();
        if previous.project_root == project_root
            && previous.manifest == *manifest
            && previous.package_assets == *package_assets
            && previous.records == records
        {
            return Arc::clone(previous);
        }
        Arc::new(Self {
            sequence: next_sequence(),
            project_root: project_root.to_path_buf(),
            manifest: manifest.clone(),
            package_assets: package_assets.clone(),
            records,
        })
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
        Arc::new(Self {
            sequence: next_sequence(),
            project_root: project_root.to_path_buf(),
            manifest: manifest.clone(),
            package_assets: package_assets.clone(),
            records: previous.records.clone(),
        })
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
        self.records.values()
    }

    pub fn record(&self, id: AssetId) -> Option<&ProjectCatalogInputRecord> {
        self.records.get(&id)
    }

    pub fn delta_since(&self, previous: &Self) -> ProjectCatalogInputDelta {
        if self.sequence == previous.sequence {
            return ProjectCatalogInputDelta::default();
        }
        if self.project_root != previous.project_root {
            let mut delta = ProjectCatalogInputDelta {
                project_metadata_changed: true,
                added: self.records.values().cloned().collect(),
                removed: previous.records.values().cloned().collect(),
                ..ProjectCatalogInputDelta::default()
            };
            sort_delta(&mut delta);
            return delta;
        }

        let mut delta = ProjectCatalogInputDelta {
            project_metadata_changed: self.manifest != previous.manifest
                || self.package_assets != previous.package_assets,
            ..ProjectCatalogInputDelta::default()
        };
        for (id, current) in &self.records {
            let Some(previous_record) = previous.records.get(id) else {
                delta.added.push(current.clone());
                continue;
            };
            if current.resource.primary_locator != previous_record.resource.primary_locator {
                delta.renamed.push(ProjectCatalogInputRename {
                    previous_locator: previous_record.resource.primary_locator.clone(),
                    current_locator: current.resource.primary_locator.clone(),
                    previous: previous_record.clone(),
                    current: current.clone(),
                });
            } else if current != previous_record {
                delta.modified.push(current.clone());
            }
        }
        for (id, previous_record) in &previous.records {
            if !self.records.contains_key(id) {
                delta.removed.push(previous_record.clone());
            }
        }
        sort_delta(&mut delta);
        delta
    }

    pub(crate) fn input_sources(&self) -> HashMap<AssetId, ProjectCatalogInputSource> {
        self.records
            .iter()
            .map(|(id, record)| {
                (
                    *id,
                    ProjectCatalogInputSource::new(
                        record.source_path.clone(),
                        record.meta_path.clone(),
                        record.meta.clone(),
                        record.source_mtime_unix_ms,
                        record.direct_references.to_vec(),
                    ),
                )
            })
            .collect()
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
            project_root: project_root.to_path_buf(),
            manifest,
            package_assets,
            records: records
                .into_iter()
                .map(|record| (record.resource.id, record))
                .collect(),
        })
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
    NEXT_CATALOG_INPUT_GENERATION.fetch_add(1, Ordering::Relaxed)
}
