use crate::scene::ecs::{
    ArchetypeId, ArchetypeIndexPerformanceStats, ArchetypeSignature, ComponentId, ComponentTicks,
    EntityLocation, InternalEntity, StableEntityLocation,
};
use crate::scene::EntityId;
use std::collections::BTreeMap;

use super::{SceneResult, World};

impl World {
    pub fn internal_entity(&self, entity: EntityId) -> Option<InternalEntity> {
        self.entity_registry.internal_for_stable(entity)
    }

    pub fn internal_entity_location(&self, entity: EntityId) -> Option<StableEntityLocation> {
        self.entity_registry.location_for_stable(entity)
    }

    pub fn contains_internal_entity(&self, entity: InternalEntity) -> bool {
        self.entity_registry.contains_internal(entity)
    }

    pub(super) fn register_stable_entity(
        &mut self,
        entity: EntityId,
    ) -> SceneResult<InternalEntity> {
        let internal = self
            .entity_registry
            .spawn(entity, EntityLocation::new(ArchetypeId::EMPTY, usize::MAX))?;
        let row = self.append_empty_archetype_row(entity);
        self.entity_registry
            .set_location(entity, EntityLocation::new(ArchetypeId::EMPTY, row))
            .expect("newly registered entity must accept its empty archetype row");
        self.stable_query_order.register(entity, internal);
        self.stable_query_order
            .move_to(entity, EntityLocation::new(ArchetypeId::EMPTY, row));

        Ok(internal)
    }

    pub(super) fn register_prevalidated_stable_entity(
        &mut self,
        entity: EntityId,
    ) -> InternalEntity {
        let internal = self
            .entity_registry
            .spawn_prevalidated(entity, EntityLocation::new(ArchetypeId::EMPTY, usize::MAX));
        let row = self.append_empty_archetype_row(entity);
        self.entity_registry
            .set_location(entity, EntityLocation::new(ArchetypeId::EMPTY, row))
            .expect("prevalidated entity must accept its empty archetype row");
        self.stable_query_order.register(entity, internal);
        self.stable_query_order
            .move_to(entity, EntityLocation::new(ArchetypeId::EMPTY, row));
        internal
    }

    pub(super) fn register_prevalidated_stable_entity_without_row(
        &mut self,
        entity: EntityId,
        stable_order: usize,
    ) -> InternalEntity {
        let internal = self
            .entity_registry
            .spawn_prevalidated(entity, EntityLocation::new(ArchetypeId::EMPTY, usize::MAX));
        self.stable_query_order
            .register_at_order(entity, internal, stable_order);
        internal
    }

    pub(super) fn unregister_stable_entity(&mut self, entity: EntityId) {
        self.stable_query_order.remove(entity);
        let _ = self.entity_registry.despawn(entity);
    }

    pub(super) fn append_entity_to_dense_storage(&mut self, entity: EntityId) {
        let row = self.entities.len();
        let replaced = self.entity_dense_rows.insert(entity, row);
        debug_assert!(replaced.is_none());
        self.entities.push(entity);
    }

    pub(super) fn remove_entity_from_dense_storage(&mut self, entity: EntityId) -> bool {
        let Some(row) = self.entity_dense_rows.remove(&entity) else {
            return false;
        };
        let removed = self.entities.swap_remove(row);
        debug_assert_eq!(removed, entity);
        if let Some(&swapped_entity) = self.entities.get(row) {
            let replaced = self.entity_dense_rows.insert(swapped_entity, row);
            debug_assert_eq!(replaced, Some(self.entities.len()));
        }
        true
    }

