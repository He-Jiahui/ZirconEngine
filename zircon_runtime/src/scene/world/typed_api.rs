mod bundle_entry;
mod bundle_transaction;
mod component_mutation_effects;
mod component_row;
mod dynamic_component_presence;
pub(super) mod fixed_components;
mod projection_rebuild;

use std::collections::BTreeMap;

pub(super) use bundle_transaction::{
    BundleInsertionTransaction, DeferredBundleTransactionArtifact,
};
use dynamic_component_presence::DynamicComponentPresence;

use crate::scene::ecs::{
    ArchetypeSignature, Component, ComponentId, ComponentRemoveResult, ComponentTicks,
    LifecycleEventKind, Resource, ResourceId, StorageError, StorageType,
};
use crate::scene::{components::Mobility, EntityId};

use super::{SceneError, SceneResult, World};

impl World {
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

    pub(in crate::scene) fn component_registry_generation(&self) -> u64 {
        self.component_registry.generation()
    }

    pub fn component_count_for_id(&self, component_id: ComponentId) -> usize {
        match self
            .component_registry
            .descriptor(component_id)
            .map(|descriptor| descriptor.storage_type)
        {
            Some(StorageType::Table) => self.archetype_index.component_len(component_id),
            Some(StorageType::SparseSet) => self.component_storage.len_for_component(component_id),
            None => 0,
        }
    }

    pub fn contains_component_id(&self, entity: EntityId, component_id: ComponentId) -> bool {
        let Some(internal) = self.internal_entity(entity) else {
            return false;
        };

        match self
            .component_registry
            .descriptor(component_id)
            .map(|descriptor| descriptor.storage_type)
        {
            Some(StorageType::Table) => self
                .entity_archetype_signature(entity)
                .is_some_and(|signature| signature.contains(component_id)),
            Some(StorageType::SparseSet) => self.component_storage.contains(component_id, internal),
            None => false,
        }
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
        self.validate_fixed_component(entity, &component)?;
        let internal = self
            .internal_entity(entity)
            .ok_or_else(|| SceneError::missing_entity("insert component on", entity))?;
        self.component_storage
            .validate_insert::<T>(component_id, T::STORAGE_TYPE)?;
        let old = match T::STORAGE_TYPE {
            StorageType::Table if was_present => {
                let location = self
                    .internal_entity_location(entity)
                    .expect("present table component must retain an entity location")
                    .location;
                let old = self
                    .archetype_index
                    .replace(
                        location.archetype_id,
                        location.table_row,
                        component_id,
                        Box::new(component),
                        tick,
                    )
                    .ok_or(StorageError::ComponentTypeMismatch { component_id })?;
                Some(
                    old.downcast::<T>()
                        .map(|value| *value)
                        .map_err(|_| StorageError::ComponentTypeMismatch { component_id })?,
                )
            }
            StorageType::Table => {
                let signature = self
                    .entity_archetype_signature(entity)
                    .expect("registered entity must own an archetype signature")
                    .with_component_added(component_id, StorageType::Table);
                let mut updates = BTreeMap::new();
                updates.insert(
                    component_id,
                    Some((
                        Box::new(component) as Box<dyn std::any::Any + Send + Sync>,
                        ComponentTicks::new(tick),
                    )),
                );
                let replaced = self.transition_entity_archetype_row(entity, signature, updates);
                debug_assert!(replaced.is_some_and(|values| values.is_empty()));
                None
            }
            StorageType::SparseSet => {
                let old = self.component_storage.insert_at_tick(
                    component_id,
                    StorageType::SparseSet,
                    internal,
                    component,
                    tick,
                )?;
                if old.is_none() {
                    let signature = self
                        .entity_archetype_signature(entity)
                        .expect("registered entity must own an archetype signature")
                        .with_component_added(component_id, StorageType::SparseSet);
                    self.transition_entity_archetype_row(entity, signature, BTreeMap::new());
                }
                old
            }
        };

        if let Some(current_parent) = current_hierarchy_parent {
            let previous_parent = old
                .as_ref()
                .and_then(Self::hierarchy_parent_from_component)
                .unwrap_or(None);
            self.update_hierarchy_mutation_index(entity, previous_parent, current_parent);
        }

        self.mark_component_mutation::<T>(entity);
        self.mark_scene_binding_component_replacement::<T>(
            entity,
            old.as_ref(),
            current_hierarchy_parent,
        );
        if !was_present {
            self.bump_lifecycle_visibility_revision();
        }
        if was_present {
            self.trigger_component_lifecycle(LifecycleEventKind::Replace, entity, component_id);
        } else {
            self.trigger_component_lifecycle(LifecycleEventKind::Add, entity, component_id);
        }
        self.trigger_component_lifecycle(LifecycleEventKind::Insert, entity, component_id);
        Ok(old)
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
        let component_id = self.registered_component_id::<T>()?;
        let internal = self.internal_entity(entity)?;
        match T::STORAGE_TYPE {
            StorageType::Table => {
                let location = self
                    .entity_registry
                    .location_for_internal(internal)
                    .ok()?
                    .location;
                self.archetype_index
                    .get(location.archetype_id, location.table_row, component_id)
            }
            StorageType::SparseSet => self.component_storage.get(component_id, internal),
        }
    }

