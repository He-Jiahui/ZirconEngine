use std::any::TypeId;

use crate::scene::ecs::{Component, ComponentId, InternalEntity, LifecycleEventKind, StorageType};
use crate::scene::{EntityId, World};

type DenseComponentValue = Box<dyn std::any::Any + Send + Sync>;

/// A type-erased removal staged alongside a deferred final-row bundle. The
/// function pointers keep type-specific change tracking and removal events at
/// the same publication boundary as the final archetype row.
#[derive(Clone, Copy)]
pub(super) struct PendingDeferredRemoval {
    type_id: TypeId,
    component_id: ComponentId,
    storage_type: StorageType,
    apply_sparse: fn(&mut World, EntityId, InternalEntity, ComponentId) -> bool,
    apply_table: fn(&mut World, EntityId, ComponentId, DenseComponentValue),
}

impl PendingDeferredRemoval {
    pub(super) fn new<T>(component_id: ComponentId) -> Self
    where
        T: Component,
    {
        Self {
            type_id: TypeId::of::<T>(),
            component_id,
            storage_type: T::STORAGE_TYPE,
            apply_sparse: apply_sparse_removal::<T>,
            apply_table: apply_table_removal::<T>,
        }
    }

    pub(super) const fn type_id(self) -> TypeId {
        self.type_id
    }

    pub(super) const fn component_id(self) -> ComponentId {
        self.component_id
    }

    pub(super) const fn storage_type(self) -> StorageType {
        self.storage_type
    }

    pub(super) fn publish_sparse(
        self,
        world: &mut World,
        entity: EntityId,
        internal: InternalEntity,
    ) -> bool {
        (self.apply_sparse)(world, entity, internal, self.component_id)
    }

    pub(super) fn publish_table(
        self,
        world: &mut World,
        entity: EntityId,
        value: DenseComponentValue,
    ) {
        (self.apply_table)(world, entity, self.component_id, value);
    }
}

fn apply_sparse_removal<T>(
    world: &mut World,
    entity: EntityId,
    internal: InternalEntity,
    component_id: ComponentId,
) -> bool
where
    T: Component,
{
    let removed = world
        .component_storage
        .remove::<T>(component_id, internal)
        .expect("preflighted deferred sparse removal must preserve component type");
    let Some(removed) = removed else {
        return false;
    };
    apply_removed_component(world, entity, component_id, removed.value);
    true
}

fn apply_table_removal<T>(
    world: &mut World,
    entity: EntityId,
    component_id: ComponentId,
    value: DenseComponentValue,
) where
    T: Component,
{
    let value = value
        .downcast::<T>()
        .expect("preflighted deferred table removal must preserve component type");
    apply_removed_component(world, entity, component_id, *value);
}

fn apply_removed_component<T>(
    world: &mut World,
    entity: EntityId,
    component_id: ComponentId,
    value: T,
) where
    T: Component,
{
    if let Some(previous_parent) = World::hierarchy_parent_from_component(&value) {
        world.update_hierarchy_mutation_index(entity, previous_parent, None);
    }
    world.record_removed_component::<T>(entity);
    world.mark_preflighted_bundle_component_mutation::<T>(entity);
    world.mark_scene_binding_component_removal::<T>(entity, &value);
    world.trigger_component_lifecycle(LifecycleEventKind::Remove, entity, component_id);
}
