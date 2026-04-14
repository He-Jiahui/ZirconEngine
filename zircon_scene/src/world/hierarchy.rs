use zircon_math::Transform;

use super::World;
use crate::components::NodeRecord;
use crate::EntityId;

impl World {
    pub fn remove_entity(&mut self, entity: EntityId) -> bool {
        let Some(index) = self.entities.iter().position(|current| *current == entity) else {
            return false;
        };
        self.entities.remove(index);
        self.names.remove(&entity);
        self.kinds.remove(&entity);
        self.hierarchy.remove(&entity);
        self.local_transforms.remove(&entity);
        self.world_transforms.remove(&entity);
        self.cameras.remove(&entity);
        self.mesh_renderers.remove(&entity);
        self.directional_lights.remove(&entity);
        self.active.remove(&entity);
        for child in self.hierarchy.values_mut() {
            if child.parent == Some(entity) {
                child.parent = None;
            }
        }
        if self.selected_entity == Some(entity) {
            self.selected_entity = None;
        }
        if self.active_camera == entity {
            self.active_camera = self
                .cameras
                .keys()
                .copied()
                .find(|camera| *camera != entity)
                .unwrap_or(0);
        }
        self.rebuild_derived_state();
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

    pub fn set_parent(&mut self, child: EntityId, parent: Option<EntityId>) {
        if let Some(hierarchy) = self.hierarchy.get_mut(&child) {
            hierarchy.parent = parent;
            self.rebuild_derived_state();
        }
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
        if self.parent_of(child) == parent {
            return Ok(false);
        }
        self.set_parent(child, parent);
        Ok(true)
    }

    pub fn update_transform(&mut self, entity: EntityId, transform: Transform) {
        if let Some(local) = self.local_transforms.get_mut(&entity) {
            local.transform = transform;
            self.rebuild_derived_state();
        }
    }
}
