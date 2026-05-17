use crate::core::math::Transform;

use super::World;
use crate::scene::components::{Hierarchy, LocalTransform, Mobility, NodeRecord};
use crate::scene::ecs::LifecycleEventKind;
use crate::scene::EntityId;

impl World {
    pub fn remove_entity(&mut self, entity: EntityId) -> bool {
        let Some(index) = self.entities.iter().position(|current| *current == entity) else {
            return false;
        };
        if let Some(internal) = self.internal_entity(entity) {
            let component_ids = self.component_storage.component_ids_for_entity(internal);
            for component_id in component_ids {
                self.trigger_component_lifecycle(LifecycleEventKind::Remove, entity, component_id);
                self.trigger_component_lifecycle(LifecycleEventKind::Despawn, entity, component_id);
            }
            let removed_components = self.component_storage.remove_entity(internal);
            for component_id in removed_components {
                if let Some((type_id, type_name)) =
                    self.component_registry.rust_type_for_id(component_id)
                {
                    self.removed_component_events
                        .push_type_id(type_id, type_name, entity);
                }
            }
        }
        self.unregister_stable_entity(entity);
        self.entities.remove(index);
        self.names.remove(&entity);
        self.kinds.remove(&entity);
        self.hierarchy.remove(&entity);
        self.local_transforms.remove(&entity);
        self.world_matrices.remove(&entity);
        self.cameras.remove(&entity);
        self.mesh_renderers.remove(&entity);
        self.sprite_2d.remove(&entity);
        self.mesh_2d.remove(&entity);
        self.directional_lights.remove(&entity);
        self.point_lights.remove(&entity);
        self.spot_lights.remove(&entity);
        self.rigid_bodies.remove(&entity);
        self.colliders.remove(&entity);
        self.joints.remove(&entity);
        self.animation_skeletons.remove(&entity);
        self.animation_players.remove(&entity);
        self.animation_sequence_players.remove(&entity);
        self.animation_graph_players.remove(&entity);
        self.animation_state_machine_players.remove(&entity);
        self.active_self.remove(&entity);
        self.active_in_hierarchy.remove(&entity);
        self.render_layer_masks.remove(&entity);
        self.mobility.remove(&entity);
        self.dynamic_components.remove(&entity);
        for child in self.hierarchy.values_mut() {
            if child.parent == Some(entity) {
                child.parent = None;
            }
        }
        self.refresh_stable_entity_locations();
        if self.active_camera == entity {
            self.active_camera = self
                .cameras
                .keys()
                .copied()
                .find(|camera| *camera != entity)
                .unwrap_or(0);
        }
        self.bump_query_cache_revision();
        self.mark_derived_state_dirty();
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
    ) -> Result<bool, String> {
        if !self.contains_entity(child) {
            return Err(format!("cannot reparent missing node {child}"));
        }
        if parent == Some(child) {
            return Err("node cannot become its own parent".to_string());
        }
        if let Some(parent) = parent {
            if !self.contains_entity(parent) {
                return Err(format!("cannot use missing parent node {parent}"));
            }
            if self.is_descendant(parent, child) {
                return Err("cannot create hierarchy cycle".to_string());
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
    ) -> Result<bool, String> {
        self.ensure_transform_mutable(entity)?;
        let Some(local) = self.local_transforms.get(&entity) else {
            return Err(format!("cannot update transform for missing node {entity}"));
        };
        if local.transform == transform {
            return Ok(false);
        }
        self.insert(entity, LocalTransform { transform })?;
        Ok(true)
    }

    pub(super) fn validate_mobility_change(
        &self,
        entity: EntityId,
        mobility: Mobility,
    ) -> Result<(), String> {
        match mobility {
            Mobility::Dynamic => {
                if self
                    .entities
                    .iter()
                    .copied()
                    .filter(|child| self.parent_of(*child) == Some(entity))
                    .any(|child| self.mobility(child) == Some(Mobility::Static))
                {
                    return Err(format!(
                        "cannot make node {entity} Dynamic while it owns Static children"
                    ));
                }
            }
            Mobility::Static => {
                if self
                    .parent_of(entity)
                    .is_some_and(|parent| self.mobility(parent) == Some(Mobility::Dynamic))
                {
                    return Err(format!(
                        "cannot make node {entity} Static under Dynamic parent"
                    ));
                }
            }
        }
        Ok(())
    }

    fn ensure_transform_mutable(&self, entity: EntityId) -> Result<(), String> {
        if !self.contains_entity(entity) {
            return Err(format!("cannot update transform for missing node {entity}"));
        }
        if self.mobility(entity) == Some(Mobility::Static) {
            return Err(format!(
                "cannot update transform for Static node {entity} at runtime"
            ));
        }
        Ok(())
    }

    fn validate_reparent(&self, child: EntityId, _parent: Option<EntityId>) -> Result<(), String> {
        if self.mobility(child) == Some(Mobility::Static) {
            return Err(format!(
                "cannot reparent Static node {child} during runtime mutation"
            ));
        }
        Ok(())
    }
}
