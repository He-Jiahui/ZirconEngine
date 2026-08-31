use crate::scene::ecs::{Component, ComponentMutationRecord};
use crate::scene::{EntityId, World};

use super::HierarchyMutationMode;

impl World {
    pub(crate) fn mark_query_component_mutation<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        self.mark_component_mutation::<T>(entity, HierarchyMutationMode::Unchecked);
        self.mark_scene_binding_component_get_mut::<T>(entity);
    }

    pub(in crate::scene::world) fn apply_deferred_component_mutation(
        &mut self,
        mutation: ComponentMutationRecord,
    ) {
        let entity = mutation.entity();
        let component_type = mutation.component_type();
        self.advance_world_generation();
        self.invalidate_world_component_type(mutation.component_type_name());

        if self.is_hierarchy_component_type(component_type)
            || self.is_active_component_type(component_type)
        {
            self.mark_inspection_subtree_fields_dirty(entity);
        } else {
            self.inspection_artifact_cache.mark_fields_dirty(entity);
        }
        if self.is_inspection_hierarchy_component_type(component_type) {
            if component_type == std::any::TypeId::of::<crate::scene::components::Name>() {
                self.inspection_artifact_cache
                    .mark_hierarchy_name_dirty(entity);
            } else {
                self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
            }
        }

        if component_type == std::any::TypeId::of::<crate::scene::components::Name>() {
            self.advance_scene_binding_generation_for_name(entity);
        } else if self.is_hierarchy_component_type(component_type) {
            self.mark_hierarchy_mutation_index_dirty();
            self.invalidate_all_scene_binding_generations();
        }
        self.mark_component_derived_state_dirty_at_type(entity, component_type);
    }

    pub(super) fn mark_scene_binding_component_replacement<T>(
        &mut self,
        entity: EntityId,
        previous: Option<&T>,
        current_hierarchy_parent: Option<Option<EntityId>>,
    ) where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        if type_id == std::any::TypeId::of::<crate::scene::components::Name>() {
            self.advance_scene_binding_generation_for_name(entity);
        } else if type_id == std::any::TypeId::of::<crate::scene::components::Hierarchy>() {
            let previous_parent = previous
                .and_then(Self::hierarchy_parent_from_component)
                .unwrap_or(None);
            self.advance_scene_binding_generations_for_reparent(
                entity,
                previous_parent,
                current_hierarchy_parent.unwrap_or(None),
            );
        }
    }

    /// Bundle publication has already moved the erased storage value, so its
    /// binding invalidation receives the preflighted hierarchy parent instead
    /// of a borrowed previous component.
    pub(super) fn mark_preflighted_bundle_component_scene_binding_replacement<T>(
        &mut self,
        entity: EntityId,
        previous_hierarchy_parent: Option<EntityId>,
        current_hierarchy_parent: Option<Option<EntityId>>,
    ) where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        if type_id == std::any::TypeId::of::<crate::scene::components::Name>() {
            self.advance_scene_binding_generation_for_name(entity);
        } else if type_id == std::any::TypeId::of::<crate::scene::components::Hierarchy>() {
            self.advance_scene_binding_generations_for_reparent(
                entity,
                previous_hierarchy_parent,
                current_hierarchy_parent.unwrap_or(None),
            );
        }
    }

    pub(super) fn mark_scene_binding_component_removal<T>(&mut self, entity: EntityId, previous: &T)
    where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        if type_id == std::any::TypeId::of::<crate::scene::components::Name>() {
            self.advance_scene_binding_generation_for_name(entity);
        } else if type_id == std::any::TypeId::of::<crate::scene::components::Hierarchy>() {
            self.advance_scene_binding_generations_for_reparent(
                entity,
                Self::hierarchy_parent_from_component(previous).unwrap_or(None),
                None,
            );
        }
    }

    pub(super) fn mark_scene_binding_component_get_mut<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        if type_id == std::any::TypeId::of::<crate::scene::components::Name>() {
            self.advance_scene_binding_generation_for_name(entity);
        } else if type_id == std::any::TypeId::of::<crate::scene::components::Hierarchy>() {
            // The raw mutable reference does not reveal its eventual parent. Structured
            // reparenting stays incremental; this escape hatch must remain correct.
            self.mark_hierarchy_mutation_index_dirty();
            self.invalidate_all_scene_binding_generations();
        }
    }

    pub(super) fn hierarchy_parent_from_component<T>(component: &T) -> Option<Option<EntityId>>
    where
        T: Component,
    {
        if std::any::TypeId::of::<T>()
            != std::any::TypeId::of::<crate::scene::components::Hierarchy>()
        {
            return None;
        }
        let hierarchy = (component as &dyn std::any::Any)
            .downcast_ref::<crate::scene::components::Hierarchy>()?;
        Some(hierarchy.parent)
    }

    pub(super) fn mark_component_derived_state_dirty<T>(&mut self)
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

    pub(super) fn mark_component_derived_state_dirty_at<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        self.mark_component_derived_state_dirty_at_type(entity, std::any::TypeId::of::<T>());
    }

    fn mark_component_derived_state_dirty_at_type(
        &mut self,
        entity: EntityId,
        type_id: std::any::TypeId,
    ) {
        if self.is_hierarchy_component_type(type_id) {
            self.mark_hierarchy_dirty_at(entity);
        } else if self.is_transform_component_type(type_id) {
            self.mark_transform_dirty_at(entity);
        } else if self.is_active_component_type(type_id) {
            self.mark_active_state_dirty_at(entity);
        } else {
            self.mark_node_cache_dirty_at(entity);
        }
    }

    pub(super) fn mark_checked_hierarchy_derived_state_dirty_at(&mut self, entity: EntityId) {
        self.mark_checked_hierarchy_dirty_at(entity);
    }

    pub(super) fn is_hierarchy_component_type(&self, type_id: std::any::TypeId) -> bool {
        type_id == std::any::TypeId::of::<crate::scene::components::Hierarchy>()
    }

    pub(super) fn is_transform_component_type(&self, type_id: std::any::TypeId) -> bool {
        type_id == std::any::TypeId::of::<crate::scene::components::LocalTransform>()
    }

    pub(super) fn is_active_component_type(&self, type_id: std::any::TypeId) -> bool {
        type_id == std::any::TypeId::of::<crate::scene::components::ActiveSelf>()
    }
}
