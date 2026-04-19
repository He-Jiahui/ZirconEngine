use crate::core::resource::{
    ResourceDiagnostic, ResourceEvent, ResourceEventKind, ResourceId, ResourceLocator,
    ResourceRecord, RuntimeResourceState, UntypedResourceHandle,
};

use super::resource_manager::ResourceManager;

impl ResourceManager {
    pub fn register_record(&self, record: ResourceRecord) -> UntypedResourceHandle {
        let event_kind = {
            let mut registry = self
                .registry
                .write()
                .expect("resource registry lock poisoned");
            match registry.upsert(record.clone()) {
                Some(_) => ResourceEventKind::Updated,
                None => ResourceEventKind::Added,
            }
        };
        self.ensure_runtime_slot(record.id);

        self.broadcast(ResourceEvent {
            kind: event_kind,
            id: record.id,
            locator: Some(record.primary_locator.clone()),
            previous_locator: None,
            revision: record.revision,
        });

        UntypedResourceHandle::new(record.id, record.kind)
    }

    pub fn start_reload(
        &self,
        id: ResourceId,
        diagnostics: Vec<ResourceDiagnostic>,
    ) -> Option<ResourceRecord> {
        let updated = {
            let mut registry = self
                .registry
                .write()
                .expect("resource registry lock poisoned");
            let mut record = registry.get(id).cloned()?;
            record.state = crate::core::resource::ResourceState::Reloading;
            record.diagnostics = diagnostics;
            registry.upsert(record.clone());
            record
        };
        self.set_runtime_state(id, RuntimeResourceState::Reloading);

        self.broadcast(ResourceEvent {
            kind: ResourceEventKind::Updated,
            id,
            locator: Some(updated.primary_locator.clone()),
            previous_locator: None,
            revision: updated.revision,
        });

        Some(updated)
    }

    pub fn fail_reload(
        &self,
        id: ResourceId,
        diagnostics: Vec<ResourceDiagnostic>,
    ) -> Option<ResourceRecord> {
        let updated = {
            let mut registry = self
                .registry
                .write()
                .expect("resource registry lock poisoned");
            let mut record = registry.get(id).cloned()?;
            record.state = crate::core::resource::ResourceState::Error;
            record.diagnostics = diagnostics;
            registry.upsert(record.clone());
            record
        };
        self.set_runtime_state(id, RuntimeResourceState::Error);

        self.broadcast(ResourceEvent {
            kind: ResourceEventKind::ReloadFailed,
            id,
            locator: Some(updated.primary_locator.clone()),
            previous_locator: None,
            revision: updated.revision,
        });

        Some(updated)
    }

    pub fn remove_by_locator(&self, locator: &ResourceLocator) -> Option<ResourceRecord> {
        let removed = {
            let mut registry = self
                .registry
                .write()
                .expect("resource registry lock poisoned");
            registry.remove_by_locator(locator)?
        };

        self.payloads
            .write()
            .expect("resource payload lock poisoned")
            .remove(&removed.id);
        self.runtime
            .write()
            .expect("resource runtime lock poisoned")
            .remove(&removed.id);

        self.broadcast(ResourceEvent {
            kind: ResourceEventKind::Removed,
            id: removed.id,
            locator: Some(removed.primary_locator.clone()),
            previous_locator: None,
            revision: removed.revision,
        });

        Some(removed)
    }

    pub fn rename(
        &self,
        from: &ResourceLocator,
        to: ResourceLocator,
    ) -> Result<ResourceRecord, String> {
        let renamed = {
            let mut registry = self
                .registry
                .write()
                .expect("resource registry lock poisoned");
            registry.rename(from, to.clone())?
        };

        self.broadcast(ResourceEvent {
            kind: ResourceEventKind::Renamed,
            id: renamed.id,
            locator: Some(renamed.primary_locator.clone()),
            previous_locator: Some(from.clone()),
            revision: renamed.revision,
        });

        Ok(renamed)
    }
}