    pub(super) fn stable_entity_ids(&self) -> super::StableWorldEntityIter<'_> {
        self.stable_query_order.entities()
    }

    pub(super) fn stable_entity_order(&self, entity: EntityId) -> Option<usize> {
        self.stable_query_order.order_of(entity)
    }

    pub(super) fn stable_entity_order_is_occupied(&self, order: usize) -> bool {
        self.stable_query_order.contains_order(order)
    }

    pub(super) fn remove_entity_from_archetype(&mut self, entity: EntityId) {
        let Some(stable_location) = self.entity_registry.location_for_stable(entity) else {
            return;
        };
        let location = stable_location.location;
        let swapped_entity = self.archetype_index.remove_entity_at(
            location.archetype_id,
            location.table_row,
            entity,
        );
        if let Some((swapped_entity, row)) = swapped_entity {
            self.update_entity_archetype_row(swapped_entity, row);
        }
    }

    pub(super) fn rebuild_entity_registry(&mut self) {
        let stable_entities = self.entities.clone();
        self.rebuild_entity_registry_with_stable_order(stable_entities);
    }

    pub(super) fn rebuild_entity_registry_with_stable_order(
        &mut self,
        stable_entities: Vec<EntityId>,
    ) {
        self.entity_dense_rows.clear();
        for (row, entity) in self.entities.iter().copied().enumerate() {
            let replaced = self.entity_dense_rows.insert(entity, row);
            assert!(
                replaced.is_none(),
                "world entity list must not contain duplicate stable ids"
            );
        }
        self.entity_registry
            .rebuild_from_stable_ids(stable_entities.iter().copied())
            .expect("world entity list must not contain duplicate stable ids");
        let stable_entities = stable_entities
            .into_iter()
            .map(|entity| {
                let internal = self
                    .entity_registry
                    .internal_for_stable(entity)
                    .expect("rebuilt entity registry must contain every stable world entity");
                (entity, internal)
            })
            .collect::<Vec<_>>();
        self.stable_query_order.rebuild(stable_entities);
        self.reset_archetype_index_for_projection();
    }

    pub(super) fn entity_archetype_signature(
        &self,
        entity: EntityId,
    ) -> Option<ArchetypeSignature> {
        let (archetype_id, _) = self.archetype_location_for_entity(entity)?;
        self.archetype_index.signature(archetype_id).cloned()
    }

    pub(super) fn archetype_assignment_count(&self) -> u64 {
        self.archetype_assignment_counter.get()
    }

    pub(super) fn entity_archetype_component_ids(&self, entity: EntityId) -> Vec<ComponentId> {
        let Some((archetype_id, _)) = self.archetype_location_for_entity(entity) else {
            return Vec::new();
        };
        let Some(signature) = self.archetype_index.signature(archetype_id) else {
            return Vec::new();
        };
        signature.ordered_component_ids()
    }

    pub(super) fn reset_archetype_index_for_projection(&mut self) {
        self.archetype_index = Default::default();
        self.stable_query_order.clear_archetypes();
    }

    pub(super) fn ensure_archetype(&mut self, signature: ArchetypeSignature) -> ArchetypeId {
        let table_columns = self
            .component_registry
            .table_column_layouts_for_ids(signature.table_components())
            .expect("every table signature component must own a registered column layout");
        self.archetype_index.id_or_insert(signature, table_columns)
    }

    pub(super) fn transition_entity_archetype_row(
        &mut self,
        entity: EntityId,
        target_signature: ArchetypeSignature,
        updates: BTreeMap<
            ComponentId,
            Option<(Box<dyn std::any::Any + Send + Sync>, ComponentTicks)>,
        >,
    ) -> Option<BTreeMap<ComponentId, (Box<dyn std::any::Any + Send + Sync>, ComponentTicks)>> {
        let stable_location = self.entity_registry.location_for_stable(entity)?;
        let target_archetype = self.ensure_archetype(target_signature);
        if stable_location.location.archetype_id == target_archetype {
            return None;
        }
        let source_table_components = self
            .archetype_index
            .signature(stable_location.location.archetype_id)
            .expect("entity location must identify a registered source archetype")
            .table_components()
            .to_vec();
        self.archetype_index
            .validate_transition(
                target_archetype,
                source_table_components.into_iter(),
                &updates,
            )
            .expect("prepared structural row delta must match the target archetype schema");

        self.archetype_assignment_counter.record_assignment();
        let mut components = self.take_archetype_row_components(entity, stable_location.location);
        let mut previous = BTreeMap::new();
        for (component_id, update) in updates {
            match update {
                Some(component) => {
                    if let Some(replaced) = components.insert(component_id, component) {
                        previous.insert(component_id, replaced);
                    }
                }
                None => {
                    if let Some(removed) = components.remove(&component_id) {
                        previous.insert(component_id, removed);
                    }
                }
            }
        }
        self.append_entity_archetype_row(entity, target_archetype, components);
        Some(previous)
    }

    fn append_empty_archetype_row(&mut self, entity: EntityId) -> usize {
        let row = self
            .archetype_index
            .preflight_row(
                ArchetypeId::EMPTY,
                std::iter::empty::<(
                    ComponentId,
                    Box<dyn std::any::Any + Send + Sync>,
                    ComponentTicks,
                )>(),
            )
            .expect("the empty archetype must accept an empty row");
        let table_row =
            self.archetype_index
                .append_preflighted_row(ArchetypeId::EMPTY, entity, row);
        self.record_ecs_archetype_index_stats(ArchetypeIndexPerformanceStats {
            row_appends: 1,
            ..Default::default()
        });
        table_row
    }

    pub(super) fn take_archetype_row_components(
        &mut self,
        entity: EntityId,
        location: EntityLocation,
    ) -> BTreeMap<ComponentId, (Box<dyn std::any::Any + Send + Sync>, ComponentTicks)> {
        let taken = self
            .archetype_index
            .take_entity_row(location.archetype_id, location.table_row, entity)
            .expect("entity location must identify its complete archetype row");
        debug_assert_eq!(taken.entity(), entity);
        if let Some(swapped_entity) = taken.swapped_entity() {
            self.update_entity_archetype_row(swapped_entity, location.table_row);
        }
        taken.into_components()
    }

    pub(super) fn append_entity_archetype_row(
        &mut self,
        entity: EntityId,
        archetype_id: ArchetypeId,
        components: BTreeMap<ComponentId, (Box<dyn std::any::Any + Send + Sync>, ComponentTicks)>,
    ) {
        let row = self
            .archetype_index
            .bind_prevalidated_row(archetype_id, components);
        let table_row = self
            .archetype_index
            .append_preflighted_row(archetype_id, entity, row);
        self.record_ecs_archetype_index_stats(ArchetypeIndexPerformanceStats {
            row_appends: 1,
            ..Default::default()
        });
        let location = EntityLocation::new(archetype_id, table_row);
        self.entity_registry
            .set_location(entity, location)
            .expect("archetype-assigned entity must remain registered");
        self.stable_query_order.move_to(entity, location);
    }

    fn archetype_location_for_entity(&self, entity: EntityId) -> Option<(ArchetypeId, usize)> {
        let location = self.entity_registry.location_for_stable(entity)?.location;
        let located_entity = self
            .archetype_index
            .entities(location.archetype_id)
            .and_then(|entities| entities.get(location.table_row))
            .copied();
        (located_entity == Some(entity)).then_some((location.archetype_id, location.table_row))
    }

    fn update_entity_archetype_row(&mut self, entity: EntityId, row: usize) {
        if let Some(stable_location) = self.entity_registry.location_for_stable(entity) {
            let mut location = stable_location.location;
            location.table_row = row;
            self.entity_registry
                .set_location(entity, location)
                .expect("swapped archetype entity must remain registered");
            self.stable_query_order.update_row(entity, row);
        }
    }
}
