mod bundle_transaction;
mod component_mutation_effects;
mod dynamic_component_presence;
pub(super) mod fixed_components;

pub(super) use bundle_transaction::BundleInsertionTransaction;
use dynamic_component_presence::DynamicComponentPresence;

use crate::scene::ecs::{
    Bundle, Component, ComponentId, ComponentRemoveResult, LifecycleEventKind, Resource, ResourceId,
};
use crate::scene::{components::Mobility, EntityId, NodeKind};

use super::{SceneError, SceneResult, World};

impl World {
    pub fn spawn<B>(&mut self, bundle: B) -> SceneResult<EntityId>
    where
        B: Bundle,
    {
        let entity = self.next_id;
        let mut transaction = self.begin_bundle_spawn(entity, NodeKind::Mesh)?;
        bundle.stage_into(&mut transaction)?;
        transaction.finish()?;
        Ok(entity)
    }

    pub(crate) fn spawn_empty_at(&mut self, entity: EntityId) -> SceneResult<bool> {
        if self.contains_entity(entity) {
            return Ok(false);
        }
        let next_id = if self.next_id <= entity {
            entity
                .checked_add(1)
                .ok_or(SceneError::EntityIdExhausted { entity })?
        } else {
            self.next_id
        };
        self.register_stable_entity(entity)?;
        self.next_id = self.next_id.max(next_id);
        self.entities.push(entity);
        self.kinds.insert(entity, NodeKind::Empty);
        self.record_node_kind_added(NodeKind::Empty);
        self.place_empty_entity_in_archetype(entity);
        self.bump_query_cache_revision();
        self.mark_derived_state_dirty();
        self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
        self.advance_world_generation();
        self.advance_scene_binding_generations_for_new_descendant(entity);
        Ok(true)
    }

    pub(crate) fn spawn_at<B>(&mut self, entity: EntityId, bundle: B) -> SceneResult<EntityId>
    where
        B: Bundle,
    {
        let mut transaction = self.begin_bundle_spawn(entity, NodeKind::Empty)?;
        bundle.stage_into(&mut transaction)?;
        transaction.finish()?;
        Ok(entity)
    }

    pub(crate) fn insert_bundle<B>(&mut self, entity: EntityId, bundle: B) -> SceneResult<()>
    where
        B: Bundle,
    {
        bundle.insert_into(self, entity)
    }

