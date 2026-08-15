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
    deferred_command_lane_count: u8,
    conservative_world_access: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SystemParamConflictKind {
    World,
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

    pub fn add_deferred_commands(&mut self) -> Result<(), SystemParamError> {
        if self.deferred_command_lane_count != 0 {
            return Err(SystemParamError::MultipleDeferredCommandParams);
        }
        self.has_deferred_commands = true;
        self.deferred_command_lane_count = 1;
        Ok(())
    }

    pub fn add_conservative_world_access(&mut self) {
        self.conservative_world_access = true;
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
        insert_type_id(&mut self.event_reads, type_id);
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
        insert_type_id(&mut self.message_reads, type_id);
        insert_type_id(&mut self.message_writes, type_id);
        Ok(())
    }

    pub(crate) fn deferred_command_lane_count(&self) -> u8 {
        self.deferred_command_lane_count
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
        self.deferred_command_lane_count = self
            .deferred_command_lane_count
            .max(other.deferred_command_lane_count);
        self.conservative_world_access |= other.conservative_world_access;
        self.component_access
            .merge_param_set_unchecked(&other.component_access);
    }

    pub fn component_access(&self) -> &QueryAccess {
        &self.component_access
    }

    pub fn has_deferred_commands(&self) -> bool {
        self.has_deferred_commands
    }

    pub fn has_conservative_world_access(&self) -> bool {
        self.conservative_world_access
    }

    pub fn conflicts_with(&self, other: &Self) -> bool {
        self.conservative_world_access
            || other.conservative_world_access
            || self
                .component_access
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
        let mut conflicts = Vec::with_capacity(system_param_conflict_upper_bound(self, other));
        if self.conservative_world_access || other.conservative_world_access {
            push_conflict(&mut conflicts, SystemParamConflictKind::World);
        }

        for component_id in self
            .component_access
            .conflicting_components_with(&other.component_access)
        {
            push_conflict(
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

fn system_param_conflict_upper_bound(left: &SystemParamAccess, right: &SystemParamAccess) -> usize {
    usize::from(left.conservative_world_access || right.conservative_world_access)
        + query_access_conflict_upper_bound(&left.component_access, &right.component_access)
        + access_conflict_upper_bound(
            &left.resource_reads,
            &left.resource_writes,
            &right.resource_reads,
            &right.resource_writes,
        )
        + access_conflict_upper_bound(
            &left.event_reads,
            &left.event_writes,
            &right.event_reads,
            &right.event_writes,
        )
        + access_conflict_upper_bound(
            &left.message_reads,
            &left.message_writes,
            &right.message_reads,
            &right.message_writes,
        )
}

fn query_access_conflict_upper_bound(left: &QueryAccess, right: &QueryAccess) -> usize {
    access_conflict_upper_bound(left.reads(), left.writes(), right.reads(), right.writes())
}

fn access_conflict_upper_bound<T>(
    left_reads: &[T],
    left_writes: &[T],
    right_reads: &[T],
    right_writes: &[T],
) -> usize {
    left_writes.len().min(right_reads.len())
        + read_only_access_count(left_reads, left_writes).min(right_writes.len())
}

fn read_only_access_count<T>(left_reads: &[T], left_writes: &[T]) -> usize {
    left_reads.len().saturating_sub(left_writes.len())
}

fn resource_access_conflicts(
    left_reads: &[ResourceId],
    left_writes: &[ResourceId],
    right_reads: &[ResourceId],
    right_writes: &[ResourceId],
) -> bool {
    access_slices_intersect(left_writes, right_reads)
        || read_only_access_intersects(left_reads, left_writes, right_writes)
}

fn type_access_conflicts(
    left_reads: &[TypeId],
    left_writes: &[TypeId],
    right_reads: &[TypeId],
    right_writes: &[TypeId],
) -> bool {
    access_slices_intersect(left_writes, right_reads)
        || read_only_access_intersects(left_reads, left_writes, right_writes)
}

fn access_slices_intersect<T>(left: &[T], right: &[T]) -> bool
where
    T: Ord,
{
    let mut left_index = 0;
    let mut right_index = 0;
    while left_index < left.len() && right_index < right.len() {
        let left_value = &left[left_index];
        let right_value = &right[right_index];
        if left_value == right_value {
            return true;
        }
        if left_value < right_value {
            left_index += 1;
        } else {
            right_index += 1;
        }
    }
    false
}

fn read_only_access_intersects<T>(reads: &[T], writes: &[T], right: &[T]) -> bool
where
    T: Copy + Ord,
{
    let mut read_index = 0;
    let mut write_index = 0;
    let mut right_index = 0;
    while read_index < reads.len() && right_index < right.len() {
        let read_value = reads[read_index];
        let right_value = right[right_index];
        if read_access_is_written(read_value, writes, &mut write_index) {
            read_index += 1;
            continue;
        }
        if read_value == right_value {
            return true;
        }
        if read_value < right_value {
            read_index += 1;
        } else {
            right_index += 1;
        }
    }
    false
}

fn push_resource_conflicts(
    conflicts: &mut Vec<SystemParamConflictKind>,
    left_reads: &[ResourceId],
    left_writes: &[ResourceId],
    right_reads: &[ResourceId],
    right_writes: &[ResourceId],
) {
    push_access_intersections(
        conflicts,
        SystemParamConflictKind::Resource,
        left_writes,
        right_reads,
    );
    // Writes are mirrored into reads, so the first pass already covers write/write
    // conflicts. The second pass scans only read-only IDs against right writes.
    push_read_only_access_intersections(
        conflicts,
        SystemParamConflictKind::Resource,
        left_reads,
        left_writes,
        right_writes,
    );
}

fn push_type_conflicts(
    conflicts: &mut Vec<SystemParamConflictKind>,
    conflict_kind: fn(TypeId) -> SystemParamConflictKind,
    left_reads: &[TypeId],
    left_writes: &[TypeId],
    right_reads: &[TypeId],
    right_writes: &[TypeId],
) {
    push_access_intersections(conflicts, conflict_kind, left_writes, right_reads);
    // Writes are mirrored into reads, so the first pass already covers write/write
    // conflicts. The second pass scans only read-only IDs against right writes.
    push_read_only_access_intersections(
        conflicts,
        conflict_kind,
        left_reads,
        left_writes,
        right_writes,
    );
}

fn push_access_intersections<T>(
    conflicts: &mut Vec<SystemParamConflictKind>,
    conflict_kind: fn(T) -> SystemParamConflictKind,
    left: &[T],
    right: &[T],
) where
    T: Copy + Ord,
{
    let mut left_index = 0;
    let mut right_index = 0;
    while left_index < left.len() && right_index < right.len() {
        let left_value = left[left_index];
        let right_value = right[right_index];
        if left_value == right_value {
            push_conflict(conflicts, conflict_kind(left_value));
            left_index += 1;
            right_index += 1;
        } else if left_value < right_value {
            left_index += 1;
        } else {
            right_index += 1;
        }
    }
}

fn push_read_only_access_intersections<T>(
    conflicts: &mut Vec<SystemParamConflictKind>,
    conflict_kind: fn(T) -> SystemParamConflictKind,
    reads: &[T],
    writes: &[T],
    right: &[T],
) where
    T: Copy + Ord,
{
    let mut read_index = 0;
    let mut write_index = 0;
    let mut right_index = 0;
    while read_index < reads.len() && right_index < right.len() {
        let read_value = reads[read_index];
        let right_value = right[right_index];
        if read_access_is_written(read_value, writes, &mut write_index) {
            read_index += 1;
            continue;
        }
        if read_value == right_value {
            push_conflict(conflicts, conflict_kind(read_value));
            read_index += 1;
            right_index += 1;
        } else if read_value < right_value {
            read_index += 1;
        } else {
            right_index += 1;
        }
    }
}

fn read_access_is_written<T>(access_id: T, writes: &[T], write_index: &mut usize) -> bool
where
    T: Copy + Ord,
{
    while *write_index < writes.len() {
        let write_value = writes[*write_index];
        if write_value < access_id {
            *write_index += 1;
            continue;
        }
        if write_value == access_id {
            *write_index += 1;
            return true;
        }
        return false;
    }
    false
}

fn push_conflict(conflicts: &mut Vec<SystemParamConflictKind>, conflict: SystemParamConflictKind) {
    conflicts.push(conflict);
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
