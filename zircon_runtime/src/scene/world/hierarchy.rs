use crate::core::math::Transform;

use super::{transform_validation::validate_transform_for_write, SceneError, SceneResult, World};
use crate::scene::components::{Hierarchy, LocalTransform, Mobility, NodeRecord};
use crate::scene::ecs::LifecycleEventKind;
use crate::scene::EntityId;

impl World {
    pub fn remove_entity(&mut self, entity: EntityId) -> bool {
        let mut index = 0_usize;
        while index < self.entities.len() {
            if self.entities[index] == entity {
                break;
            }
            index += 1;
        }
        if index == self.entities.len() {
            return false;
        }
        let removed_kind = self.kinds.get(&entity).copied();
        let removed_parent = self.parent_of(entity);
        if let Some(internal) = self.internal_entity(entity) {
            let component_ids = self.entity_archetype_component_ids(entity);
            for component_id in &component_ids {
                self.trigger_component_lifecycle(LifecycleEventKind::Remove, entity, *component_id);
                self.trigger_component_lifecycle(
                    LifecycleEventKind::Despawn,
                    entity,
                    *component_id,
                );
            }
            let removed_components = self
                .component_storage
                .remove_entity_components(internal, &component_ids);
            for component_id in removed_components {
                if let Some((type_id, type_name)) =
                    self.component_registry.rust_type_for_id(component_id)
                {
                    self.removed_component_events
                        .push_type_id(type_id, type_name, entity);
                }
            }
        }
        self.observers.remove_entity_observers(entity);
        self.remove_entity_from_archetype(entity);
        self.unregister_stable_entity(entity);
        self.entities.remove(index);
        self.names.remove(&entity);
        self.kinds.remove(&entity);
        if let Some(kind) = removed_kind {
            self.record_node_kind_removed(kind);
        }
        self.hierarchy.remove(&entity);
        self.local_transforms.remove(&entity);
        self.cameras.remove(&entity);
        self.mesh_renderers.remove(&entity);
        self.sprite_2d.remove(&entity);
        self.mesh_2d.remove(&entity);
        self.directional_lights.remove(&entity);
        self.point_lights.remove(&entity);
        self.spot_lights.remove(&entity);
        self.post_process_settings.remove(&entity);
        self.post_process_volumes.remove(&entity);
        self.rigid_bodies.remove(&entity);
        self.colliders.remove(&entity);
        self.joints.remove(&entity);
        self.animation_skeletons.remove(&entity);
        self.animation_players.remove(&entity);
        self.animation_sequence_players.remove(&entity);
        self.animation_graph_players.remove(&entity);
        self.animation_state_machine_players.remove(&entity);
        self.active_self.remove(&entity);
        self.render_layer_masks.remove(&entity);
        self.mobility.remove(&entity);
        if let Some(components) = self.dynamic_components.remove(&entity) {
            for component_id in components.keys() {
                self.advance_dynamic_component_generation(component_id);
            }
        }
        let orphaned_children = self
            .hierarchy
            .iter()
            .filter_map(|(child, hierarchy)| (hierarchy.parent == Some(entity)).then_some(*child))
            .collect::<Vec<_>>();
        for child in orphaned_children {
            if let Some(hierarchy) = self.hierarchy.get_mut(&child) {
                hierarchy.parent = None;
            }
            self.mark_inspection_subtree_fields_dirty(child);
        }
        if self.active_camera == entity {
            self.active_camera = 0;
            for camera in self.cameras.keys().copied() {
                if camera != entity {
                    self.active_camera = camera;
                    break;
                }
            }
        }
        self.bump_query_cache_revision();
        self.mark_derived_state_dirty();
        self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
        self.advance_world_generation();
        self.advance_scene_binding_generations_for_removal(entity, removed_parent);
        true
    }

    pub fn remove_entity_recursive(&mut self, entity: EntityId) -> Vec<NodeRecord> {
        let records = self.subtree_records(entity);
        for record in records.iter().rev() {
            let _ = self.remove_entity(record.id);
        }
        records
    }

    pub fn subtree_records(&self, entity: EntityId) -> Vec<NodeRecord> {
        let mut records = Vec::new();
        self.collect_subtree_records(entity, &mut records);
        records
    }

    pub fn set_parent_checked(
        &mut self,
        child: EntityId,
        parent: Option<EntityId>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(child) {
            return Err(SceneError::missing_entity("reparent", child));
        }
        if parent == Some(child) {
            return Err(SceneError::EntityCannotParentItself { entity: child });
        }
        if let Some(parent) = parent {
            if !self.contains_entity(parent) {
                return Err(SceneError::MissingParent { child, parent });
            }
            if self.is_descendant(parent, child) {
                return Err(SceneError::HierarchyCycle { child, parent });
            }
        }
        self.validate_reparent(child, parent)?;
        if self.parent_of(child) == parent {
            return Ok(false);
        }
        self.insert(child, Hierarchy { parent })?;
        Ok(true)
    }

    pub fn update_transform(
        &mut self,
        entity: EntityId,
        transform: Transform,
    ) -> SceneResult<bool> {
        self.ensure_transform_mutable(entity)?;
        let Some(local) = self.local_transforms.get(&entity) else {
            return Err(SceneError::MissingRequiredComponent {
                operation: "update transform",
                entity,
                component: "LocalTransform",
            });
        };
        if local.transform == transform {
            return Ok(false);
        }
        validate_transform_for_write(entity, transform)?;
        self.insert(entity, LocalTransform { transform })?;
        Ok(true)
    }

    pub(super) fn validate_mobility_change(
        &self,
        entity: EntityId,
        mobility: Mobility,
    ) -> SceneResult<()> {
        match mobility {
            Mobility::Dynamic => {
                for child in self.entities.iter().copied() {
                    if self.parent_of(child) != Some(entity) {
                        continue;
                    }
                    if self.mobility(child) == Some(Mobility::Static) {
                        return Err(SceneError::DynamicMobilityWithStaticChildren { entity });
                    }
                }
            }
            Mobility::Static => {
                if let Some(parent) = self.parent_of(entity) {
                    if self.mobility(parent) == Some(Mobility::Dynamic) {
                        return Err(SceneError::StaticMobilityUnderDynamicParent {
                            entity,
                            parent,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn ensure_transform_mutable(&self, entity: EntityId) -> SceneResult<()> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("update transform for", entity));
        }
        if self.mobility(entity) == Some(Mobility::Static) {
            return Err(SceneError::StaticTransformMutation { entity });
        }
        Ok(())
    }

    fn validate_reparent(&self, child: EntityId, _parent: Option<EntityId>) -> SceneResult<()> {
        if self.mobility(child) == Some(Mobility::Static) {
            return Err(SceneError::StaticReparentMutation { entity: child });
        }
        Ok(())
    }
}
