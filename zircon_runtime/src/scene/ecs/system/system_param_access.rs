use std::any::TypeId;

use crate::scene::ecs::{QueryAccess, ResourceId, SystemParamError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SystemParamAccess {
    component_access: QueryAccess,
    resource_reads: Vec<ResourceId>,
    resource_writes: Vec<ResourceId>,
    event_reads: Vec<TypeId>,
    event_writes: Vec<TypeId>,
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

    pub fn add_event_read<T>(&mut self) -> Result<(), SystemParamError>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        if contains_type_id(&self.event_writes, type_id) {
            return Err(SystemParamError::ConflictingEventAccess {
                type_name: std::any::type_name::<T>(),
            });
        }
        insert_type_id(&mut self.event_reads, type_id);
        Ok(())
    }

    pub fn add_event_write<T>(&mut self) -> Result<(), SystemParamError>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        if contains_type_id(&self.event_reads, type_id)
            || contains_type_id(&self.event_writes, type_id)
        {
            return Err(SystemParamError::ConflictingEventAccess {
                type_name: std::any::type_name::<T>(),
            });
        }
        insert_type_id(&mut self.event_writes, type_id);
        Ok(())
    }

    pub(crate) fn merge_param_set_access(&mut self, other: &Self) {
        for resource_id in other.resource_reads.iter().copied() {
            insert_id(&mut self.resource_reads, resource_id);
        }
        for resource_id in other.resource_writes.iter().copied() {
            insert_id(&mut self.resource_writes, resource_id);
        }
        for type_id in other.event_reads.iter().copied() {
            insert_type_id(&mut self.event_reads, type_id);
        }
        for type_id in other.event_writes.iter().copied() {
            insert_type_id(&mut self.event_writes, type_id);
        }
        self.has_deferred_commands |= other.has_deferred_commands;
        self.component_access
            .merge_param_set_unchecked(&other.component_access);
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

fn insert_type_id(ids: &mut Vec<TypeId>, type_id: TypeId) {
    if !contains_type_id(ids, type_id) {
        ids.push(type_id);
    }
}

fn contains_type_id(ids: &[TypeId], type_id: TypeId) -> bool {
    ids.contains(&type_id)
}
