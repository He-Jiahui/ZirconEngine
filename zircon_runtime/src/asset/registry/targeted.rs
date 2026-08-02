use std::collections::{HashMap, HashSet};

use crate::asset::project::{AssetMetaDocument, AssetMetaEntry};
use crate::asset::{AssetId, AssetUri, AssetUuid};

use super::asset_registry_index::source_locator;
use super::rebuild::registry_entries;
use super::{AssetRegistryDiagnostic, AssetRegistryEntry, AssetRegistryError, AssetRegistryIndex};

impl AssetRegistryIndex {
    pub(crate) fn prepare_source_removal(&self, source: &AssetUri) -> (Self, HashSet<AssetUuid>) {
        let source = source_locator(source);
        let removed_paths = self
            .source_entries(&source)
            .into_iter()
            .map(|entry| entry.path().clone())
            .collect::<HashSet<_>>();
        let affected_owners = removed_paths
            .iter()
            .filter_map(|path| self.referencers_by_path.get(path))
            .flatten()
            .copied()
            .collect::<HashSet<_>>();
        let mut candidate = self.clone();
        candidate.remove_source_path(&source);
        candidate.refresh_dependency_owners(&affected_owners);
        (candidate, affected_owners)
    }

    pub(crate) fn prepare_source_replacement(
        &self,
        meta: &mut AssetMetaDocument,
    ) -> Result<Self, AssetRegistryError> {
        Ok(self.prepare_source_replacement_generation(meta)?.0)
    }

    pub(crate) fn prepare_source_replacement_generation(
        &self,
        meta: &mut AssetMetaDocument,
    ) -> Result<(Self, HashSet<AssetUuid>), AssetRegistryError> {
        let source = source_locator(&meta.url);
        let identity_diagnostics = self.normalize_source_identities(meta, &source);
        let entries = registry_entries(meta);
        self.preflight_source_paths(&source, &entries)?;

        let mut affected_paths = self
            .entry_uuids_by_source
            .get(&source)
            .into_iter()
            .flatten()
            .filter_map(|uuid| self.entries_by_uuid.get(uuid))
            .map(|entry| entry.path().clone())
            .collect::<HashSet<_>>();
        affected_paths.extend(entries.iter().map(|entry| entry.path().clone()));
        let mut affected_owners = affected_paths
            .iter()
            .filter_map(|path| self.referencers_by_path.get(path))
            .flatten()
            .copied()
            .collect::<HashSet<_>>();

        let mut candidate = self.clone();
        candidate.remove_source_path(&source);
        for entry in entries {
            affected_owners.insert(entry.uuid());
            candidate.insert_checked(entry)?;
        }
        for (uuid, paths) in dependency_paths(meta) {
            candidate.replace_dependency_paths(uuid, paths);
        }
        candidate.refresh_dependency_owners(&affected_owners);
        candidate.diagnostics.extend(identity_diagnostics);
        Ok((candidate, affected_owners))
    }

    pub(crate) fn source_entries(&self, locator: &AssetUri) -> Vec<AssetRegistryEntry> {
        self.entry_uuids_by_source
            .get(&source_locator(locator))
            .into_iter()
            .flatten()
            .filter_map(|uuid| self.entries_by_uuid.get(uuid))
            .cloned()
            .collect()
    }

    pub(crate) fn retarget_runtime_dependency_paths(
        &mut self,
        changes: impl IntoIterator<Item = (AssetId, Vec<AssetUri>, Vec<AssetUri>)>,
    ) -> HashSet<AssetUuid> {
        let mut owners = HashSet::new();
        for (id, removed, added) in changes {
            let Some(owner) = self.uuid_by_asset_id.get(&id).copied() else {
                continue;
            };
            let mut paths = self
                .dependency_paths_by_uuid
                .get(&owner)
                .cloned()
                .unwrap_or_default();
            for path in removed {
                if let Some(index) = paths.iter().position(|candidate| candidate == &path) {
                    paths.remove(index);
                }
            }
            for path in added {
                paths.push(path);
            }
            self.replace_dependency_paths(owner, paths);
            owners.insert(owner);
        }
        self.refresh_dependency_owners(&owners);
        owners
    }

