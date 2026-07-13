use std::collections::HashMap;

use crate::asset::{AssetUri, AssetUuid};

use super::{AssetRegistryDiagnostic, AssetRegistryEntry, AssetRegistryError};

/// Authoritative project registry; all query data is memory-resident metadata.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetRegistryIndex {
    pub(super) entries_by_uuid: HashMap<AssetUuid, AssetRegistryEntry>,
    pub(super) uuids_by_path: HashMap<AssetUri, AssetUuid>,
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
        Ok(index)
    }

    pub fn len(&self) -> usize {
        self.entries_by_uuid.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries_by_uuid.is_empty()
    }

    pub fn entries(&self) -> Vec<&AssetRegistryEntry> {
        let mut entries = self.entries_by_uuid.values().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.path().cmp(right.path()));
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
        self.entries_by_uuid.insert(entry.uuid(), entry);
        Ok(())
    }

    pub(super) fn remove_source_path(&mut self, path: &AssetUri) {
        let removed = self
            .entries_by_uuid
            .values()
            .filter(|entry| same_source_path(entry.path(), path))
            .map(AssetRegistryEntry::uuid)
            .collect::<Vec<_>>();
        for uuid in removed {
            if let Some(entry) = self.entries_by_uuid.remove(&uuid) {
                self.uuids_by_path.remove(entry.path());
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

fn same_source_path(left: &AssetUri, right: &AssetUri) -> bool {
    left.scheme() == right.scheme() && left.path() == right.path()
}
