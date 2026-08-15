use std::collections::HashMap;
use std::ops::Deref;

use crate::core::resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceRecord, ResourceRegistryError,
    ResourceResult,
};

#[derive(Clone, Debug, Default)]
pub struct ResourceRegistry {
    by_id: HashMap<ResourceId, ResourceRecord>,
    id_by_locator: HashMap<ResourceLocator, ResourceId>,
}

impl ResourceRegistry {
    pub(crate) fn insert_unchecked(&mut self, record: ResourceRecord) -> Option<ResourceRecord> {
        debug_assert!(
            self.id_by_locator
                .get(&record.primary_locator)
                .is_none_or(|existing_id| *existing_id == record.id),
            "resource mutation preflight must reject occupied locators"
        );
        if let Some(existing) = self.by_id.get(&record.id) {
            self.id_by_locator.remove(&existing.primary_locator);
        }

        self.id_by_locator
            .insert(record.primary_locator.clone(), record.id);
        self.by_id.insert(record.id, record)
    }

    pub fn get(&self, id: ResourceId) -> Option<&ResourceRecord> {
        self.by_id.get(&id)
    }

    pub fn get_by_locator(&self, locator: &ResourceLocator) -> Option<&ResourceRecord> {
        self.id_by_locator
            .get(locator)
            .and_then(|id| self.by_id.get(id))
    }

    pub(crate) fn id_for_locator(&self, locator: &ResourceLocator) -> Option<ResourceId> {
        self.id_by_locator.get(locator).copied()
    }

    pub fn values(&self) -> impl Iterator<Item = &ResourceRecord> {
        self.by_id.values()
    }

    pub(crate) fn remove_by_id(&mut self, id: ResourceId) -> Option<ResourceRecord> {
        let removed = self.by_id.remove(&id)?;
        self.id_by_locator.remove(&removed.primary_locator);
        Some(removed)
    }

    pub(crate) fn begin_staging(&self) -> ResourceRegistryStaging {
        ResourceRegistryStaging {
            registry: self.clone(),
            identities: self
                .by_id
                .values()
                .map(|record| {
                    (
                        record.id,
                        ResourceRegistryIdentity {
                            kind: record.kind,
                            authorized_locator: record.primary_locator.clone(),
                        },
                    )
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
struct ResourceRegistryIdentity {
    kind: ResourceKind,
    authorized_locator: ResourceLocator,
}

/// Offline catalog builder. Live runtime mutation remains exclusive to ResourceManager::commit.
#[derive(Clone, Debug, Default)]
pub(crate) struct ResourceRegistryStaging {
    registry: ResourceRegistry,
    identities: HashMap<ResourceId, ResourceRegistryIdentity>,
}

impl ResourceRegistryStaging {
    pub(crate) fn stage_record(
        &mut self,
        record: ResourceRecord,
    ) -> ResourceResult<Option<ResourceRecord>> {
        if let Some(existing_id) = self.registry.id_for_locator(&record.primary_locator) {
            if existing_id != record.id {
                return Err(ResourceRegistryError::LocatorOccupied {
                    locator: record.primary_locator.to_string(),
                    existing_id: existing_id.to_string(),
                    requested_id: record.id.to_string(),
                });
            }
        }
        if let Some(identity) = self.identities.get(&record.id) {
            if identity.kind != record.kind {
                return Err(ResourceRegistryError::KindConflict {
                    id: record.id.to_string(),
                    current_kind: identity.kind,
                    requested_kind: record.kind,
                });
            }
            if identity.authorized_locator != record.primary_locator {
                return Err(ResourceRegistryError::ExplicitRenameRequired {
                    id: record.id.to_string(),
                    current_locator: identity.authorized_locator.to_string(),
                    requested_locator: record.primary_locator.to_string(),
                });
            }
        }
        self.identities
            .entry(record.id)
            .or_insert_with(|| ResourceRegistryIdentity {
                kind: record.kind,
                authorized_locator: record.primary_locator.clone(),
            });
        Ok(self.registry.insert_unchecked(record))
    }

    pub(crate) fn stage_remove_locator(
        &mut self,
        locator: &ResourceLocator,
    ) -> Option<ResourceRecord> {
        let id = self.registry.id_for_locator(locator)?;
        self.registry.remove_by_id(id)
    }

    pub(crate) fn finish(self) -> ResourceRegistry {
        self.registry
    }
}

impl Deref for ResourceRegistryStaging {
    type Target = ResourceRegistry;

    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}
