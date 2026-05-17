pub(super) mod fixed_components;

use crate::scene::ecs::{
    Bundle, Component, ComponentId, ComponentRemoveResult, LifecycleEventKind, Resource, ResourceId,
};
use crate::scene::{EntityId, NodeKind};

use super::World;

impl World {
    pub fn spawn<B>(&mut self, bundle: B) -> Result<EntityId, String>
    where
        B: Bundle,
    {
        let entity = self.spawn_node(NodeKind::Mesh);
        self.insert_bundle(entity, bundle)?;
        Ok(entity)
    }

    pub(crate) fn spawn_empty_at(&mut self, entity: EntityId) -> bool {
        if self.contains_entity(entity) {
            return false;
        }
        if self.next_id <= entity {
            self.next_id = entity + 1;
        }
        self.register_stable_entity(entity)
            .expect("reserved scene entity must have a unique stable id");
        self.entities.push(entity);
        self.kinds.insert(entity, NodeKind::Mesh);
        self.refresh_stable_entity_locations();
        self.bump_query_cache_revision();
        self.mark_derived_state_dirty();
        true
    }

    pub(crate) fn spawn_at<B>(&mut self, entity: EntityId, bundle: B) -> Result<EntityId, String>
    where
        B: Bundle,
    {
        self.spawn_empty_at(entity);
        self.insert_bundle(entity, bundle)?;
        Ok(entity)
    }

    pub(crate) fn insert_bundle<B>(&mut self, entity: EntityId, bundle: B) -> Result<(), String>
    where
        B: Bundle,
    {
        bundle.insert_into(self, entity)
    }

    pub fn component_id<T>(&mut self) -> ComponentId
    where
        T: Component,
    {
        self.component_registry.component_id::<T>()
    }

    pub fn registered_component_id<T>(&self) -> Option<ComponentId>
    where
        T: Component,
    {
        self.component_registry.registered_component_id::<T>()
    }

    pub fn registered_dynamic_component_id(&self, component_type_id: &str) -> Option<ComponentId> {
        self.component_registry
            .registered_dynamic_component_id(component_type_id)
    }

    pub fn component_count_for_id(&self, component_id: ComponentId) -> usize {
        self.component_storage.len_for_component(component_id)
    }

    pub fn contains_component_id(&self, entity: EntityId, component_id: ComponentId) -> bool {
        self.internal_entity(entity)
            .is_some_and(|internal| self.component_storage.contains(component_id, internal))
    }

    pub fn insert<T>(&mut self, entity: EntityId, component: T) -> Result<Option<T>, String>
    where
        T: Component,
    {
        if !self.contains_entity(entity) {
            return Err(format!(
                "cannot insert component on missing entity {entity}"
            ));
        }

        let tick = self.mutation_change_tick();
        let component_id = self.component_id::<T>();
        let was_present = self.contains_component_id(entity, component_id);
        self.insert_fixed_component(entity, &component)?;
        let internal = self
            .internal_entity(entity)
            .expect("stable entity must have an internal identity");
        let old = self
            .component_storage
            .insert_at_tick(component_id, T::STORAGE_TYPE, internal, component, tick)
            .map_err(|error| error.to_string())?;

        self.mark_component_mutation::<T>();
        if !was_present {
            self.bump_query_cache_revision();
        }
        if was_present {
            self.trigger_component_lifecycle(LifecycleEventKind::Replace, entity, component_id);
        } else {
            self.trigger_component_lifecycle(LifecycleEventKind::Add, entity, component_id);
        }
        self.trigger_component_lifecycle(LifecycleEventKind::Insert, entity, component_id);
        Ok(old)
    }

    pub fn get<T>(&self, entity: EntityId) -> Option<&T>
    where
        T: Component,
    {
        if let Some(component) = self.fixed_component_ref::<T>(entity) {
            return Some(component);
        }
        let component_id = self.registered_component_id::<T>()?;
        let internal = self.internal_entity(entity)?;
        self.component_storage.get(component_id, internal)
    }

