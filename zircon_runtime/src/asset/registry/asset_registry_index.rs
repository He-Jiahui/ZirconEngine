use std::collections::{HashMap, HashSet};

use crate::asset::{AssetId, AssetKind, AssetUri, AssetUuid};
use crate::core::resource::ResourceRegistry;

use super::{AssetRegistryDiagnostic, AssetRegistryEntry, AssetRegistryError};

/// Authoritative project registry; all query data is memory-resident metadata.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetRegistryIndex {
    pub(super) entries_by_uuid: HashMap<AssetUuid, AssetRegistryEntry>,
    pub(super) uuids_by_path: HashMap<AssetUri, AssetUuid>,
    pub(super) uuids_by_type: HashMap<AssetKind, HashSet<AssetUuid>>,
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
        let mut index = Self::default();
        for entry in entries {
            index.insert_checked(entry)?;
        }
        let dependency_paths = index
            .entries_by_uuid
            .values()
            .map(|entry| {
                let paths = entry
                    .dependencies()
                    .iter()
                    .filter_map(|uuid| index.entries_by_uuid.get(uuid))
                    .map(|dependency| dependency.path().clone())
                    .collect::<Vec<_>>();
                (entry.uuid(), paths)
            })
            .collect::<Vec<_>>();
        for (uuid, paths) in dependency_paths {
            index.replace_dependency_paths(uuid, paths);
        }
        Ok(index)
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
        let mut entries = self.entries_by_uuid.values().collect::<Vec<_>>();
        entries.sort_unstable_by(|left, right| left.path().cmp(right.path()));
        entries
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
        self.uuids_by_type
            .entry(entry.type_marker())
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
                let type_marker = entry.type_marker();
                let remove_type_bucket = self
                    .uuids_by_type
                    .get_mut(&type_marker)
                    .is_some_and(|uuids| {
                        uuids.remove(&uuid);
                        uuids.is_empty()
                    });
                if remove_type_bucket {
                    self.uuids_by_type.remove(&type_marker);
                }
                self.uuid_by_asset_id
                    .remove(&AssetId::from_asset_uuid(uuid));
                for dependency in entry.dependencies() {
                    if let Some(referencers) = self.referencers_by_uuid.get_mut(dependency) {
                        referencers.remove(&uuid);
                    }
                }
                self.replace_dependency_paths(uuid, Vec::new());
            }
        }
        self.referencers_by_uuid
            .retain(|_, referencers| !referencers.is_empty());
    }

    pub(super) fn replace_dependency_paths(
        &mut self,
        uuid: AssetUuid,
        dependencies: Vec<AssetUri>,
    ) {
        for dependency in self
            .dependency_paths_by_uuid
            .remove(&uuid)
            .unwrap_or_default()
        {
            if let Some(referencers) = self.referencers_by_path.get_mut(&dependency) {
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
        self.referencers_by_path
            .retain(|_, referencers| !referencers.is_empty());
    }

    pub(super) fn replace_dependencies(&mut self, uuid: AssetUuid, dependencies: Vec<AssetUuid>) {
        let Some(entry) = self.entries_by_uuid.get_mut(&uuid) else {
            return;
        };
        let previous = entry.dependencies().to_vec();
        entry.set_dependencies(dependencies);
        let current = entry.dependencies().to_vec();

        for dependency in previous {
            if let Some(referencers) = self.referencers_by_uuid.get_mut(&dependency) {
                referencers.remove(&uuid);
            }
        }
        for dependency in current {
            self.referencers_by_uuid
                .entry(dependency)
                .or_default()
                .insert(uuid);
        }
        self.referencers_by_uuid
            .retain(|_, referencers| !referencers.is_empty());
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
mod tests {
    use std::collections::HashSet;

    use crate::asset::{AssetKind, AssetUri, AssetUuid};

    use super::{AssetRegistryEntry, AssetRegistryIndex};

    #[test]
    fn persisted_entries_restore_dependency_path_and_referencer_indexes() {
        let dependency_uuid = AssetUuid::new();
        let owner_uuid = AssetUuid::new();
        let dependency_path = AssetUri::parse("res://data/dependency.json").unwrap();
        let owner_path = AssetUri::parse("res://data/owner.json").unwrap();
        let index = AssetRegistryIndex::from_entries([
            AssetRegistryEntry::new(
                dependency_uuid,
                dependency_path.clone(),
                AssetKind::Data,
                "dependency",
            ),
            AssetRegistryEntry::new(owner_uuid, owner_path, AssetKind::Data, "owner")
                .with_dependencies(vec![dependency_uuid]),
        ])
        .unwrap();

        assert_eq!(
            index.get_referencers_by_path(&dependency_path),
            vec![owner_uuid]
        );
    }

    #[test]
    fn overlapping_dependency_path_sources_resolve_to_one_uuid_edge() {
        let dependency_uuid = AssetUuid::new();
        let owner_uuid = AssetUuid::new();
        let dependency_path = AssetUri::parse("res://data/dependency.json").unwrap();
        let mut index = AssetRegistryIndex::from_entries([
            AssetRegistryEntry::new(
                dependency_uuid,
                dependency_path.clone(),
                AssetKind::Data,
                "dependency",
            ),
            AssetRegistryEntry::new(
                owner_uuid,
                AssetUri::parse("res://data/owner.json").unwrap(),
                AssetKind::Data,
                "owner",
            ),
        ])
        .unwrap();
        index.replace_dependency_paths(owner_uuid, vec![dependency_path.clone(), dependency_path]);

        index.refresh_dependency_owners(&HashSet::from([owner_uuid]));

        assert_eq!(
            index.entry_by_uuid(owner_uuid).unwrap().dependencies(),
            &[dependency_uuid]
        );
    }
}

#[cfg(test)]
#[path = "asset_registry_index/optimization_tests.rs"]
mod optimization_tests;

#[cfg(test)]
#[path = "asset_registry_index/type_posting_tests.rs"]
mod type_posting_tests;
