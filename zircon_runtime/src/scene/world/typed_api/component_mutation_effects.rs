use crate::scene::ecs::Component;
use crate::scene::{EntityId, World};

impl World {
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
