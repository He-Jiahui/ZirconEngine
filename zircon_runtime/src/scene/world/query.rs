use crate::core::math::{Mat4, Transform};

use super::{SceneError, SceneResult, World};
use crate::scene::components::{
    ActiveSelf, CameraComponent, Hierarchy, LocalTransform, Mobility, RenderLayerMask, SceneNode,
};
use std::any::TypeId;

use crate::scene::ecs::{
    ArchetypeId, ArchetypeIndexPerformanceStats, ChangeTick, Component, ComponentId,
    ComponentStorageLocation, ComponentTicks, InternalEntity, QueryAccess, StableEntityLocation,
    StorageType,
};
use crate::scene::EntityId;

impl World {
    /// Borrows only the stable-order leaf; ordinary eager metadata is not retained.
    ///
    /// # Safety
    /// The original shared loan or System query grant must keep this World and its
    /// structural order fixed for `'world`. No overlapping structural write is allowed.
    pub(crate) unsafe fn query_entity_ids<'world>(
        world: *const Self,
    ) -> super::StableWorldEntityIter<'world> {
        unsafe { (&*std::ptr::addr_of!((*world).stable_query_order)).entities() }
    }

    /// Projects the existing stable-order merge cursor without borrowing its World parent.
    ///
    /// # Safety
    /// The requirements of `query_entity_ids` apply to the complete cursor lifetime.
    pub(crate) unsafe fn query_stable_location_iter<'world>(
        world: *const Self,
        archetypes: impl IntoIterator<Item = ArchetypeId>,
    ) -> super::StableQueryLocationIter<'world> {
        unsafe { (&*std::ptr::addr_of!((*world).stable_query_order)).iter_matching(archetypes) }
    }

    /// Fetches a shared component through its exact storage field.
    ///
    /// # Safety
    /// `world` must remain valid for `'world`; its structure and storage allocations
    /// must stay fixed. The selected payload must have declared shared access compatible
    /// with every live item/Param. Only contracted, compatible validation may form a
    /// scoped shared World; no World or ordinary metadata parent reference may escape.
    pub(crate) unsafe fn query_component_ref<'world, T>(
        world: *const Self,
        entity: EntityId,
    ) -> Option<&'world T>
    where
        T: Component,
    {
        unsafe {
            let (component_id, internal) = {
                let read = &*world;
                (
                    read.registered_component_id::<T>()?,
                    read.internal_entity(entity)?,
                )
            };
            match T::STORAGE_TYPE {
                StorageType::Table => {
                    let location = (&*std::ptr::addr_of!((*world).entity_registry))
                        .location_for_internal(internal)
                        .ok()?
                        .location;
                    (&*std::ptr::addr_of!((*world).archetype_index)).get::<T>(
                        location.archetype_id,
                        location.table_row,
                        component_id,
                    )
                }
                StorageType::SparseSet => (&*std::ptr::addr_of!((*world).component_storage))
                    .get::<T>(component_id, internal),
            }
        }
    }

    /// Returns a shared row leaf and copied ticks under the same checked type semantics.
    ///
    /// # Safety
    /// `query_component_ref`'s requirements also cover the selected tick read; the
    /// returned component reference must not conflict with any current mutable item.
    pub(crate) unsafe fn query_component_ref_with_ticks<'world, T>(
        world: *const Self,
        entity: EntityId,
    ) -> Option<(&'world T, ComponentTicks)>
    where
        T: Component,
    {
        unsafe {
            let value = Self::query_component_ref::<T>(world, entity)?;
            let (component_id, internal) = {
                let read = &*world;
                (
                    read.registered_component_id::<T>()?,
                    read.internal_entity(entity)?,
                )
            };
            let ticks = match T::STORAGE_TYPE {
                StorageType::Table => {
                    let location = (&*std::ptr::addr_of!((*world).entity_registry))
                        .location_for_internal(internal)
                        .ok()?
                        .location;
                    (&*std::ptr::addr_of!((*world).archetype_index)).component_ticks(
                        location.archetype_id,
                        location.table_row,
                        component_id,
                    )?
                }
                StorageType::SparseSet => (&*std::ptr::addr_of!((*world).component_storage))
                    .ticks(component_id, internal)?,
            };
            Some((value, ticks))
        }
    }

    /// Fetches shared leaves from a coherent compiled location with real row-type checks.
    ///
    /// # Safety
    /// The grant/lifetime/compatible payload and tick requirements of
    /// `query_component_ref_with_ticks` apply. The location must name the current row;
    /// its public type identifier never replaces the lower owner's actual type check.
    pub(crate) unsafe fn query_component_ref_with_ticks_at_location<'world, T>(
        world: *const Self,
        location: ComponentStorageLocation,
    ) -> Option<(&'world T, ComponentTicks)>
    where
        T: Component,
    {
        unsafe {
            match location.storage_type {
                StorageType::Table => {
                    let row = location.table_row?;
                    let archetype = location.table_archetype?;
                    let column_slot = location.table_column_slot?;
                    let index = &*std::ptr::addr_of!((*world).archetype_index);
                    let value = index.get_by_slot::<T>(archetype, row, column_slot)?;
                    let ticks = index.component_ticks_by_slot(archetype, row, column_slot)?;
                    Some((value, ticks))
                }
                StorageType::SparseSet => (&*std::ptr::addr_of!((*world).component_storage))
                    .get_with_ticks_at_location::<T>(location),
            }
        }
    }

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

    /// Copies ticks after checking the actual stored type, without borrowing a value as `&T`.
    pub(crate) fn component_ticks_at_location<T>(
        &self,
        location: ComponentStorageLocation,
    ) -> Option<ComponentTicks>
    where
        T: Component,
    {
        match location.storage_type {
            StorageType::Table => self.archetype_index.component_ticks_by_slot_for_type::<T>(
                location.table_archetype?,
                location.table_row?,
                location.table_column_slot?,
            ),
            StorageType::SparseSet => self
                .component_storage
                .ticks_at_location_for_type::<T>(location),
        }
    }

    /// Fetches a plain mutable query row without borrowing its World or storage parents mutably.
    ///
    /// # Safety
    /// The caller must own the exclusive World query loan for `'world`, keep entity and
    /// storage allocations fixed, and grant unique value/tick access to this row. Every live
    /// item must be disjoint from this row and from the ordinary metadata fields updated by
    /// this fetch; an item may retain only its row leaves and the shared mutation-sink leaf.
    /// Scoped validation and eager Name/ActiveSelf effects also require compatible Hierarchy
    /// reads, including ancestors/subtrees outside the candidate row.
    pub(crate) unsafe fn query_component_mut<'world, T>(
        world: *mut Self,
        entity: EntityId,
    ) -> Option<&'world mut T>
    where
        T: Component,
    {
        if Self::protected_derived_component_name::<T>().is_some()
            || Self::protected_authored_component_name::<T>().is_some()
        {
            return None;
        }
        unsafe {
            // Preserve get_mut's clock-before-registration/missing-component order.
            let tick = Self::query_mutation_change_tick(world);
            let (component_id, internal) = {
                let read = &*world;
                let component_id = read.registered_component_id::<T>()?;
                let internal = read.internal_entity(entity)?;
                if !read.contains_component_id(entity, component_id) {
                    return None;
                }
                (component_id, internal)
            };
            Self::mark_query_component_mutation::<T>(world, entity);
            match T::STORAGE_TYPE {
                StorageType::Table => {
                    let location = (&*std::ptr::addr_of!((*world).entity_registry))
                        .location_for_internal(internal)
                        .ok()?
                        .location;
                    let index = &*std::ptr::addr_of!((*world).archetype_index);
                    let slot = index.column_slot(location.archetype_id, component_id)?;
                    index.get_mut_at_tick_by_slot_unchecked::<T>(
                        location.archetype_id,
                        location.table_row,
                        slot,
                        tick,
                    )
                }
                StorageType::SparseSet => (&*std::ptr::addr_of!((*world).component_storage))
                    .get_mut_at_tick_unchecked::<T>(component_id, internal, tick),
            }
        }
    }

    /// Fetches a tracked row and its mutation-sink leaf with the ordinary Mut semantics.
    ///
    /// # Safety
    /// The raw World grant, row-disjointness, structural stability, metadata exclusivity and
    /// compatible readonly accesses required by `query_component_mut` apply here too.
    /// The sink and selected row must remain valid for `'world`; no live item may retain a
    /// World, storage container, DerivedStateDirty or RenderDirtyJournalState parent reference.
    pub(crate) unsafe fn query_component_mut_with_ticks<'world, T>(
        world: *mut Self,
        entity: EntityId,
    ) -> Option<(
        &'world mut T,
        &'world mut ComponentTicks,
        ChangeTick,
        crate::scene::ecs::ComponentMutationRecorder<'world>,
    )>
    where
        T: Component,
    {
        if Self::protected_derived_component_name::<T>().is_some()
            || Self::protected_authored_component_name::<T>().is_some()
        {
            return None;
        }
        unsafe {
            // Tracked fetch checks registration/entity before advancing the clock.
            let (component_id, internal) = {
                let read = &*world;
                (
                    read.registered_component_id::<T>()?,
                    read.internal_entity(entity)?,
                )
            };
            let tick = Self::query_mutation_change_tick(world);
            let mutation_recorder =
                super::dirty_state::DerivedStateDirty::component_mutation_recorder_unchecked::<T>(
                    std::ptr::addr_of!((*world).derived_state_dirty),
                    entity,
                );
            let (value, ticks) = match T::STORAGE_TYPE {
                StorageType::Table => {
                    let location = (&*std::ptr::addr_of!((*world).entity_registry))
                        .location_for_internal(internal)
                        .ok()?
                        .location;
                    let index = &*std::ptr::addr_of!((*world).archetype_index);
                    let slot = index.column_slot(location.archetype_id, component_id)?;
                    index.get_mut_with_ticks_by_slot_unchecked::<T>(
                        location.archetype_id,
                        location.table_row,
                        slot,
                    )?
                }
                StorageType::SparseSet => (&*std::ptr::addr_of!((*world).component_storage))
                    .get_mut_with_ticks_unchecked::<T>(component_id, internal)?,
            };
            Some((value, ticks, tick, mutation_recorder))
        }
    }

    /// Fetches a compiled plain row under the same grant as `query_component_mut`.
    ///
    /// # Safety
    /// The caller must satisfy `query_component_mut`'s requirements and keep `location`
    /// tied to the current entity/storage generation. The lower owner checks the actual
    /// value type; the location's user-visible Rust type identifier is not an authority.
    pub(crate) unsafe fn query_component_mut_at_location<'world, T>(
        world: *mut Self,
        entity: EntityId,
        location: ComponentStorageLocation,
    ) -> Option<&'world mut T>
    where
        T: Component,
    {
        if Self::protected_derived_component_name::<T>().is_some()
            || Self::protected_authored_component_name::<T>().is_some()
        {
            return None;
        }
        unsafe {
            let tick = Self::query_mutation_change_tick(world);
            Self::mark_query_component_mutation::<T>(world, entity);
            match location.storage_type {
                StorageType::Table => (&*std::ptr::addr_of!((*world).archetype_index))
                    .get_mut_at_tick_by_slot_unchecked::<T>(
                        location.table_archetype?,
                        location.table_row?,
                        location.table_column_slot?,
                        tick,
                    ),
                StorageType::SparseSet => (&*std::ptr::addr_of!((*world).component_storage))
                    .get_mut_at_tick_unchecked::<T>(location.component_id, location.entity, tick),
            }
        }
    }

    /// Fetches a compiled tracked row under the same grant as `query_component_mut_with_ticks`.
    ///
    /// # Safety
    /// The tracked raw fetch requirements apply, and `location` must remain tied to the
    /// current entity/storage generation. Only row leaves and the sink leaf may escape.
    pub(crate) unsafe fn query_component_mut_with_ticks_at_location<'world, T>(
        world: *mut Self,
        entity: EntityId,
        location: ComponentStorageLocation,
    ) -> Option<(
        &'world mut T,
        &'world mut ComponentTicks,
        ChangeTick,
        crate::scene::ecs::ComponentMutationRecorder<'world>,
    )>
    where
        T: Component,
    {
        if Self::protected_derived_component_name::<T>().is_some()
            || Self::protected_authored_component_name::<T>().is_some()
        {
            return None;
        }
        unsafe {
            let tick = Self::query_mutation_change_tick(world);
            let mutation_recorder =
                super::dirty_state::DerivedStateDirty::component_mutation_recorder_unchecked::<T>(
                    std::ptr::addr_of!((*world).derived_state_dirty),
                    entity,
                );
            let (value, ticks) = match location.storage_type {
                StorageType::Table => (&*std::ptr::addr_of!((*world).archetype_index))
                    .get_mut_with_ticks_by_slot_unchecked::<T>(
                        location.table_archetype?,
                        location.table_row?,
                        location.table_column_slot?,
                    )?,
                StorageType::SparseSet => (&*std::ptr::addr_of!((*world).component_storage))
                    .get_mut_with_ticks_unchecked::<T>(location.component_id, location.entity)?,
            };
            Some((value, ticks, tick, mutation_recorder))
        }
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
        self.insert_prevalidated_authored_component(entity, mobility)?;
        Ok(true)
    }
}
