use std::collections::HashMap;

use crate::{ResourceId, ResourceLocator, ResourceRecord};

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
    ) -> Result<ResourceRecord, String> {
        let Some(id) = self.id_by_locator.remove(from) else {
            return Err(format!("missing resource record for {from}"));
        };
        let Some(record) = self.by_id.get_mut(&id) else {
            return Err(format!("missing resource record for id {id}"));
        };
        record.primary_locator = to.clone();
        self.id_by_locator.insert(to, id);
        Ok(record.clone())
    }

    pub fn remove_by_locator(&mut self, locator: &ResourceLocator) -> Option<ResourceRecord> {
        let id = self.id_by_locator.remove(locator)?;
        self.by_id.remove(&id)
    }
}
