use crate::core::math::{Mat4, Transform};

use super::{SceneError, SceneResult, World};
use crate::scene::EntityId;
use crate::scene::components::{ActiveSelf, Mobility, RenderLayerMask, SceneNode};
use crate::scene::ecs::{
    ArchetypeId, Component, ComponentId, ComponentStorageLocation, ComponentTicks, InternalEntity,
    QueryAccess, StableEntityLocation,
};

impl World {
    pub(crate) fn query_cache_revision(&self) -> u64 {
        self.query_cache_revision.get()
    }

    pub(super) fn bump_query_cache_revision(&mut self) {
        self.query_cache_revision.advance();
    }

    pub(crate) fn archetype_generation(&self) -> u64 {
        self.archetype_index.generation()
    }

    pub(crate) fn matching_query_archetypes(&self, access: &QueryAccess) -> Vec<ArchetypeId> {
        self.archetype_index
            .matching_archetypes(access.with(), access.without())
    }

    pub(crate) fn matching_query_archetype_entity_count(
        &self,
        archetypes: &[ArchetypeId],
    ) -> usize {
        let mut count = 0;
        for archetype in archetypes {
            if let Some(entities) = self.archetype_index.entities(*archetype) {
                count += entities.len();
            }
        }
        count
    }

    pub(crate) fn visit_entity_locations_matching_archetypes(
        &self,
        archetypes: &[ArchetypeId],
        mut visitor: impl FnMut(StableEntityLocation),
    ) {
        if archetypes.is_empty() {
            return;
        }
        for entity in self.entities.iter().copied() {
            let Some(location) = self.internal_entity_location(entity) else {
                continue;
            };
            if archetypes
                .binary_search(&location.location.archetype_id)
                .is_ok()
            {
                visitor(location);
            }
        }
    }

    pub(crate) fn component_storage_locations_for_internal(
        &self,
        internal: InternalEntity,
        component_ids: &[ComponentId],
        output: &mut Vec<ComponentStorageLocation>,
    ) {
        output.clear();
        let component_count = component_ids.len();
        if component_count == 0 {
            return;
        }
        output.reserve(component_count);
        for component_id in component_ids {
            if let Some(location) = self.component_storage.location(*component_id, internal) {
                output.push(location);
            }
        }
    }

    pub(crate) fn component_ref_with_ticks_at_location<T>(
        &self,
        location: ComponentStorageLocation,
    ) -> Option<(&T, ComponentTicks)>
    where
        T: Component,
    {
        self.component_storage
            .get_with_ticks_at_location::<T>(location)
    }

    pub fn contains_entity(&self, entity: EntityId) -> bool {
        self.entity_registry.contains_stable(entity)
    }

    pub fn camera_count(&self) -> usize {
        self.cameras.len()
    }

    pub fn parent_of(&self, entity: EntityId) -> Option<EntityId> {
        let Some(hierarchy) = self.hierarchy.get(&entity) else {
            return None;
        };

        hierarchy.parent
    }

    pub fn active_camera(&self) -> EntityId {
        self.active_camera
    }

    pub fn set_active_camera(&mut self, entity: EntityId) {
        if self.cameras.contains_key(&entity) && self.active_camera != entity {
            self.active_camera = entity;
            self.mark_node_cache_dirty();
        }
    }

    pub fn nodes(&self) -> &[SceneNode] {
        &self.node_cache
    }

    pub fn node_records(&self) -> Vec<SceneNode> {
        let mut nodes = Vec::with_capacity(self.entities.len());
        for entity in self.entities.iter().copied() {
            let Some(node) = self.project_node_for_read(entity) else {
                continue;
            };
            nodes.push(node);
        }
        nodes.sort_by_key(|node| node.id);
        nodes
    }

    pub fn find_node(&self, entity: EntityId) -> Option<SceneNode> {
        self.project_node_for_read(entity)
    }

    /// Reads only the entity's local transform without projecting an owned scene node.
    pub fn local_transform(&self, entity: EntityId) -> Option<Transform> {
        self.local_transforms
            .get(&entity)
            .map(|local| local.transform)
    }

    pub fn world_matrix(&self, entity: EntityId) -> Option<Mat4> {
        self.project_world_matrix_for_read(entity)
    }

    pub fn world_transform(&self, entity: EntityId) -> Option<Transform> {
        self.project_world_transform(entity)
    }

    pub fn active_self(&self, entity: EntityId) -> Option<bool> {
        let Some(active) = self.active_self.get(&entity) else {
            return None;
        };

        Some(active.0)
    }

    pub fn set_active_self(&mut self, entity: EntityId, active: bool) -> SceneResult<bool> {
        let Some(current) = self.active_self.get(&entity) else {
            if !self.contains_entity(entity) {
                return Err(SceneError::missing_entity(
                    "update active state for",
                    entity,
                ));
            }
            return Err(SceneError::MissingRequiredComponent {
                operation: "update active state",
                entity,
                component: "ActiveSelf",
            });
        };
        if current.0 == active {
            return Ok(false);
        }
        self.insert(entity, ActiveSelf(active))?;
        Ok(true)
    }

    pub fn active_in_hierarchy(&self, entity: EntityId) -> Option<bool> {
        self.project_active_in_hierarchy_for_read(entity)
    }

    pub fn render_layer_mask(&self, entity: EntityId) -> Option<u32> {
        let Some(mask) = self.render_layer_masks.get(&entity) else {
            return None;
        };

        Some(mask.0)
    }

    pub fn set_render_layer_mask(&mut self, entity: EntityId, mask: u32) -> SceneResult<bool> {
        let Some(current) = self.render_layer_masks.get(&entity) else {
            if !self.contains_entity(entity) {
                return Err(SceneError::missing_entity(
                    "update render layer mask for",
                    entity,
                ));
            }
            return Err(SceneError::MissingRequiredComponent {
                operation: "update render layer mask",
                entity,
                component: "RenderLayerMask",
            });
        };
        if current.0 == mask {
            return Ok(false);
        }
        self.insert(entity, RenderLayerMask(mask))?;
        Ok(true)
    }

    pub fn mobility(&self, entity: EntityId) -> Option<Mobility> {
        self.mobility.get(&entity).copied()
    }

    pub fn set_mobility(&mut self, entity: EntityId, mobility: Mobility) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("update mobility for", entity));
        }
        if self.mobility(entity) == Some(mobility) {
            return Ok(false);
        }
        self.validate_mobility_change(entity, mobility)?;
        self.insert(entity, mobility)?;
        Ok(true)
    }
}