    fn normalize_source_identities(
        &self,
        meta: &mut AssetMetaDocument,
        source: &AssetUri,
    ) -> Vec<AssetRegistryDiagnostic> {
        let mut owners = HashMap::new();
        let mut diagnostics = Vec::new();
        let original_root = meta.uuid;
        if let Some(first_path) = self.identity_owner(&owners, source, original_root) {
            let replacement = self.unique_uuid(&owners);
            diagnostics.push(AssetRegistryDiagnostic::DuplicateGuidReminted {
                original: original_root,
                first_path,
                path: meta.url.clone(),
                replacement,
            });
            meta.uuid = replacement;
            for entry in &mut meta.entries {
                if entry.url.label().is_none() && entry.uuid == original_root {
                    entry.uuid = replacement;
                }
            }
        }
        owners.insert(meta.uuid, meta.url.clone());

        for entry in &mut meta.entries {
            if entry.url.label().is_none() {
                entry.uuid = meta.uuid;
                continue;
            }
            let Some(first_path) = self.identity_owner(&owners, source, entry.uuid) else {
                owners.insert(entry.uuid, entry.url.clone());
                continue;
            };
            let original = entry.uuid;
            let replacement = self.unique_uuid(&owners);
            diagnostics.push(AssetRegistryDiagnostic::DuplicateGuidReminted {
                original,
                first_path,
                path: entry.url.clone(),
                replacement,
            });
            entry.uuid = replacement;
            owners.insert(replacement, entry.url.clone());
        }
        diagnostics
    }

    fn identity_owner(
        &self,
        prepared: &HashMap<AssetUuid, AssetUri>,
        source: &AssetUri,
        uuid: AssetUuid,
    ) -> Option<AssetUri> {
        prepared.get(&uuid).cloned().or_else(|| {
            self.entries_by_uuid
                .get(&uuid)
                .filter(|entry| source_locator(entry.path()) != *source)
                .map(|entry| entry.path().clone())
        })
    }

    fn unique_uuid(&self, prepared: &HashMap<AssetUuid, AssetUri>) -> AssetUuid {
        loop {
            let candidate = AssetUuid::new();
            if !prepared.contains_key(&candidate) && !self.entries_by_uuid.contains_key(&candidate)
            {
                return candidate;
            }
        }
    }

    fn preflight_source_paths(
        &self,
        source: &AssetUri,
        entries: &[AssetRegistryEntry],
    ) -> Result<(), AssetRegistryError> {
        let mut paths = HashMap::with_capacity(entries.len());
        for entry in entries {
            if let Some(first) = paths.insert(entry.path().clone(), entry.uuid()) {
                return Err(AssetRegistryError::DuplicatePath {
                    path: entry.path().clone(),
                    first,
                    second: entry.uuid(),
                });
            }
            if let Some(existing) = self.entry_by_path(entry.path()) {
                if source_locator(existing.path()) != *source {
                    return Err(AssetRegistryError::DuplicatePath {
                        path: entry.path().clone(),
                        first: existing.uuid(),
                        second: entry.uuid(),
                    });
                }
            }
        }
        Ok(())
    }

    pub(super) fn refresh_dependency_owners(&mut self, owners: &HashSet<AssetUuid>) {
        let mut unresolved = Vec::new();
        let resolved = owners
            .iter()
            .filter(|owner| self.entries_by_uuid.contains_key(owner))
            .map(|owner| {
                let mut dependencies = Vec::new();
                for path in self
                    .dependency_paths_by_uuid
                    .get(owner)
                    .into_iter()
                    .flatten()
                {
                    if let Some(dependency) = self.uuids_by_path.get(path).copied() {
                        if !dependencies.contains(&dependency) {
                            dependencies.push(dependency);
                        }
                    } else {
                        unresolved.push(AssetRegistryDiagnostic::UnresolvedDependency {
                            owner: *owner,
                            path: path.clone(),
                        });
                    }
                }
                (*owner, dependencies)
            })
            .collect::<Vec<_>>();
        for (owner, dependencies) in resolved {
            self.replace_dependencies(owner, dependencies);
        }
        self.diagnostics.retain(|diagnostic| {
            !matches!(
                diagnostic,
                AssetRegistryDiagnostic::UnresolvedDependency { owner, .. }
                    if owners.contains(owner)
            )
        });
        self.diagnostics.extend(unresolved);
    }
}

fn dependency_paths(meta: &AssetMetaDocument) -> Vec<(AssetUuid, Vec<AssetUri>)> {
    let mut dependencies = Vec::new();
    if !meta.entries.iter().any(|entry| entry.url.label().is_none()) {
        dependencies.push((meta.uuid, meta.dependencies.clone()));
    }
    dependencies.extend(
        meta.entries
            .iter()
            .map(|entry: &AssetMetaEntry| (entry.uuid, entry.dependencies.clone())),
    );
    dependencies
}