    pub(crate) fn begin_bundle_insertion(
        &mut self,
        entity: EntityId,
    ) -> SceneResult<BundleInsertionTransaction<'_>> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("insert component on", entity));
        }
        let internal_entity = self
            .internal_entity(entity)
            .ok_or_else(|| SceneError::missing_entity("insert component on", entity))?;
        Ok(BundleInsertionTransaction::new(
            self,
            entity,
            internal_entity,
        ))
    }

    fn begin_bundle_spawn(
        &mut self,
        entity: EntityId,
        kind: NodeKind,
    ) -> SceneResult<BundleInsertionTransaction<'_>> {
        let record = self.default_node_record(entity, kind);
        self.validate_owned_node_records(std::slice::from_ref(&record))?;
        BundleInsertionTransaction::new_spawn(self, record)
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
        let current_hierarchy_parent = Self::hierarchy_parent_from_component(&component);
        self.insert_fixed_component(entity, &component)?;
        let internal = self
            .internal_entity(entity)
            .ok_or_else(|| SceneError::missing_entity("insert component on", entity))?;
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

        self.mark_component_mutation::<T>(entity);
        self.mark_scene_binding_component_replacement::<T>(
            entity,
            old.as_ref(),
            current_hierarchy_parent,
        );
        if !was_present {
            self.add_component_to_entity_archetype(entity, component_id, T::STORAGE_TYPE);
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

    fn insert_preflighted_bundle_component<T>(
        &mut self,
        entity: EntityId,
        component: T,
        component_id: ComponentId,
    ) -> SceneResult<bool>
    where
        T: Component,
    {
        if self.component_id::<T>() != component_id {
            return Err(SceneError::Message(
                "bundle component registry changed after preflight".to_string(),
            ));
        }
        let internal = self
            .internal_entity(entity)
            .ok_or_else(|| SceneError::missing_entity("insert component on", entity))?;
        self.component_storage
            .validate_insert::<T>(component_id, T::STORAGE_TYPE)?;
        let was_present = self.contains_component_id(entity, component_id);
        let current_hierarchy_parent = Self::hierarchy_parent_from_component(&component);
        self.insert_prevalidated_fixed_component(entity, &component);
        let tick = self.mutation_change_tick();
        let old = self.component_storage.insert_at_tick(
            component_id,
            T::STORAGE_TYPE,
            internal,
            component,
            tick,
        )?;

        self.mark_preflighted_bundle_component_mutation::<T>(entity);
        self.mark_scene_binding_component_replacement::<T>(
            entity,
            old.as_ref(),
            current_hierarchy_parent,
        );
        if was_present {
            self.trigger_component_lifecycle(LifecycleEventKind::Replace, entity, component_id);
        } else {
            self.trigger_component_lifecycle(LifecycleEventKind::Add, entity, component_id);
        }
        self.trigger_component_lifecycle(LifecycleEventKind::Insert, entity, component_id);
        Ok(!was_present)
    }

    fn validate_bundle_mobility_state(
        &self,
        entity: EntityId,
        parent: Option<EntityId>,
        mobility: Mobility,
    ) -> SceneResult<()> {
        match mobility {
            Mobility::Dynamic => self.validate_mobility_change(entity, mobility),
            Mobility::Static => {
                if let Some(parent) = parent {
                    if self.mobility(parent) == Some(Mobility::Dynamic) {
                        return Err(SceneError::StaticMobilityUnderDynamicParent {
                            entity,
                            parent,
                        });
                    }
                }
                Ok(())
            }
        }
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
            if self.fixed_component_ref::<T>(entity).is_none() {
                return None;
            }
            self.mark_component_changed_at_tick::<T>(entity, tick);
            self.mark_component_mutation::<T>(entity);
            self.mark_scene_binding_component_get_mut::<T>(entity);
            return self.fixed_component_mut::<T>(entity);
        }
        let component_id = self.registered_component_id::<T>()?;
        let internal = self.internal_entity(entity)?;
        if !self.component_storage.contains(component_id, internal) {
            return None;
        }
        self.mark_component_mutation::<T>(entity);
        self.mark_scene_binding_component_get_mut::<T>(entity);
        self.component_storage
            .get_mut_at_tick(component_id, internal, tick)
    }

    pub(in crate::scene) fn has_fixed_component_owner<T>(&self) -> bool
    where
        T: Component,
    {
        self.is_fixed_component_type::<T>()
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
            .ok_or_else(|| SceneError::missing_entity("remove component from", entity))?;
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
            if let Some(removed) = removed.as_ref() {
                self.record_removed_component::<T>(entity);
                self.mark_component_mutation::<T>(entity);
                self.mark_scene_binding_component_removal::<T>(entity, removed);
            }
            if removed.is_some() || removed_from_storage {
                if let Some(component_id) = component_id {
                    self.remove_component_from_entity_archetype(
                        entity,
                        component_id,
                        T::STORAGE_TYPE,
                    );
                }
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
        if let Some(removed) = removed.as_ref() {
            self.record_removed_component::<T>(entity);
            self.mark_component_mutation::<T>(entity);
            self.mark_scene_binding_component_removal::<T>(entity, removed);
            self.remove_component_from_entity_archetype(entity, component_id, T::STORAGE_TYPE);
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

    pub(crate) fn external_resource_id(&mut self, stable_id: &str) -> ResourceId {
        self.resource_registry.external_resource_id(stable_id)
    }

    pub fn registered_external_resource_id(&self, stable_id: &str) -> Option<ResourceId> {
        self.resource_registry
            .registered_external_resource_id(stable_id)
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
        let Some((resource, ticks, tick)) = self.resource_mut_with_ticks::<T>() else {
            return None;
        };

        ticks.set_changed(tick);

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
        let internal = self.internal_entity(entity).ok_or_else(|| {
            SceneError::missing_entity("insert dynamic component presence for", entity)
        })?;
        let tick = self.mutation_change_tick();
        let old = self.component_storage.insert_at_tick(
            component_id,
            crate::scene::ecs::StorageType::SparseSet,
            internal,
            DynamicComponentPresence,
            tick,
        )?;
        if old.is_none() {
            self.add_component_to_entity_archetype(
                entity,
                component_id,
                crate::scene::ecs::StorageType::SparseSet,
            );
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
            self.remove_component_from_entity_archetype(
                entity,
                component_id,
                crate::scene::ecs::StorageType::SparseSet,
            );
            self.bump_query_cache_revision();
        }
        Ok(())
    }

    /// Rebuilds the ECS storage projection for an already-authoritative fixed
    /// component without reporting a second observable world mutation.
    pub(in crate::scene::world) fn insert_rebuilt_fixed_component_presence<T>(
        &mut self,
        entity: EntityId,
        component: T,
    ) where
        T: Component,
    {
        self.insert_rebuilt_fixed_component_presence_with_archetype_update(entity, component, true);
    }

    pub(in crate::scene::world) fn insert_rebuilt_fixed_component_presence_without_archetype<T>(
        &mut self,
        entity: EntityId,
        component: T,
    ) where
        T: Component,
    {
        self.insert_rebuilt_fixed_component_presence_with_archetype_update(
            entity, component, false,
        );
    }

    fn insert_rebuilt_fixed_component_presence_with_archetype_update<T>(
        &mut self,
        entity: EntityId,
        component: T,
        update_archetype: bool,
    ) where
        T: Component,
    {
        let component_id = self.component_id::<T>();
        let was_present = self.contains_component_id(entity, component_id);
        let internal = self
            .internal_entity(entity)
            .expect("fixed component presence rebuild requires a registered entity");
        let tick = self.mutation_change_tick();
        let old = self
            .component_storage
            .insert_at_tick(component_id, T::STORAGE_TYPE, internal, component, tick)
            .expect("fixed component presence rebuild must preserve the registered storage type");
        debug_assert_eq!(old.is_some(), was_present);

        self.mark_component_derived_state_dirty::<T>();
        if !was_present {
            if update_archetype {
                self.add_component_to_entity_archetype(entity, component_id, T::STORAGE_TYPE);
                self.bump_query_cache_revision();
            }
            self.trigger_component_lifecycle(LifecycleEventKind::Add, entity, component_id);
        } else {
            self.trigger_component_lifecycle(LifecycleEventKind::Replace, entity, component_id);
        }
        self.trigger_component_lifecycle(LifecycleEventKind::Insert, entity, component_id);
    }

    pub(super) fn rebuild_typed_component_presence(&mut self) {
        self.component_registry = Default::default();
        self.rebuild_component_storage_projection();
    }

    pub(super) fn replace_derived_component<T>(&mut self, entity: EntityId, component: T)
    where
        T: Component,
    {
        let component_id = self.component_id::<T>();
        let internal = self
            .internal_entity(entity)
            .expect("derived component replacement requires a registered entity");
        let was_present = self.contains_component_id(entity, component_id);
        let tick = self.mutation_change_tick();
        self.component_storage
            .insert_at_tick(component_id, T::STORAGE_TYPE, internal, component, tick)
            .expect("derived component replacement must preserve the registered storage type");
        if !was_present {
            self.add_component_to_entity_archetype(entity, component_id, T::STORAGE_TYPE);
            self.bump_query_cache_revision();
        }
    }

    pub(super) fn rebuild_component_storage_projection(&mut self) {
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

    fn mark_component_mutation<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        self.advance_world_generation();
        if self.is_hierarchy_component_type(type_id) || self.is_active_component_type(type_id) {
            self.mark_inspection_subtree_fields_dirty(entity);
        } else {
            self.inspection_artifact_cache.mark_fields_dirty(entity);
        }
        if self.is_inspection_hierarchy_component_type(type_id) {
            self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
        }
        self.mark_component_derived_state_dirty::<T>();
    }

    fn mark_preflighted_bundle_component_mutation<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        if self.is_hierarchy_component_type(type_id) || self.is_active_component_type(type_id) {
            self.mark_inspection_subtree_fields_dirty(entity);
        } else {
            self.inspection_artifact_cache.mark_fields_dirty(entity);
        }
        if self.is_inspection_hierarchy_component_type(type_id) {
            self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
        }
        self.mark_component_derived_state_dirty::<T>();
    }

    pub(super) fn mark_inspection_subtree_fields_dirty(&self, root: EntityId) {
        for entity in self.subtree_entity_ids(root) {
            self.inspection_artifact_cache.mark_fields_dirty(entity);
        }
    }

    fn is_inspection_hierarchy_component_type(&self, type_id: std::any::TypeId) -> bool {
        type_id == std::any::TypeId::of::<crate::scene::components::Name>()
            || type_id == std::any::TypeId::of::<crate::scene::components::Hierarchy>()
            || type_id == std::any::TypeId::of::<crate::scene::components::ActiveSelf>()
            || type_id == std::any::TypeId::of::<crate::scene::components::CameraComponent>()
            || type_id == std::any::TypeId::of::<crate::scene::components::MeshRenderer>()
            || type_id == std::any::TypeId::of::<crate::scene::components::AmbientLight>()
            || type_id == std::any::TypeId::of::<crate::scene::components::DirectionalLight>()
            || type_id == std::any::TypeId::of::<crate::scene::components::PointLight>()
            || type_id == std::any::TypeId::of::<crate::scene::components::RectLight>()
            || type_id == std::any::TypeId::of::<crate::scene::components::SpotLight>()
    }
}
