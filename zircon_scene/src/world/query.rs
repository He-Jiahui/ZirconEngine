use zircon_math::Transform;

use super::World;
use crate::components::{SceneNode, Schedule};
use crate::EntityId;

impl World {
    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }

    pub fn contains_entity(&self, entity: EntityId) -> bool {
        self.entities.contains(&entity)
    }

    pub fn camera_count(&self) -> usize {
        self.cameras.len()
    }

    pub fn parent_of(&self, entity: EntityId) -> Option<EntityId> {
        self.hierarchy
            .get(&entity)
            .and_then(|hierarchy| hierarchy.parent)
    }

    pub fn active_camera(&self) -> EntityId {
        self.active_camera
    }

    pub fn set_active_camera(&mut self, entity: EntityId) {
        if self.cameras.contains_key(&entity) {
            self.active_camera = entity;
        }
    }

    pub fn set_selected(&mut self, entity: Option<EntityId>) {
        self.selected_entity = entity;
    }

    pub fn selected_node(&self) -> Option<EntityId> {
        self.selected_entity
    }

    pub fn nodes(&self) -> &[SceneNode] {
        &self.node_cache
    }

    pub fn find_node(&self, entity: EntityId) -> Option<&SceneNode> {
        self.node_cache.iter().find(|node| node.id == entity)
    }

    pub fn world_transform(&self, entity: EntityId) -> Option<Transform> {
        self.world_transforms
            .get(&entity)
            .map(|transform| transform.transform)
    }
}
