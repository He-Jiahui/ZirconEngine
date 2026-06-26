pub(super) mod fixed_components;

use crate::scene::ecs::{
    Bundle, Component, ComponentId, ComponentRemoveResult, LifecycleEventKind, Resource, ResourceId,
};
use crate::scene::{EntityId, NodeKind};

use super::{SceneError, SceneResult, World};

impl World {
    pub fn spawn<B>(&mut self, bundle: B) -> SceneResult<EntityId>
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
        self.kinds.insert(entity, NodeKind::Empty);
        self.refresh_stable_entity_locations();
        self.bump_query_cache_revision();
        self.mark_derived_state_dirty();
        true
    }

    pub(crate) fn spawn_at<B>(&mut self, entity: EntityId, bundle: B) -> SceneResult<EntityId>
    where
        B: Bundle,
    {
        self.spawn_empty_at(entity);
        self.insert_bundle(entity, bundle)?;
        Ok(entity)
    }

    pub(crate) fn insert_bundle<B>(&mut self, entity: EntityId, bundle: B) -> SceneResult<()>
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
        let Some(internal) = self.internal_entity(entity) else {
            return false;
        };

        self.component_storage.contains(component_id, internal)
    }

    pub fn contains_component<T>(&self, entity: EntityId) -> bool
    where
        T: Component,
    {
        let Some(component_id) = self.registered_component_id::<T>() else {
            return false;
        };

        self.contains_component_id(entity, component_id)
    }

    pub fn is_component_added<T>(&self, entity: EntityId) -> bool
    where
        T: Component,
    {
        let Some(ticks) = self.component_change_ticks::<T>(entity) else {
            return false;
        };

        ticks.is_added(crate::scene::ecs::ChangeTickWindow::new(
            self.last_change_tick(),
            self.read_change_tick(),
        ))
    }

    pub fn is_component_changed<T>(&self, entity: EntityId) -> bool
    where
        T: Component,
    {
        let Some(ticks) = self.component_change_ticks::<T>(entity) else {
            return false;
        };

        ticks.is_changed(crate::scene::ecs::ChangeTickWindow::new(
            self.last_change_tick(),
            self.read_change_tick(),
        ))
    }

    pub fn insert<T>(&mut self, entity: EntityId, component: T) -> SceneResult<Option<T>>
    where
        T: Component,
    {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("insert component on", entity));
        }

        let tick = self.mutation_change_tick();
        let component_id = self.component_id::<T>();
        let was_present = self.contains_component_id(entity, component_id);
        self.insert_fixed_component(entity, &component)?;
        let internal = self
            .internal_entity(entity)
            .expect("stable entity must have an internal identity");
        let old = match self.component_storage.insert_at_tick(
            component_id,
            T::STORAGE_TYPE,
            internal,
            component,
            tick,
        ) {
            Ok(old) => old,
            Err(error) => return Err(error.into()),
        };

        self.mark_component_mutation::<T>();
        if !was_present {
            self.refresh_entity_archetype(entity);
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

    pub fn remove<T>(&mut self, entity: EntityId) -> SceneResult<Option<T>>
    where
        T: Component,
    {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("remove component from", entity));
        }
        let component_id = self.registered_component_id::<T>();
        let internal = self
            .internal_entity(entity)
            .expect("stable entity must have an internal identity");
        if self.is_fixed_component_type::<T>() {
            if let Some(component_id) = component_id {
                if self.contains_component_id(entity, component_id) {
                    self.trigger_component_lifecycle(
                        LifecycleEventKind::Remove,
                        entity,
                        component_id,
                    );
                }
            }
            let removed = self.remove_fixed_component_value::<T>(entity);
            let mut removed_from_storage = false;
            if let Some(component_id) = component_id {
                removed_from_storage = self.component_storage.contains(component_id, internal);
                match self.component_storage.remove::<T>(component_id, internal) {
                    Ok(_) => {}
                    Err(error) => return Err(error.into()),
                }
            }
            if removed.is_some() {
                self.record_removed_component::<T>(entity);
                self.mark_component_mutation::<T>();
            }
            if removed.is_some() || removed_from_storage {
                self.refresh_entity_archetype(entity);
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
        let removed = match self.component_storage.remove::<T>(component_id, internal) {
            Ok(Some(ComponentRemoveResult { value, .. })) => Some(value),
            Ok(None) => None,
            Err(error) => return Err(error.into()),
        };
        if removed.is_some() {
            self.record_removed_component::<T>(entity);
            self.mark_component_mutation::<T>();
            self.refresh_entity_archetype(entity);
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

    pub fn contains_resource<T>(&self) -> bool
    where
        T: Resource,
    {
        self.resources.contains::<T>()
    }

    pub fn is_resource_added<T>(&self) -> bool
    where
        T: Resource,
    {
        let Some(ticks) = self.resource_change_ticks::<T>() else {
            return false;
        };

        ticks.is_added(crate::scene::ecs::ChangeTickWindow::new(
            self.last_change_tick(),
            self.read_change_tick(),
        ))
    }

    pub fn is_resource_changed<T>(&self) -> bool
    where
        T: Resource,
    {
        let Some(ticks) = self.resource_change_ticks::<T>() else {
            return false;
        };

        ticks.is_changed(crate::scene::ecs::ChangeTickWindow::new(
            self.last_change_tick(),
            self.read_change_tick(),
        ))
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
        let Some(resource) = self.get_resource::<T>() else {
            panic!(
                "requested missing scene resource {}",
                std::any::type_name::<T>()
            );
        };

        resource
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
        let Some(resource) = self.get_resource_mut::<T>() else {
            panic!(
                "requested missing scene resource {}",
                std::any::type_name::<T>()
            );
        };

        resource
    }

    pub fn get_resource_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Resource,
    {
        let Some((resource, _ticks)) = self.resource_mut_with_ticks::<T>() else {
            return None;
        };

        Some(resource)
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
    ) -> SceneResult<()> {
        let component_id = self
            .component_registry
            .dynamic_component_id(component_type_id);
        let internal = self
            .internal_entity(entity)
            .expect("stable entity must have an internal identity");
        let tick = self.mutation_change_tick();
        let old = self.component_storage.insert_at_tick(
            component_id,
            crate::scene::ecs::StorageType::SparseSet,
            internal,
            DynamicComponentPresence,
            tick,
        )?;
        if old.is_none() {
            self.refresh_entity_archetype(entity);
            self.bump_query_cache_revision();
        }
        Ok(())
    }

    pub(super) fn remove_dynamic_component_presence(
        &mut self,
        entity: EntityId,
        component_type_id: &str,
    ) -> SceneResult<()> {
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
            .remove::<DynamicComponentPresence>(component_id, internal)?;
        if removed.is_some() {
            self.refresh_entity_archetype(entity);
            self.bump_query_cache_revision();
        }
        Ok(())
    }

    pub(super) fn rebuild_typed_component_presence(&mut self) {
        self.component_registry = Default::default();
        self.component_storage = Default::default();
        self.archetype_index = Default::default();
        let mut dynamic_component_type_ids = Vec::new();
        for entity_index in 0..self.entities.len() {
            let entity = self.entities[entity_index];
            self.rebuild_fixed_component_presence_for_entity(entity);
            self.dynamic_component_type_ids_for_presence_rebuild(
                entity,
                &mut dynamic_component_type_ids,
            );
            for component_type_id in &dynamic_component_type_ids {
                let _ = self.insert_dynamic_component_presence(entity, component_type_id);
            }
        }
        self.rebuild_archetype_index();
        self.mark_derived_state_dirty();
    }

    fn dynamic_component_type_ids_for_presence_rebuild(
        &self,
        entity: EntityId,
        output: &mut Vec<String>,
    ) {
        output.clear();
        let Some(components) = self.dynamic_components.get(&entity) else {
            return;
        };
        output.reserve(components.len());
        for component_type_id in components.keys() {
            output.push(component_type_id.clone());
        }
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
