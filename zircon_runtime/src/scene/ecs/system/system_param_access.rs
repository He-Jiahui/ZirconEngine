use crate::scene::ecs::{QueryAccess, ResourceId, SystemParamError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SystemParamAccess {
    component_access: QueryAccess,
    resource_reads: Vec<ResourceId>,
    resource_writes: Vec<ResourceId>,
    has_deferred_commands: bool,
}

impl SystemParamAccess {
    pub fn add_query_access(&mut self, query_access: &QueryAccess) -> Result<(), SystemParamError> {
        for component_id in query_access.writes() {
            self.component_access.add_write(*component_id)?;
        }
        for component_id in query_access.reads() {
            if !query_access.writes().contains(component_id) {
                self.component_access.add_read(*component_id)?;
            }
        }
        for component_id in query_access.with() {
            self.component_access.add_with(*component_id);
        }
        for component_id in query_access.without() {
            self.component_access.add_without(*component_id);
        }
        Ok(())
    }

    pub fn add_resource_read(&mut self, resource_id: ResourceId) -> Result<(), SystemParamError> {
        if contains_id(&self.resource_writes, resource_id) {
            return Err(SystemParamError::ConflictingResourceAccess { resource_id });
        }
        insert_id(&mut self.resource_reads, resource_id);
        Ok(())
    }

    pub fn add_resource_write(&mut self, resource_id: ResourceId) -> Result<(), SystemParamError> {
        if contains_id(&self.resource_reads, resource_id)
            || contains_id(&self.resource_writes, resource_id)
        {
            return Err(SystemParamError::ConflictingResourceAccess { resource_id });
        }
        insert_id(&mut self.resource_reads, resource_id);
        insert_id(&mut self.resource_writes, resource_id);
        Ok(())
    }

    pub fn add_deferred_commands(&mut self) {
        self.has_deferred_commands = true;
    }

    pub fn component_access(&self) -> &QueryAccess {
        &self.component_access
    }

    pub fn has_deferred_commands(&self) -> bool {
        self.has_deferred_commands
    }
}

fn insert_id(ids: &mut Vec<ResourceId>, resource_id: ResourceId) {
    if !contains_id(ids, resource_id) {
        ids.push(resource_id);
        ids.sort_unstable();
    }
}

fn contains_id(ids: &[ResourceId], resource_id: ResourceId) -> bool {
    ids.binary_search(&resource_id).is_ok()
}
