use std::collections::{BTreeMap, HashMap, HashSet};

use crate::asset::{AssetId, AssetKind, AssetUri, AssetUuid};
use crate::core::resource::ResourceRegistry;

use super::{AssetRegistryDiagnostic, AssetRegistryEntry, AssetRegistryError};

/// Authoritative project registry; all query data is memory-resident metadata.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetRegistryIndex {
    pub(super) entries_by_uuid: HashMap<AssetUuid, AssetRegistryEntry>,
    pub(super) uuids_by_path: HashMap<AssetUri, AssetUuid>,
    uuids_by_canonical_path: BTreeMap<AssetUri, AssetUuid>,
    pub(super) uuids_by_type: HashMap<AssetKind, HashSet<AssetUuid>>,
    /// Lookup-only postings; `entries_by_uuid` remains the row authority.
    pub(super) uuids_by_tag: HashMap<String, HashSet<AssetUuid>>,
    pub(super) uuids_by_package: HashMap<String, HashSet<AssetUuid>>,
    /// Exact path buckets are range-scanned to preserve `starts_with` filter semantics.
    pub(super) uuids_by_path_prefix: BTreeMap<String, HashSet<AssetUuid>>,
    pub(super) uuid_by_asset_id: HashMap<AssetId, AssetUuid>,
    pub(super) referencers_by_uuid: HashMap<AssetUuid, HashSet<AssetUuid>>,
    pub(super) entry_uuids_by_source: HashMap<AssetUri, HashSet<AssetUuid>>,
    pub(super) dependency_paths_by_uuid: HashMap<AssetUuid, Vec<AssetUri>>,
    pub(super) referencers_by_path: HashMap<AssetUri, HashSet<AssetUuid>>,
    pub(super) diagnostics: Vec<AssetRegistryDiagnostic>,
}

