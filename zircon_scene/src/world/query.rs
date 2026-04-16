use zircon_math::{Mat4, Transform};

use super::World;
use crate::components::{Mobility, SceneNode, Schedule};
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

    pub fn world_matrix(&self, entity: EntityId) -> Option<Mat4> {
        self.world_matrices
            .get(&entity)
            .copied()
            .map(|matrix| matrix.0)
    }

    pub fn world_transform(&self, entity: EntityId) -> Option<Transform> {
        self.project_world_transform(entity)
    }

    pub fn active_self(&self, entity: EntityId) -> Option<bool> {
        self.active_self
            .get(&entity)
            .copied()
            .map(|active| active.0)
    }

    pub fn set_active_self(&mut self, entity: EntityId, active: bool) -> Result<bool, String> {
        let Some(current) = self.active_self.get_mut(&entity) else {
            return Err(format!(
                "cannot update active state for missing node {entity}"
            ));
        };
        if current.0 == active {
            return Ok(false);
        }
        current.0 = active;
        self.rebuild_derived_state();
        Ok(true)
    }

    pub fn active_in_hierarchy(&self, entity: EntityId) -> Option<bool> {
        self.active_in_hierarchy
            .get(&entity)
            .copied()
            .map(|active| active.0)
    }

    pub fn render_layer_mask(&self, entity: EntityId) -> Option<u32> {
        self.render_layer_masks
            .get(&entity)
            .copied()
            .map(|mask| mask.0)
    }

    pub fn set_render_layer_mask(&mut self, entity: EntityId, mask: u32) -> Result<bool, String> {
        let Some(current) = self.render_layer_masks.get_mut(&entity) else {
            return Err(format!(
                "cannot update render layer mask for missing node {entity}"
            ));
        };
        if current.0 == mask {
            return Ok(false);
        }
        current.0 = mask;
        Ok(true)
    }

    pub fn mobility(&self, entity: EntityId) -> Option<Mobility> {
        self.mobility.get(&entity).copied()
    }

    pub fn set_mobility(&mut self, entity: EntityId, mobility: Mobility) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!("cannot update mobility for missing node {entity}"));
        }
        if self.mobility(entity) == Some(mobility) {
            return Ok(false);
        }
        self.validate_mobility_change(entity, mobility)?;
        self.mobility.insert(entity, mobility);
        Ok(true)
    }
}
