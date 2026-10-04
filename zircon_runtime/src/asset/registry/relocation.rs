use std::collections::{HashMap, HashSet};

use crate::asset::project::AssetMetaDocument;
use crate::asset::{AssetUri, AssetUuid};

use super::asset_registry_index::source_locator;
use super::rebuild::registry_entries;
use super::{AssetRegistryEntry, AssetRegistryError, AssetRegistryIndex};

impl AssetRegistryIndex {
    /// Builds a relocation candidate without reminting the source's stable asset identities.
    /// 只有根与子资产 UUID、标签、kind、digest 一一对应时才允许替换 locator。
    ///
    /// The caller must persist the matching source/meta move and rewrite the returned dependent
    /// closure before publishing this candidate.
    pub(crate) fn prepare_source_relocation_generation(
        &self,
        source: &AssetUri,
        relocated_meta: &AssetMetaDocument,
    ) -> Result<(Self, HashSet<AssetUuid>), AssetRegistryError> {
        let source = source_locator(source);
        let target = source_locator(&relocated_meta.url);
        if source == target {
            return Err(relocation_identity_mismatch(
                &source,
                &target,
                "source and target locators are identical",
            ));
        }

        let previous_entries = self.source_entries(&source);
        if previous_entries.is_empty() {
            return Err(AssetRegistryError::AssetPathNotFound { path: source });
        }
        let relocated_entries = registry_entries(relocated_meta);
        let previous_by_uuid = entries_by_uuid(&previous_entries, &source, &target)?;
        let relocated_by_uuid = entries_by_uuid(&relocated_entries, &source, &target)?;
        validate_relocated_entry_set(&source, &target, &previous_by_uuid, &relocated_by_uuid)?;
        preflight_target_paths(self, &source, &relocated_entries)?;

        let mut locator_map = HashMap::with_capacity(previous_by_uuid.len());
        for (uuid, previous) in &previous_by_uuid {
            let relocated = relocated_by_uuid.get(uuid).ok_or_else(|| {
                relocation_identity_mismatch(
                    &source,
                    &target,
                    format!("entry UUID {uuid} is absent from the relocated metadata"),
                )
            })?;
            locator_map.insert(previous.path().clone(), relocated.path().clone());
        }
        let mut affected_owners = previous_by_uuid.keys().copied().collect::<HashSet<_>>();
        for previous in previous_by_uuid.values() {
            affected_owners.extend(
                self.referencers_by_path
                    .get(previous.path())
                    .into_iter()
                    .flatten()
                    .copied(),
            );
        }

        let relocated_dependency_paths = dependency_paths(relocated_meta);
        // 候选索引先删除旧 source，再插入新条目并改写受影响 owner 的依赖路径；完成后统一刷新反向边。
        let mut candidate = self.clone();
        candidate.remove_source_path(&source);
        for entry in relocated_entries {
            candidate.insert_checked(entry)?;
        }
        for owner in &affected_owners {
            let paths = relocated_dependency_paths
                .get(owner)
                .cloned()
                .unwrap_or_else(|| {
                    self.dependency_paths_by_uuid
                        .get(owner)
                        .cloned()
                        .unwrap_or_default()
                })
                .into_iter()
                .map(|path| locator_map.get(&path).cloned().unwrap_or(path))
                .collect();
            candidate.replace_dependency_paths(*owner, paths);
        }
        candidate.refresh_dependency_owners(&affected_owners);
        Ok((candidate, affected_owners))
    }
}

fn entries_by_uuid<'entry>(
    entries: &'entry [AssetRegistryEntry],
    source: &AssetUri,
    target: &AssetUri,
) -> Result<HashMap<AssetUuid, &'entry AssetRegistryEntry>, AssetRegistryError> {
    let mut by_uuid = HashMap::with_capacity(entries.len());
    for entry in entries {
        if by_uuid.insert(entry.uuid(), entry).is_some() {
            return Err(relocation_identity_mismatch(
                source,
                target,
                format!("entry UUID {} is duplicated", entry.uuid()),
            ));
        }
    }
    Ok(by_uuid)
}

fn validate_relocated_entry_set(
    source: &AssetUri,
    target: &AssetUri,
    previous: &HashMap<AssetUuid, &AssetRegistryEntry>,
    relocated: &HashMap<AssetUuid, &AssetRegistryEntry>,
) -> Result<(), AssetRegistryError> {
    if previous.len() != relocated.len() {
        return Err(relocation_identity_mismatch(
            source,
            target,
            "root/subasset entry count changed",
        ));
    }
    for (uuid, previous_entry) in previous {
        let Some(relocated_entry) = relocated.get(uuid) else {
            return Err(relocation_identity_mismatch(
                source,
                target,
                format!("entry UUID {uuid} is absent from the relocated metadata"),
            ));
        };
        if source_locator(relocated_entry.path()) != target.clone() {
            return Err(relocation_identity_mismatch(
                source,
                target,
                format!("entry UUID {uuid} does not belong to the target source"),
            ));
        }
        if previous_entry.path().label() != relocated_entry.path().label() {
            return Err(relocation_identity_mismatch(
                source,
                target,
                format!("entry UUID {uuid} changed its subasset label"),
            ));
        }
        if previous_entry.type_marker() != relocated_entry.type_marker() {
            return Err(relocation_identity_mismatch(
                source,
                target,
                format!("entry UUID {uuid} changed its asset kind"),
            ));
        }
        if previous_entry.source_digest() != relocated_entry.source_digest() {
            return Err(relocation_identity_mismatch(
                source,
                target,
                format!("entry UUID {uuid} changed its source digest"),
            ));
        }
    }
    Ok(())
}

fn preflight_target_paths(
    index: &AssetRegistryIndex,
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
        if let Some(existing) = index.entry_by_path(entry.path()) {
            if source_locator(existing.path()) != source.clone() {
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

fn dependency_paths(meta: &AssetMetaDocument) -> HashMap<AssetUuid, Vec<AssetUri>> {
    let mut paths = HashMap::with_capacity(meta.entries.len().max(1));
    if !meta.entries.iter().any(|entry| entry.url.label().is_none()) {
        paths.insert(meta.uuid, meta.dependencies.clone());
    }
    paths.extend(
        meta.entries
            .iter()
            .map(|entry| (entry.uuid, entry.dependencies.clone())),
    );
    paths
}

fn relocation_identity_mismatch(
    source: &AssetUri,
    target: &AssetUri,
    reason: impl Into<String>,
) -> AssetRegistryError {
    AssetRegistryError::SourceRelocationIdentityMismatch {
        source_uri: source.clone(),
        target: target.clone(),
        reason: reason.into(),
    }
}

#[cfg(test)]
#[path = "tests/relocation.rs"]
mod tests;