    pub fn get_mut<T>(&mut self, entity: EntityId) -> Option<&mut T>
    where
        T: Component,
    {
        let tick = self.mutation_change_tick();
        if self.is_fixed_component_type::<T>() {
            self.mark_component_changed_at_tick::<T>(entity, tick);
            self.mark_component_mutation::<T>();
            return self.fixed_component_mut::<T>(entity);
        }
        let component_id = self.registered_component_id::<T>()?;
        let internal = self.internal_entity(entity)?;
        self.mark_component_mutation::<T>();
        self.component_storage
            .get_mut_at_tick(component_id, internal, tick)
    }

    pub fn remove<T>(&mut self, entity: EntityId) -> Result<Option<T>, String>
    where
        T: Component,
    {
        if !self.contains_entity(entity) {
            return Err(format!(
                "cannot remove component from missing entity {entity}"
            ));
        }
        let component_id = self.registered_component_id::<T>();
        let internal = self
            .internal_entity(entity)
            .expect("stable entity must have an internal identity");
        if self.is_fixed_component_type::<T>() {
            if component_id
                .is_some_and(|component_id| self.contains_component_id(entity, component_id))
            {
                self.trigger_component_lifecycle(
                    LifecycleEventKind::Remove,
                    entity,
                    component_id.expect("checked component id must be present"),
                );
            }
            let removed = self.remove_fixed_component_value::<T>(entity);
            let mut removed_from_storage = false;
            if let Some(component_id) = component_id {
                removed_from_storage = self.component_storage.contains(component_id, internal);
                self.component_storage
                    .remove::<T>(component_id, internal)
                    .map_err(|error| error.to_string())?;
            }
            if removed.is_some() {
                self.record_removed_component::<T>(entity);
                self.mark_component_mutation::<T>();
            }
            if removed.is_some() || removed_from_storage {
                self.bump_query_cache_revision();
            }
            return Ok(removed);
        }
        let Some(component_id) = component_id else {
            return Ok(None);
        };
        if self.contains_component_id(entity, component_id) {
            self.trigger_component_lifecycle(LifecycleEventKind::Remove, entity, component_id);
        }
        let removed = self
            .component_storage
            .remove::<T>(component_id, internal)
            .map_err(|error| error.to_string())?
            .map(|ComponentRemoveResult { value, .. }| value);
        if removed.is_some() {
            self.record_removed_component::<T>(entity);
            self.mark_component_mutation::<T>();
            self.bump_query_cache_revision();
        }
        Ok(removed)
    }

    pub fn resource_id<T>(&mut self) -> ResourceId
    where
        T: Resource,
    {
        self.resource_registry.resource_id::<T>()
    }

    pub fn registered_resource_id<T>(&self) -> Option<ResourceId>
    where
        T: Resource,
    {
        self.resource_registry.registered_resource_id::<T>()
    }

    pub fn insert_resource<T>(&mut self, resource: T) -> Option<T>
    where
        T: Resource,
    {
        self.resource_id::<T>();
        let tick = self.mutation_change_tick();
        self.resources.insert_at_tick(resource, tick)
    }

    pub fn resource<T>(&self) -> &T
    where
        T: Resource,
    {
        self.get_resource::<T>().unwrap_or_else(|| {
            panic!(
                "requested missing scene resource {}",
                std::any::type_name::<T>()
            )
        })
    }

    pub fn get_resource<T>(&self) -> Option<&T>
    where
        T: Resource,
    {
        self.resources.get::<T>()
    }

    pub fn resource_mut<T>(&mut self) -> &mut T
    where
        T: Resource,
    {
        self.get_resource_mut::<T>().unwrap_or_else(|| {
            panic!(
                "requested missing scene resource {}",
                std::any::type_name::<T>()
            )
        })
    }