impl AssetRegistryIndex {
    pub fn from_entries(
        entries: impl IntoIterator<Item = AssetRegistryEntry>,
    ) -> Result<Self, AssetRegistryError> {
        let entries = entries.into_iter();
        let mut index = Self::default();
        index.reserve_build_capacity(entries.size_hint().0);
        for entry in entries {
            index.insert_checked(entry)?;
        }
        let entry_uuids = index.entries_by_uuid.keys().copied().collect::<Vec<_>>();
        for uuid in entry_uuids {
            let paths = index
                .entries_by_uuid
                .get(&uuid)
                .map(|entry| {
                    entry
                        .dependencies()
                        .iter()
                        .filter_map(|dependency_uuid| index.entries_by_uuid.get(dependency_uuid))
                        .map(|dependency| dependency.path().clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            index.replace_dependency_paths_inner(uuid, paths, false);
        }
        index
            .referencers_by_path
            .retain(|_, referencers| !referencers.is_empty());
        Ok(index)
    }

    fn reserve_build_capacity(&mut self, entry_count: usize) {
        self.entries_by_uuid.reserve(entry_count);
        self.uuids_by_path.reserve(entry_count);
        self.uuid_by_asset_id.reserve(entry_count);
        self.entry_uuids_by_source.reserve(entry_count);
    }

    pub(crate) fn reconcile_resource_dependencies(
        &mut self,
        registry: &ResourceRegistry,
        dependency_paths_by_id: &HashMap<AssetId, Vec<AssetUri>>,
    ) {
        let dependencies = registry
            .values()
            .filter_map(|record| {
                let owner = self.uuid_by_asset_id.get(&record.id()).copied()?;
                let dependency_paths = dependency_paths_by_id
                    .get(&record.id())
                    .cloned()
                    .unwrap_or_default();
                Some((owner, dependency_paths))
            })
            .collect::<Vec<_>>();
        let owners = dependencies
            .iter()
            .map(|(owner, _)| *owner)
            .collect::<HashSet<_>>();
        for (owner, dependency_paths) in dependencies {
            self.replace_dependency_paths(owner, dependency_paths);
        }
        self.refresh_dependency_owners(&owners);
    }

    pub fn len(&self) -> usize {
        self.entries_by_uuid.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries_by_uuid.is_empty()
    }

    pub fn entries(&self) -> Vec<&AssetRegistryEntry> {
        self.entries_iter().collect()
    }

    pub fn entries_iter(
        &self,
    ) -> impl ExactSizeIterator<Item = &AssetRegistryEntry> + DoubleEndedIterator {
        self.uuids_by_canonical_path.values().map(|uuid| {
            self.entries_by_uuid
                .get(uuid)
                .expect("canonical asset path index must reference a live entry")
        })
    }

    pub fn diagnostics(&self) -> &[AssetRegistryDiagnostic] {
        &self.diagnostics
    }

    pub fn entry_by_uuid(&self, uuid: AssetUuid) -> Option<&AssetRegistryEntry> {
        self.entries_by_uuid.get(&uuid)
    }

    pub fn entry_by_path(&self, path: &AssetUri) -> Option<&AssetRegistryEntry> {
        self.uuids_by_path
            .get(path)
            .and_then(|uuid| self.entries_by_uuid.get(uuid))
    }

    pub(super) fn insert_checked(
        &mut self,
        entry: AssetRegistryEntry,
    ) -> Result<(), AssetRegistryError> {
        if let Some(previous) = self.entries_by_uuid.get(&entry.uuid()) {
            return Err(AssetRegistryError::DuplicateUuid {
                uuid: entry.uuid(),
                first: previous.path().clone(),
                second: entry.path().clone(),
            });
        }
        if let Some(previous) = self.uuids_by_path.get(entry.path()) {
            return Err(AssetRegistryError::DuplicatePath {
                path: entry.path().clone(),
                first: *previous,
                second: entry.uuid(),
            });
        }
        self.uuids_by_path
            .insert(entry.path().clone(), entry.uuid());
        self.uuids_by_canonical_path
            .insert(entry.path().clone(), entry.uuid());
        self.uuids_by_type
            .entry(entry.type_marker())
            .or_default()
            .insert(entry.uuid());
        for tag in entry.tags() {
            self.uuids_by_tag
                .entry(tag.clone())
                .or_default()
                .insert(entry.uuid());
        }
        if let Some(package_id) = entry.path().package_id() {
            self.uuids_by_package
                .entry(package_id.to_owned())
                .or_default()
                .insert(entry.uuid());
        }
        self.uuids_by_path_prefix
            .entry(entry.path().path().to_owned())
            .or_default()
            .insert(entry.uuid());
        self.uuid_by_asset_id
            .insert(AssetId::from_asset_uuid(entry.uuid()), entry.uuid());
        self.entry_uuids_by_source
            .entry(source_locator(entry.path()))
            .or_default()
            .insert(entry.uuid());
        for dependency in entry.dependencies() {
            self.referencers_by_uuid
                .entry(*dependency)
                .or_default()
                .insert(entry.uuid());
        }
        self.entries_by_uuid.insert(entry.uuid(), entry);
        Ok(())
    }

    pub(super) fn remove_source_path(&mut self, path: &AssetUri) {
        let removed = self
            .entry_uuids_by_source
            .remove(&source_locator(path))
            .unwrap_or_default();
        for uuid in removed {
            if let Some(entry) = self.entries_by_uuid.remove(&uuid) {
                self.uuids_by_path.remove(entry.path());
                self.uuids_by_canonical_path.remove(entry.path());
                let type_marker = entry.type_marker();
                let remove_type_bucket =
                    self.uuids_by_type
                        .get_mut(&type_marker)
                        .is_some_and(|uuids| {
                            uuids.remove(&uuid);
                            uuids.is_empty()
                        });
                if remove_type_bucket {
                    self.uuids_by_type.remove(&type_marker);
                }
                for tag in entry.tags() {
                    let remove_tag_bucket = self.uuids_by_tag.get_mut(tag).is_some_and(|uuids| {
                        uuids.remove(&uuid);
                        uuids.is_empty()
                    });
                    if remove_tag_bucket {
                        self.uuids_by_tag.remove(tag);
                    }
                }
                if let Some(package_id) = entry.path().package_id() {
                    let remove_package_bucket = self
                        .uuids_by_package
                        .get_mut(package_id)
                        .is_some_and(|uuids| {
                            uuids.remove(&uuid);
                            uuids.is_empty()
                        });
                    if remove_package_bucket {
                        self.uuids_by_package.remove(package_id);
                    }
                }
                let path_prefix = entry.path().path().to_owned();
                let remove_path_prefix_bucket = self
                    .uuids_by_path_prefix
                    .get_mut(&path_prefix)
                    .is_some_and(|uuids| {
                        uuids.remove(&uuid);
                        uuids.is_empty()
                    });
                if remove_path_prefix_bucket {
                    self.uuids_by_path_prefix.remove(&path_prefix);
                }
                self.uuid_by_asset_id
                    .remove(&AssetId::from_asset_uuid(uuid));
                for dependency in entry.dependencies() {
                    let remove_bucket =
                        self.referencers_by_uuid
                            .get_mut(dependency)
                            .is_some_and(|referencers| {
                                referencers.remove(&uuid);
                                referencers.is_empty()
                            });
                    if remove_bucket {
                        self.referencers_by_uuid.remove(dependency);
                    }
                }
                self.replace_dependency_paths(uuid, Vec::new());
            }
        }
    }

    pub(super) fn replace_dependency_paths(
        &mut self,
        uuid: AssetUuid,
        dependencies: Vec<AssetUri>,
    ) {
        self.replace_dependency_paths_inner(uuid, dependencies, true);
    }

    fn replace_dependency_paths_inner(
        &mut self,
        uuid: AssetUuid,
        dependencies: Vec<AssetUri>,
        prune_empty_buckets: bool,
    ) {
        let previous = self
            .dependency_paths_by_uuid
            .remove(&uuid)
            .unwrap_or_default();
        for dependency in &previous {
            if let Some(referencers) = self.referencers_by_path.get_mut(dependency) {
                referencers.remove(&uuid);
            }
        }
        for dependency in &dependencies {
            self.referencers_by_path
                .entry(dependency.clone())
                .or_default()
                .insert(uuid);
        }
        if !dependencies.is_empty() {
            self.dependency_paths_by_uuid.insert(uuid, dependencies);
        }
        if prune_empty_buckets {
            // Only old buckets can become empty; reinsert overlapping edges before pruning.
            for dependency in previous {
                if self
                    .referencers_by_path
                    .get(&dependency)
                    .is_some_and(HashSet::is_empty)
                {
                    self.referencers_by_path.remove(&dependency);
                }
            }
        }
    }

    pub(super) fn replace_dependencies(&mut self, uuid: AssetUuid, dependencies: Vec<AssetUuid>) {
        let Some(entry) = self.entries_by_uuid.get_mut(&uuid) else {
            return;
        };
        let previous = entry.dependencies().to_vec();
        entry.set_dependencies(dependencies);
        let current = entry.dependencies().to_vec();

        for dependency in &previous {
            if let Some(referencers) = self.referencers_by_uuid.get_mut(dependency) {
                referencers.remove(&uuid);
            }
        }
        for dependency in current {
            self.referencers_by_uuid
                .entry(dependency)
                .or_default()
                .insert(uuid);
        }
        // Preserve overlapping and shared buckets; only removed edges can empty a bucket.
        for dependency in previous {
            if self
                .referencers_by_uuid
                .get(&dependency)
                .is_some_and(HashSet::is_empty)
            {
                self.referencers_by_uuid.remove(&dependency);
            }
        }
    }

    pub(super) fn push_diagnostic(&mut self, diagnostic: AssetRegistryDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub(crate) fn replace_duplicate_diagnostics(
        &mut self,
        diagnostics: Vec<AssetRegistryDiagnostic>,
    ) {
        self.diagnostics.retain(|diagnostic| {
            !matches!(
                diagnostic,
                AssetRegistryDiagnostic::DuplicateGuidReminted { .. }
            )
        });
        self.diagnostics.extend(diagnostics);
    }
}

pub(super) fn source_locator(locator: &AssetUri) -> AssetUri {
    AssetUri::new(locator.scheme(), locator.path().to_string(), None)
        .expect("a parsed asset URI remains valid when its label is removed")
}

#[cfg(test)]
#[path = "tests/asset_registry_index.rs"]
mod tests;

#[cfg(test)]
#[path = "asset_registry_index/tests/optimization_tests.rs"]
mod optimization_tests;

#[cfg(test)]
#[path = "asset_registry_index/tests/type_posting_tests.rs"]
mod type_posting_tests;

#[cfg(test)]
#[path = "asset_registry_index/tests/secondary_query_tests.rs"]
mod secondary_query_tests;

#[cfg(test)]
#[path = "asset_registry_index/tests/build_tests.rs"]
mod build_tests;

#[cfg(test)]
#[path = "asset_registry_index/tests/incremental_referencer_pruning_tests.rs"]
mod incremental_referencer_pruning_tests;

#[cfg(test)]
#[path = "asset_registry_index/tests/source_removal_pruning_tests.rs"]
mod source_removal_pruning_tests;
