use crate::core::math::{Mat4, Transform};

use super::{SceneError, SceneResult, World};
use crate::scene::components::{
    ActiveSelf, CameraComponent, Hierarchy, LocalTransform, Mobility, RenderLayerMask, SceneNode,
};
use std::any::TypeId;

use crate::scene::EntityId;
use crate::scene::ecs::{
    ArchetypeId, ArchetypeIndexPerformanceStats, ChangeTick, Component, ComponentId,
    ComponentStorageLocation, ComponentTicks, InternalEntity, QueryAccess, StableEntityLocation,
    StorageType,
};

impl World {
    pub(crate) fn archetype_generation(&self) -> u64 {
        self.archetype_index.generation()
    }

    pub(crate) fn matching_query_archetypes(&self, access: &QueryAccess) -> Vec<ArchetypeId> {
        self.archetype_index
            .matching_archetypes(access.with(), access.without())
    }

    pub(crate) fn matching_query_archetypes_from(
        &self,
        access: &QueryAccess,
        first_archetype_index: usize,
    ) -> Vec<ArchetypeId> {
        self.archetype_index.matching_archetypes_from(
            access.with(),
            access.without(),
            first_archetype_index,
        )
    }

    pub(crate) fn query_archetype_index_performance_stats(&self) -> ArchetypeIndexPerformanceStats {
        self.archetype_index.performance_stats()
    }

    pub(crate) fn query_archetype_membership_generation(
        &self,
        archetype: ArchetypeId,
    ) -> Option<u64> {
        self.archetype_index.membership_generation(archetype)
    }

    pub(crate) fn query_component_storage_type(
        &self,
        component_id: ComponentId,
    ) -> Option<StorageType> {
        self.component_registry
            .descriptor(component_id)
            .map(|descriptor| descriptor.storage_type)
    }

    pub(crate) fn query_component_rust_type_id(&self, component_id: ComponentId) -> Option<TypeId> {
        self.component_registry
            .rust_type_for_id(component_id)
            .map(|(rust_type_id, _)| rust_type_id)
    }

    pub(crate) fn query_archetype_contains_component(
        &self,
        archetype: ArchetypeId,
        component_id: ComponentId,
    ) -> bool {
        self.archetype_index
            .signature(archetype)
            .is_some_and(|signature| signature.contains(component_id))
    }

    pub(crate) fn query_archetype_column_slot(
        &self,
        archetype: ArchetypeId,
        component_id: ComponentId,
    ) -> Option<usize> {
        self.archetype_index.column_slot(archetype, component_id)
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

    pub(crate) fn query_archetype_entity_count(&self, archetype: ArchetypeId) -> usize {
        self.archetype_index
            .entities(archetype)
            .map_or(0, <[_]>::len)
    }

    pub(crate) fn stable_query_location_iter(
        &self,
        archetypes: impl IntoIterator<Item = ArchetypeId>,
    ) -> super::StableQueryLocationIter<'_> {
        self.stable_query_order.iter_matching(archetypes)
    }

    pub(crate) fn query_stable_location_at(
        &self,
        archetype: ArchetypeId,
        row: usize,
    ) -> Option<StableEntityLocation> {
        let entity = *self.archetype_index.entities(archetype)?.get(row)?;
        self.internal_entity_location(entity)
    }

    pub(crate) fn query_sparse_component_location(
        &self,
        component_id: ComponentId,
        internal: InternalEntity,
    ) -> Option<ComponentStorageLocation> {
        self.component_storage.location(component_id, internal)
    }

    pub(crate) fn component_ref_with_ticks_at_location<T>(
        &self,
        location: ComponentStorageLocation,
    ) -> Option<(&T, ComponentTicks)>
    where
        T: Component,
    {
        match location.storage_type {
            StorageType::Table => {
                let row = location.table_row?;
                let archetype = location.table_archetype?;
                let column_slot = location.table_column_slot?;
                let value = self
                    .archetype_index
                    .get_by_slot::<T>(archetype, row, column_slot)?;
                let ticks =
                    self.archetype_index
                        .component_ticks_by_slot(archetype, row, column_slot)?;
                Some((value, ticks))
            }
            StorageType::SparseSet => self
                .component_storage
                .get_with_ticks_at_location::<T>(location),
        }
    }

