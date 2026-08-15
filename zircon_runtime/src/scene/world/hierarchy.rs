use crate::core::math::Transform;

use super::{transform_validation::validate_transform_for_write, SceneError, SceneResult, World};
use crate::scene::components::{Hierarchy, LocalTransform, Mobility, NodeRecord};
use crate::scene::ecs::LifecycleEventKind;
use crate::scene::EntityId;
use zircon_runtime_interface::world_sync::WorldFact;

impl World {
    pub fn remove_entity(&mut self, entity: EntityId) -> SceneResult<()> {
        if !self.entity_dense_rows.contains_key(&entity) {
            return Err(SceneError::missing_entity("remove", entity));
        }
        let _hierarchy_index_rebuild_rows = self.ensure_hierarchy_mutation_index_current();
        let removed_kind = self.kinds.get(&entity).copied();
        let removed_parent = self.parent_of(entity);
        let removed_order = self
            .stable_entity_order(entity)
            .expect("registered entity must retain stable order");
        let orphaned_children = self.direct_child_entity_ids(entity);
        for child in orphaned_children {
            self.insert(child, Hierarchy { parent: None })?;
            self.mark_inspection_subtree_fields_dirty(child);
        }
        self.record_world_fact(WorldFact::Despawned(entity));
        if let Some(internal) = self.internal_entity(entity) {
            let component_ids = self.entity_archetype_component_ids(entity);
            for component_id in &component_ids {
                self.trigger_component_lifecycle(LifecycleEventKind::Remove, entity, *component_id);
                self.trigger_component_lifecycle(
                    LifecycleEventKind::Despawn,
                    entity,
                    *component_id,
                );
                if let Some((type_id, type_name)) =
                    self.component_registry.rust_type_for_id(*component_id)
                {
                    self.removed_component_events
                        .push_type_id(type_id, type_name, entity);
                }
            }
            self.component_storage
                .remove_entity_components(internal, &component_ids);
        }
        self.observers.remove_entity_observers(entity);
        self.remove_entity_from_archetype(entity);
        self.remove_hierarchy_mutation_index_entry(entity, removed_order, removed_parent);
        self.unregister_stable_entity(entity);
        let removed = self.remove_entity_from_dense_storage(entity);
        debug_assert!(removed);
        self.kinds.remove(&entity);
        if let Some(kind) = removed_kind {
            self.record_node_kind_removed(kind);
        }
        if let Some(components) = self.dynamic_components.remove(&entity) {
            for component_id in components.keys() {
                self.advance_dynamic_component_generation(component_id);
            }
        }
        if self.active_camera == entity {
            self.active_camera = self.first_stable_camera_entity().unwrap_or(0);
        }
        self.bump_lifecycle_visibility_revision();
        self.mark_derived_state_dirty();
        self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
        self.advance_world_generation();
        self.advance_scene_binding_generations_for_removal(entity, removed_parent);
        Ok(())
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
        self.record_world_fact(WorldFact::Reparented {
            entity: child,
            new_parent: parent,
        });
        self.insert(child, Hierarchy { parent })?;
        self.record_world_fact(WorldFact::Reparented {
            entity: child,
            new_parent: parent,
        });
        Ok(true)
    }

    pub fn update_transform(
        &mut self,
        entity: EntityId,
        transform: Transform,
    ) -> SceneResult<bool> {
        self.ensure_transform_mutable(entity)?;
        let Some(local) = self.get::<LocalTransform>(entity) else {
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
                for child in self.stable_entity_ids() {
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
