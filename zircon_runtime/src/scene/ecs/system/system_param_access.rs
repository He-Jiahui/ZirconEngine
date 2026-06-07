use std::any::TypeId;

use crate::scene::ecs::{ComponentId, QueryAccess, ResourceId, SystemParamError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SystemParamAccess {
    component_access: QueryAccess,
    resource_reads: Vec<ResourceId>,
    resource_writes: Vec<ResourceId>,
    event_reads: Vec<TypeId>,
    event_writes: Vec<TypeId>,
    message_reads: Vec<TypeId>,
    message_writes: Vec<TypeId>,
    has_deferred_commands: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SystemParamConflictKind {
    Component(ComponentId),
    Resource(ResourceId),
    Event(TypeId),
    Message(TypeId),
}

impl SystemParamAccess {
    pub fn add_query_access(&mut self, query_access: &QueryAccess) -> Result<(), SystemParamError> {
        for component_id in query_access.writes() {
            self.component_access.add_write(*component_id)?;
        }
        for component_id in query_access.reads() {
            if query_access.writes().binary_search(component_id).is_err() {
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

    pub fn add_message_read<T>(&mut self) -> Result<(), SystemParamError>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        if contains_type_id(&self.message_writes, type_id) {
            return Err(SystemParamError::ConflictingMessageAccess {
                type_name: std::any::type_name::<T>(),
            });
        }
        insert_type_id(&mut self.message_reads, type_id);
        Ok(())
    }

    pub fn add_message_write<T>(&mut self) -> Result<(), SystemParamError>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        if contains_type_id(&self.message_reads, type_id)
            || contains_type_id(&self.message_writes, type_id)
        {
            return Err(SystemParamError::ConflictingMessageAccess {
                type_name: std::any::type_name::<T>(),
            });
        }
        insert_type_id(&mut self.message_writes, type_id);
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
        for type_id in other.message_reads.iter().copied() {
            insert_type_id(&mut self.message_reads, type_id);
        }
        for type_id in other.message_writes.iter().copied() {
            insert_type_id(&mut self.message_writes, type_id);
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

    pub fn conflicts_with(&self, other: &Self) -> bool {
        self.component_access
            .conflicts_with(&other.component_access)
            || resource_access_conflicts(
                &self.resource_reads,
                &self.resource_writes,
                &other.resource_reads,
                &other.resource_writes,
            )
            || type_access_conflicts(
                &self.event_reads,
                &self.event_writes,
                &other.event_reads,
                &other.event_writes,
            )
            || type_access_conflicts(
                &self.message_reads,
                &self.message_writes,
                &other.message_reads,
                &other.message_writes,
            )
    }

    pub fn conflict_kinds_with(&self, other: &Self) -> Vec<SystemParamConflictKind> {
        let mut conflicts = Vec::new();

        for component_id in self
            .component_access
            .conflicting_components_with(&other.component_access)
        {
            insert_conflict(
                &mut conflicts,
                SystemParamConflictKind::Component(component_id),
            );
        }

        push_resource_conflicts(
            &mut conflicts,
            &self.resource_reads,
            &self.resource_writes,
            &other.resource_reads,
            &other.resource_writes,
        );
        push_type_conflicts(
            &mut conflicts,
            SystemParamConflictKind::Event,
            &self.event_reads,
            &self.event_writes,
            &other.event_reads,
            &other.event_writes,
        );
        push_type_conflicts(
            &mut conflicts,
            SystemParamConflictKind::Message,
            &self.message_reads,
            &self.message_writes,
            &other.message_reads,
            &other.message_writes,
        );

        conflicts
    }
}

fn resource_access_conflicts(
    left_reads: &[ResourceId],
    left_writes: &[ResourceId],
    right_reads: &[ResourceId],
    right_writes: &[ResourceId],
) -> bool {
    resource_intersects(left_writes, right_reads)
        || resource_intersects(left_reads, right_writes)
        || resource_intersects(left_writes, right_writes)
}

fn resource_intersects(left: &[ResourceId], right: &[ResourceId]) -> bool {
    left.iter()
        .any(|resource_id| contains_id(right, *resource_id))
}

fn type_access_conflicts(
    left_reads: &[TypeId],
    left_writes: &[TypeId],
    right_reads: &[TypeId],
    right_writes: &[TypeId],
) -> bool {
    type_intersects(left_writes, right_reads)
        || type_intersects(left_reads, right_writes)
        || type_intersects(left_writes, right_writes)
}

fn type_intersects(left: &[TypeId], right: &[TypeId]) -> bool {
    left.iter().any(|type_id| contains_type_id(right, *type_id))
}

fn push_resource_conflicts(
    conflicts: &mut Vec<SystemParamConflictKind>,
    left_reads: &[ResourceId],
    left_writes: &[ResourceId],
    right_reads: &[ResourceId],
    right_writes: &[ResourceId],
) {
    push_resource_intersections(conflicts, left_writes, right_reads);
    push_resource_intersections(conflicts, left_reads, right_writes);
    push_resource_intersections(conflicts, left_writes, right_writes);
}

fn push_resource_intersections(
    conflicts: &mut Vec<SystemParamConflictKind>,
    left: &[ResourceId],
    right: &[ResourceId],
) {
    for resource_id in left {
        if contains_id(right, *resource_id) {
            insert_conflict(conflicts, SystemParamConflictKind::Resource(*resource_id));
        }
    }
}

fn push_type_conflicts(
    conflicts: &mut Vec<SystemParamConflictKind>,
    conflict_kind: fn(TypeId) -> SystemParamConflictKind,
    left_reads: &[TypeId],
    left_writes: &[TypeId],
    right_reads: &[TypeId],
    right_writes: &[TypeId],
) {
    push_type_intersections(conflicts, conflict_kind, left_writes, right_reads);
    push_type_intersections(conflicts, conflict_kind, left_reads, right_writes);
    push_type_intersections(conflicts, conflict_kind, left_writes, right_writes);
}

fn push_type_intersections(
    conflicts: &mut Vec<SystemParamConflictKind>,
    conflict_kind: fn(TypeId) -> SystemParamConflictKind,
    left: &[TypeId],
    right: &[TypeId],
) {
    for type_id in left {
        if contains_type_id(right, *type_id) {
            insert_conflict(conflicts, conflict_kind(*type_id));
        }
    }
}

fn insert_conflict(
    conflicts: &mut Vec<SystemParamConflictKind>,
    conflict: SystemParamConflictKind,
) {
    if !conflicts.contains(&conflict) {
        conflicts.push(conflict);
    }
}

fn insert_id(ids: &mut Vec<ResourceId>, resource_id: ResourceId) {
    if let Err(index) = ids.binary_search(&resource_id) {
        ids.insert(index, resource_id);
    }
}

fn contains_id(ids: &[ResourceId], resource_id: ResourceId) -> bool {
    ids.binary_search(&resource_id).is_ok()
}

fn insert_type_id(ids: &mut Vec<TypeId>, type_id: TypeId) {
    if let Err(index) = ids.binary_search(&type_id) {
        ids.insert(index, type_id);
    }
}

fn contains_type_id(ids: &[TypeId], type_id: TypeId) -> bool {
    ids.binary_search(&type_id).is_ok()
}