    pub(crate) fn query_component_mut_at_location<T>(
        &mut self,
        entity: EntityId,
        location: ComponentStorageLocation,
    ) -> Option<&mut T>
    where
        T: Component,
    {
        let tick = self.mutation_change_tick();
        self.mark_query_component_mutation::<T>(entity);
        match location.storage_type {
            StorageType::Table => self.archetype_index.get_mut_at_tick_by_slot::<T>(
                location.table_archetype?,
                location.table_row?,
                location.table_column_slot?,
                tick,
            ),
            StorageType::SparseSet => self.component_storage.get_mut_at_tick::<T>(
                location.component_id,
                location.entity,
                tick,
            ),
        }
    }

    pub(crate) fn query_component_mut_with_ticks_at_location<T>(
        &mut self,
        entity: EntityId,
        location: ComponentStorageLocation,
    ) -> Option<(
        &mut T,
        &mut ComponentTicks,
        ChangeTick,
        crate::scene::ecs::ComponentMutationRecorder<'_>,
    )>
    where
        T: Component,
    {
        let tick = self.mutation_change_tick();
        let mutation_recorder = self
            .derived_state_dirty
            .component_mutation_recorder::<T>(entity);
        let (value, ticks) = match location.storage_type {
            StorageType::Table => self.archetype_index.get_mut_with_ticks_by_slot::<T>(
                location.table_archetype?,
                location.table_row?,
                location.table_column_slot?,
            )?,
            StorageType::SparseSet => self
                .component_storage
                .get_mut_with_ticks::<T>(location.component_id, location.entity)?,
        };
        Some((value, ticks, tick, mutation_recorder))
    }

    pub fn contains_entity(&self, entity: EntityId) -> bool {
        self.entity_registry.contains_stable(entity)
    }

    pub fn camera_count(&self) -> usize {
        self.registered_component_id::<CameraComponent>()
            .map(|component_id| self.component_count_for_id(component_id))
            .unwrap_or(0)
    }

    pub(super) fn first_stable_camera_entity(&self) -> Option<EntityId> {
        let camera_component_id = self.registered_component_id::<CameraComponent>()?;
        let archetypes = self
            .archetype_index
            .matching_archetypes(&[camera_component_id], &[]);
        self.stable_query_order
            .iter_matching(archetypes)
            .next()
            .map(|location| location.stable_id)
    }

    pub fn parent_of(&self, entity: EntityId) -> Option<EntityId> {
        let Some(hierarchy) = self.get::<Hierarchy>(entity) else {
            return None;
        };

        hierarchy.parent
    }

    pub fn active_camera(&self) -> EntityId {
        self.active_camera
    }

    pub fn set_active_camera(&mut self, entity: EntityId) {
        if self.contains_component::<CameraComponent>(entity) && self.active_camera != entity {
            self.active_camera = entity;
            self.mark_node_cache_dirty();
        }
    }

    pub fn nodes(&self) -> &[SceneNode] {
        &self.node_cache
    }

    pub fn node_records(&self) -> Vec<SceneNode> {
        let mut nodes = Vec::with_capacity(self.entities.len());
        for entity in self.stable_entity_ids() {
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
        self.get::<LocalTransform>(entity)
            .map(|local| local.transform)
    }

    pub fn world_matrix(&self, entity: EntityId) -> Option<Mat4> {
        self.project_world_matrix_for_read(entity)
    }

    pub fn world_transform(&self, entity: EntityId) -> Option<Transform> {
        self.project_world_transform(entity)
    }

    pub fn active_self(&self, entity: EntityId) -> Option<bool> {
        let Some(active) = self.get::<ActiveSelf>(entity) else {
            return None;
        };

        Some(active.0)
    }

    pub fn set_active_self(&mut self, entity: EntityId, active: bool) -> SceneResult<bool> {
        let Some(current) = self.get::<ActiveSelf>(entity) else {
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
        let Some(mask) = self.get::<RenderLayerMask>(entity) else {
            return None;
        };

        Some(mask.0)
    }

    pub fn set_render_layer_mask(&mut self, entity: EntityId, mask: u32) -> SceneResult<bool> {
        let Some(current) = self.get::<RenderLayerMask>(entity) else {
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
        self.get::<Mobility>(entity).copied()
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
