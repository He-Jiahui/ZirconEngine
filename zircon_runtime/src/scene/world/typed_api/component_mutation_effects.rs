use crate::scene::ecs::{Component, ComponentMutationRecord};
use crate::scene::{EntityId, World};

use super::HierarchyMutationMode;

impl World {
    /// Query 可变访问的副作用入口；普通引用借出前仍即时更新绑定、派生状态和渲染脏日志。
    ///
    /// # Safety
    /// The caller must hold the World query loan, exclusive access to the affected
    /// ordinary metadata leaves, and compatible reads of hierarchy/registry state.
    /// Live items may retain row cells and the shared sink leaf, never their parents.
    pub(crate) unsafe fn mark_query_component_mutation<T>(world: *mut Self, entity: EntityId)
    where
        T: Component,
    {
        unsafe {
            Self::mark_component_mutation_unchecked::<T>(
                world,
                entity,
                HierarchyMutationMode::Unchecked,
            );
            Self::mark_scene_binding_component_get_mut_unchecked::<T>(world, entity);
        }
    }

    /// Common eager owner for an exclusive World entry and a granted raw query fetch.
    ///
    /// # Safety
    /// The caller must uniquely own all mutated clock/generation/frontier/binding
    /// metadata and permit the scoped Hierarchy reads. No live reference may cover
    /// those leaves or their World/dirty-state parents. A retained mutation recorder
    /// covers only the shared sink leaf, which this function does not mutably borrow.
    pub(super) unsafe fn mark_component_mutation_unchecked<T>(
        world: *mut Self,
        entity: EntityId,
        hierarchy_mutation: HierarchyMutationMode,
    ) where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        let hierarchy = type_id == std::any::TypeId::of::<crate::scene::components::Hierarchy>();
        let active = type_id == std::any::TypeId::of::<crate::scene::components::ActiveSelf>();
        unsafe {
            Self::advance_world_generation_unchecked(world);
            (&*std::ptr::addr_of!((*world).world_sync_subscriptions))
                .invalidate_component_type(std::any::type_name::<T>());

            // Collect owned ids before mutating cache/frontier leaves. The temporary
            // shared World is not the origin of a returned row or recorder reference.
            let subtree = if hierarchy || active {
                Some((&*world).subtree_entity_ids(entity))
            } else {
                None
            };
            let inspection_hierarchy = (&*world).is_inspection_hierarchy_component_type(type_id);
            let cache = &*std::ptr::addr_of!((*world).inspection_artifact_cache);
            if let Some(subtree) = subtree {
                for affected in subtree {
                    cache.mark_fields_dirty(affected);
                }
            } else {
                cache.mark_fields_dirty(entity);
            }
            if inspection_hierarchy {
                if type_id == std::any::TypeId::of::<crate::scene::components::Name>() {
                    cache.mark_hierarchy_name_dirty(entity);
                } else {
                    cache.mark_hierarchy_rows_dirty();
                }
            }
            super::super::dirty_state::DerivedStateDirty::mark_component_at_unchecked(
                std::ptr::addr_of_mut!((*world).derived_state_dirty),
                entity,
                type_id,
                hierarchy && hierarchy_mutation == HierarchyMutationMode::Checked,
            );
        }
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
        // Ordinary authored/prevalidated calls retain their exclusive World role.
        unsafe { Self::mark_scene_binding_component_get_mut_unchecked::<T>(self, entity) }
    }

    /// # Safety
    /// The caller must own the binding/index leaves exclusively and permit the
    /// existing Hierarchy ancestor reads. Live items may not cover those owners.
    unsafe fn mark_scene_binding_component_get_mut_unchecked<T>(world: *mut Self, entity: EntityId)
    where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        unsafe {
            if type_id == std::any::TypeId::of::<crate::scene::components::Name>() {
                Self::advance_scene_binding_generation_for_name_unchecked(world, entity);
            } else if type_id == std::any::TypeId::of::<crate::scene::components::Hierarchy>() {
                // The mutable reference does not reveal its eventual parent. Structured
                // reparenting stays incremental; the exclusive escape hatch stays correct.
                (&mut *std::ptr::addr_of_mut!((*world).hierarchy_mutation_index)).mark_dirty();
                Self::invalidate_all_scene_binding_generations_unchecked(world);
            }
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
