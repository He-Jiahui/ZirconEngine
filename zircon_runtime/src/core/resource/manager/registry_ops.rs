use crate::core::resource::{
    ResourceDiagnostic, ResourceEvent, ResourceEventKind, ResourceId, ResourceLocator,
    ResourceRecord, ResourceResult, ResourceState, RuntimeResourceState, UntypedResourceHandle,
};

use super::resource_manager::ResourceManager;

impl ResourceManager {
    pub fn register_record(&self, record: ResourceRecord) -> UntypedResourceHandle {
        let id = record.id;
        let kind = record.kind;
        let revision = record.revision;
        let locator = record.primary_locator.clone();
        let event_kind = {
            let mut registry = self.lock_registry_write();
            match registry.upsert(record) {
                Some(_) => ResourceEventKind::Updated,
                None => ResourceEventKind::Added,
            }
        };
        self.ensure_runtime_slot(id);
        self.refresh_readiness(id);

        self.broadcast(ResourceEvent {
            kind: event_kind,
            resource_kind: kind,
            id,
            locator: Some(locator),
            previous_locator: None,
            revision,
        });

        UntypedResourceHandle::new(id, kind)
    }

    pub fn start_reload(
        &self,
        id: ResourceId,
        diagnostics: Vec<ResourceDiagnostic>,
    ) -> Option<ResourceRecord> {
        let updated = {
            let mut registry = self.lock_registry_write();
            let updated = {
                let record = registry.get_mut(id)?;
                if !matches!(
                    record.state,
                    ResourceState::Ready | ResourceState::Reloading | ResourceState::Error
                ) {
                    return None;
                }
                record.state = crate::core::resource::ResourceState::Reloading;
                record.diagnostics = diagnostics;
                record.clone()
            };
            registry.publish_record(&updated);
            updated
        };
        self.set_runtime_state(id, RuntimeResourceState::Reloading);
        self.refresh_readiness(id);

        self.broadcast(ResourceEvent {
            kind: ResourceEventKind::Updated,
            resource_kind: updated.kind,
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
            let mut registry = self.lock_registry_write();
            let updated = {
                let record = registry.get_mut(id)?;
                if !matches!(
                    record.state,
                    ResourceState::Pending | ResourceState::Reloading | ResourceState::Error
                ) {
                    return None;
                }
                record.state = crate::core::resource::ResourceState::Error;
                record.diagnostics = diagnostics;
                record.clone()
            };
            registry.publish_record(&updated);
            updated
        };
        self.set_runtime_state(id, RuntimeResourceState::Error);
        self.refresh_readiness(id);

        self.broadcast(ResourceEvent {
            kind: ResourceEventKind::ReloadFailed,
            resource_kind: updated.kind,
            id,
            locator: Some(updated.primary_locator.clone()),
            previous_locator: None,
            revision: updated.revision,
        });

        Some(updated)
    }

    pub fn remove_by_locator(&self, locator: &ResourceLocator) -> Option<ResourceRecord> {
        let removed = {
            let mut registry = self.lock_registry_write();
            registry.remove_by_locator(locator)?
        };

        self.lock_payloads_write().remove(&removed.id);
        self.lock_runtime_write().remove(&removed.id);
        self.refresh_readiness(removed.id);

        self.broadcast(ResourceEvent {
            kind: ResourceEventKind::Removed,
            resource_kind: removed.kind,
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
    ) -> ResourceResult<ResourceRecord> {
        let renamed = {
            let mut registry = self.lock_registry_write();
            registry.rename(from, to.clone())?
        };
        self.refresh_readiness(renamed.id);

        self.broadcast(ResourceEvent {
            kind: ResourceEventKind::Renamed,
            resource_kind: renamed.kind,
            id: renamed.id,
            locator: Some(renamed.primary_locator.clone()),
            previous_locator: Some(from.clone()),
            revision: renamed.revision,
        });

        Ok(renamed)
    }
}
