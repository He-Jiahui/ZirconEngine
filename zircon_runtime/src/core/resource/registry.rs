use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use crate::core::resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceRecord, ResourceRegistryError,
    ResourceResult,
};

#[derive(Clone, Debug, Default)]
pub struct ResourceRegistry {
    by_id: Arc<HashMap<ResourceId, Arc<ResourceRecord>>>,
    id_by_locator: Arc<HashMap<Arc<ResourceLocator>, ResourceId>>,
}

impl ResourceRegistry {
    pub(crate) fn insert_unchecked(&mut self, record: ResourceRecord) -> Option<ResourceRecord> {
        debug_assert!(
            self.id_by_locator
                .get(&record.primary_locator)
                .is_none_or(|existing_id| *existing_id == record.id),
            "resource mutation preflight must reject occupied locators"
        );
        if let Some(existing_locator) = self
            .by_id
            .get(&record.id)
            .map(|existing| existing.primary_locator.clone())
        {
            Arc::make_mut(&mut self.id_by_locator).remove(&existing_locator);
        }

        Arc::make_mut(&mut self.id_by_locator)
            .insert(Arc::new(record.primary_locator.clone()), record.id);
        Arc::make_mut(&mut self.by_id)
            .insert(record.id, Arc::new(record))
            .map(into_owned_record)
    }

    pub fn get(&self, id: ResourceId) -> Option<&ResourceRecord> {
        self.by_id.get(&id).map(Arc::as_ref)
    }

    pub fn get_by_locator(&self, locator: &ResourceLocator) -> Option<&ResourceRecord> {
        self.id_by_locator
            .get(locator)
            .and_then(|id| self.by_id.get(id))
            .map(Arc::as_ref)
    }

    pub(crate) fn id_for_locator(&self, locator: &ResourceLocator) -> Option<ResourceId> {
        self.id_by_locator.get(locator).copied()
    }

    pub fn values(&self) -> impl Iterator<Item = &ResourceRecord> {
        self.by_id.values().map(Arc::as_ref)
    }

    pub(crate) fn remove_by_id(&mut self, id: ResourceId) -> Option<ResourceRecord> {
        let removed = Arc::make_mut(&mut self.by_id).remove(&id)?;
        Arc::make_mut(&mut self.id_by_locator).remove(&removed.primary_locator);
        Some(into_owned_record(removed))
    }

    pub(crate) fn begin_staging(&self) -> ResourceRegistryStaging {
        ResourceRegistryStaging {
            registry: self.clone(),
            identity_registry: self.clone(),
            staged_identities: HashMap::new(),
        }
    }
}

fn into_owned_record(record: Arc<ResourceRecord>) -> ResourceRecord {
    Arc::try_unwrap(record).unwrap_or_else(|record| (*record).clone())
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
    identity_registry: ResourceRegistry,
    staged_identities: HashMap<ResourceId, ResourceRegistryIdentity>,
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
        if let Some((kind, authorized_locator)) = self.identity_for(record.id) {
            if kind != record.kind {
                return Err(ResourceRegistryError::KindConflict {
                    id: record.id.to_string(),
                    current_kind: kind,
                    requested_kind: record.kind,
                });
            }
            if authorized_locator != &record.primary_locator {
                return Err(ResourceRegistryError::ExplicitRenameRequired {
                    id: record.id.to_string(),
                    current_locator: authorized_locator.to_string(),
                    requested_locator: record.primary_locator.to_string(),
                });
            }
        }
        if self.identity_registry.get(record.id).is_none() {
            self.staged_identities
                .entry(record.id)
                .or_insert_with(|| ResourceRegistryIdentity {
                    kind: record.kind,
                    authorized_locator: record.primary_locator.clone(),
                });
        }
        Ok(self.registry.insert_unchecked(record))
    }

    fn identity_for(&self, id: ResourceId) -> Option<(ResourceKind, &ResourceLocator)> {
        self.identity_registry
            .get(id)
            .map(|record| (record.kind, &record.primary_locator))
            .or_else(|| {
                self.staged_identities
                    .get(&id)
                    .map(|identity| (identity.kind, &identity.authorized_locator))
            })
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
