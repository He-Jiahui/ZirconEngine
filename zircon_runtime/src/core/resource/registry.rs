use std::collections::HashMap;

use crate::core::resource::{ResourceId, ResourceLocator, ResourceRecord};
use crate::core::{CoreError, CoreResult};

#[derive(Clone, Debug, Default)]
pub struct ResourceRegistry {
    by_id: HashMap<ResourceId, ResourceRecord>,
    id_by_locator: HashMap<ResourceLocator, ResourceId>,
}

impl ResourceRegistry {
    pub fn upsert(&mut self, record: ResourceRecord) -> Option<ResourceRecord> {
        if let Some(existing_id) = self.id_by_locator.get(&record.primary_locator).copied() {
            if existing_id != record.id {
                self.by_id.remove(&existing_id);
            }
        }

        if let Some(existing) = self.by_id.get(&record.id).cloned() {
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

    pub fn values(&self) -> impl Iterator<Item = &ResourceRecord> {
        self.by_id.values()
    }

    pub fn rename(
        &mut self,
        from: &ResourceLocator,
        to: ResourceLocator,
    ) -> CoreResult<ResourceRecord> {
        let Some(id) = self.id_by_locator.get(from).copied() else {
            return Err(CoreError::MissingResourceRecordForLocator {
                locator: from.to_string(),
            });
        };
        let Some(record) = self.by_id.get_mut(&id) else {
            return Err(CoreError::MissingResourceRecordForId { id: id.to_string() });
        };
        record.primary_locator = to.clone();
        self.id_by_locator.remove(from);
        self.id_by_locator.insert(to, id);
        Ok(record.clone())
    }

    pub fn remove_by_locator(&mut self, locator: &ResourceLocator) -> Option<ResourceRecord> {
        let id = self.id_by_locator.remove(locator)?;
        self.by_id.remove(&id)
    }
}