    pub fn get_resource_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Resource,
    {
        self.resource_mut_with_ticks::<T>()
            .map(|(resource, _ticks)| resource)
    }

    pub fn remove_resource<T>(&mut self) -> Option<T>
    where
        T: Resource,
    {
        self.resources.remove::<T>()
    }

    pub fn query<D>(&mut self) -> crate::scene::ecs::QueryState<D>
    where
        D: crate::scene::ecs::QueryDataAccess,
    {
        crate::scene::ecs::QueryState::new(self)
    }

    pub fn query_filtered<D, F>(&mut self) -> crate::scene::ecs::QueryState<D, F>
    where
        D: crate::scene::ecs::QueryDataAccess,
        F: crate::scene::ecs::QueryFilter,
    {
        crate::scene::ecs::QueryState::new(self)
    }

    pub(crate) fn entity_ids_for_query(&self) -> &[EntityId] {
        &self.entities
    }

    pub(super) fn insert_dynamic_component_presence(
        &mut self,
        entity: EntityId,
        component_type_id: &str,
    ) -> Result<(), String> {
        let component_id = self
            .component_registry
            .dynamic_component_id(component_type_id);
        let internal = self
            .internal_entity(entity)
            .expect("stable entity must have an internal identity");
        let tick = self.mutation_change_tick();
        let old = self
            .component_storage
            .insert_at_tick(
                component_id,
                crate::scene::ecs::StorageType::SparseSet,
                internal,
                DynamicComponentPresence,
                tick,
            )
            .map_err(|error| error.to_string())?;
        if old.is_none() {
            self.bump_query_cache_revision();
        }
        Ok(())
    }

    pub(super) fn remove_dynamic_component_presence(
        &mut self,
        entity: EntityId,
        component_type_id: &str,
    ) -> Result<(), String> {
        let Some(component_id) = self
            .component_registry
            .registered_dynamic_component_id(component_type_id)
        else {
            return Ok(());
        };
        let Some(internal) = self.internal_entity(entity) else {
            return Ok(());
        };
        let removed = self
            .component_storage
            .remove::<DynamicComponentPresence>(component_id, internal)
            .map_err(|error| error.to_string())?;
        if removed.is_some() {
            self.bump_query_cache_revision();
        }
        Ok(())
    }

    pub(super) fn rebuild_typed_component_presence(&mut self) {
        self.component_registry = Default::default();
        self.component_storage = Default::default();
        for entity in self.entities.clone() {
            self.rebuild_fixed_component_presence_for_entity(entity);
            if let Some(components) = self.dynamic_components.get(&entity).cloned() {
                for component_type_id in components.keys() {
                    let _ = self.insert_dynamic_component_presence(entity, component_type_id);
                }
            }
        }
        self.mark_derived_state_dirty();
    }

    fn mark_component_mutation<T>(&mut self)
    where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        if self.is_hierarchy_component_type(type_id) {
            self.mark_hierarchy_dirty();
        } else if self.is_transform_component_type(type_id) {
            self.mark_transform_dirty();
        } else if self.is_active_component_type(type_id) {
            self.mark_active_state_dirty();
        } else {
            self.mark_node_cache_dirty();
        }
    }

    fn is_hierarchy_component_type(&self, type_id: std::any::TypeId) -> bool {
        type_id == std::any::TypeId::of::<crate::scene::components::Hierarchy>()
    }

    fn is_transform_component_type(&self, type_id: std::any::TypeId) -> bool {
        type_id == std::any::TypeId::of::<crate::scene::components::LocalTransform>()
    }

    fn is_active_component_type(&self, type_id: std::any::TypeId) -> bool {
        type_id == std::any::TypeId::of::<crate::scene::components::ActiveSelf>()
    }
}

#[derive(Debug)]
struct DynamicComponentPresence;

impl Component for DynamicComponentPresence {
    const STORAGE_TYPE: crate::scene::ecs::StorageType = crate::scene::ecs::StorageType::SparseSet;
}