    pub fn get_mut<T>(&mut self, entity: EntityId) -> Option<&mut T>
    where
        T: Component,
    {
        let tick = self.mutation_change_tick();
        let component_id = self.registered_component_id::<T>()?;
        let internal = self.internal_entity(entity)?;
        if !self.contains_component_id(entity, component_id) {
            return None;
        }
        self.mark_component_mutation::<T>(entity);
        self.mark_scene_binding_component_get_mut::<T>(entity);
        match T::STORAGE_TYPE {
            StorageType::Table => {
                let location = self
                    .entity_registry
                    .location_for_internal(internal)
                    .ok()?
                    .location;
                self.archetype_index.get_mut_at_tick(
                    location.archetype_id,
                    location.table_row,
                    component_id,
                    tick,
                )
            }
            StorageType::SparseSet => {
                self.component_storage
                    .get_mut_at_tick(component_id, internal, tick)
            }
        }
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
        let Some(component_id) = component_id else {
            return Ok(None);
        };
        if self.contains_component_id(entity, component_id) {
            self.trigger_component_lifecycle(LifecycleEventKind::Remove, entity, component_id);
        }
        let removed = match T::STORAGE_TYPE {
            StorageType::Table => {
                let signature = self
                    .entity_archetype_signature(entity)
                    .expect("registered entity must own an archetype signature")
                    .with_component_removed(component_id, StorageType::Table);
                let mut updates = BTreeMap::new();
                updates.insert(component_id, None);
                let removed = self
                    .transition_entity_archetype_row(entity, signature, updates)
                    .and_then(|mut values| values.remove(&component_id));
                match removed {
                    Some((value, _)) => Some(
                        value
                            .downcast::<T>()
                            .map(|value| *value)
                            .map_err(|_| StorageError::ComponentTypeMismatch { component_id })?,
                    ),
                    None => None,
                }
            }
            StorageType::SparseSet => {
                let removed = match self.component_storage.remove::<T>(component_id, internal)? {
                    Some(ComponentRemoveResult { value }) => Some(value),
                    None => None,
                };
                if removed.is_some() {
                    let signature = self
                        .entity_archetype_signature(entity)
                        .expect("registered entity must own an archetype signature")
                        .with_component_removed(component_id, StorageType::SparseSet);
                    self.transition_entity_archetype_row(entity, signature, BTreeMap::new());
                }
                removed
            }
        };
        if let Some(removed) = removed.as_ref() {
            if let Some(previous_parent) = Self::hierarchy_parent_from_component(removed) {
                self.update_hierarchy_mutation_index(entity, previous_parent, None);
            }
            self.record_removed_component::<T>(entity);
            self.mark_component_mutation::<T>(entity);
            self.mark_scene_binding_component_removal::<T>(entity, removed);
            self.bump_lifecycle_visibility_revision();
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
        let removed = self.resources.remove::<T>()?;
        self.mutation_change_tick();
        Some(removed)
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

    pub(crate) fn entity_ids_for_query(&self) -> super::StableWorldEntityIter<'_> {
        self.stable_entity_ids()
    }

    pub(super) fn insert_dynamic_component_presence(
        &mut self,
        entity: EntityId,
        component_type_id: &str,
    ) -> SceneResult<()> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity(
                "insert dynamic component presence for",
                entity,
            ));
        }
        let component_id = self
            .component_registry
            .dynamic_component_id(component_type_id);
        let mut row = self.begin_component_row(entity);
        self.stage_component_row_value_with_id(&mut row, component_id, DynamicComponentPresence);
        if self.commit_component_row(entity, row, false) {
            self.bump_lifecycle_visibility_revision();
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
            let signature = self
                .entity_archetype_signature(entity)
                .expect("dynamic component target must retain its archetype signature")
                .with_component_removed(component_id, StorageType::SparseSet);
            self.transition_entity_archetype_row(entity, signature, BTreeMap::new());
            self.bump_lifecycle_visibility_revision();
        }
        Ok(())
    }

    pub(super) fn rebuild_typed_component_presence(&mut self) {
        let persistent_entity_core = self.persistent_entity_core_component_snapshot();
        let persistent_scene_render = self.persistent_scene_render_component_snapshot();
        let runtime_only_post_process = self.runtime_only_post_process_component_snapshot();
        let persistent_physics = self.persistent_physics_component_snapshot();
        let persistent_lighting = self.persistent_lighting_component_snapshot();
        let persistent_render_2d = self.persistent_render_2d_component_snapshot();
        let persistent_animation_runtime = self.persistent_animation_runtime_component_snapshot();
        self.component_registry = Default::default();
        self.rebuild_component_storage_projection_with_owned_components(
            persistent_entity_core,
            persistent_scene_render,
            runtime_only_post_process,
            persistent_physics,
            persistent_lighting,
            persistent_render_2d,
            persistent_animation_runtime,
        );
    }

    pub(super) fn replace_derived_component<T>(&mut self, entity: EntityId, component: T)
    where
        T: Component,
    {
        let component_id = self.component_id::<T>();
        let was_present = self.contains_component_id(entity, component_id);
        let tick = self.mutation_change_tick();
        match T::STORAGE_TYPE {
            StorageType::Table if was_present => {
                let location = self
                    .internal_entity_location(entity)
                    .expect("derived component replacement requires an archetype row")
                    .location;
                let replaced = self.archetype_index.replace(
                    location.archetype_id,
                    location.table_row,
                    component_id,
                    Box::new(component),
                    tick,
                );
                debug_assert!(replaced.is_some());
            }
            StorageType::Table => {
                let signature = self
                    .entity_archetype_signature(entity)
                    .expect("derived component replacement requires an archetype signature")
                    .with_component_added(component_id, StorageType::Table);
                let mut updates = BTreeMap::new();
                updates.insert(
                    component_id,
                    Some((
                        Box::new(component) as Box<dyn std::any::Any + Send + Sync>,
                        ComponentTicks::new(tick),
                    )),
                );
                self.transition_entity_archetype_row(entity, signature, updates);
            }
            StorageType::SparseSet => {
                let internal = self
                    .internal_entity(entity)
                    .expect("derived component replacement requires a registered entity");
                let old = self
                    .component_storage
                    .insert_at_tick(
                        component_id,
                        StorageType::SparseSet,
                        internal,
                        component,
                        tick,
                    )
                    .expect("derived component replacement must preserve sparse storage type");
                if old.is_none() {
                    let signature = self
                        .entity_archetype_signature(entity)
                        .expect("derived component replacement requires an archetype signature")
                        .with_component_added(component_id, StorageType::SparseSet);
                    self.transition_entity_archetype_row(entity, signature, BTreeMap::new());
                }
            }
        }
        if !was_present {
            self.bump_lifecycle_visibility_revision();
        }
    }

    fn mark_component_mutation<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        self.advance_world_generation();
        self.invalidate_world_component_type(std::any::type_name::<T>());
        if self.is_hierarchy_component_type(type_id) || self.is_active_component_type(type_id) {
            self.mark_inspection_subtree_fields_dirty(entity);
        } else {
            self.inspection_artifact_cache.mark_fields_dirty(entity);
        }
        if self.is_inspection_hierarchy_component_type(type_id) {
            if type_id == std::any::TypeId::of::<crate::scene::components::Name>() {
                self.inspection_artifact_cache
                    .mark_hierarchy_name_dirty(entity);
            } else {
                self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
            }
        }
        self.mark_component_derived_state_dirty::<T>();
    }

    fn mark_preflighted_bundle_component_mutation<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        self.invalidate_world_component_type(std::any::type_name::<T>());
        if self.is_hierarchy_component_type(type_id) || self.is_active_component_type(type_id) {
            self.mark_inspection_subtree_fields_dirty(entity);
        } else {
            self.inspection_artifact_cache.mark_fields_dirty(entity);
        }
        if self.is_inspection_hierarchy_component_type(type_id) {
            if type_id == std::any::TypeId::of::<crate::scene::components::Name>() {
                self.inspection_artifact_cache
                    .mark_hierarchy_name_dirty(entity);
            } else {
                self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
            }
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
