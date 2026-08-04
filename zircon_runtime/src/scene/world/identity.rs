use crate::scene::ecs::{
    ArchetypeId, ArchetypeSignature, ComponentId, EntityLocation, InternalEntity,
    StableEntityLocation, StorageType,
};
use crate::scene::EntityId;

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
        // Callers append the entity immediately after registration, so the current list length is its row.
        let row = self.entities.len();
        let internal = self
            .entity_registry
            .spawn(entity, EntityLocation::new(ArchetypeId::EMPTY, row))?;

        Ok(internal)
    }

    pub(super) fn register_prevalidated_stable_entity(
        &mut self,
        entity: EntityId,
    ) -> InternalEntity {
        let row = self.entities.len();
        self.entity_registry
            .spawn_prevalidated(entity, EntityLocation::new(ArchetypeId::EMPTY, row))
    }

    pub(super) fn unregister_stable_entity(&mut self, entity: EntityId) {
        let _ = self.entity_registry.despawn(entity);
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

    pub(super) fn refresh_stable_entity_locations(&mut self) {
        self.rebuild_archetype_index();
    }

    pub(super) fn rebuild_entity_registry(&mut self) {
        self.entity_registry
            .rebuild_from_stable_ids(self.entities.iter().copied())
            .expect("world entity list must not contain duplicate stable ids");
        self.rebuild_archetype_index();
    }

    pub(super) fn refresh_entity_archetype(&mut self, entity: EntityId) {
        let previous = self.archetype_location_for_entity(entity);
        self.assign_entity_archetype_from_component_storage(entity, previous);
    }

    pub(super) fn place_empty_entity_in_archetype(&mut self, entity: EntityId) {
        let previous = self.archetype_location_for_entity(entity);
        self.assign_entity_archetype_with_signature(entity, previous, ArchetypeSignature::empty());
    }

    pub(super) fn add_component_to_entity_archetype(
        &mut self,
        entity: EntityId,
        component_id: ComponentId,
        storage_type: StorageType,
    ) {
        self.update_entity_archetype_component_membership(entity, |signature| {
            signature.with_component_added(component_id, storage_type)
        });
    }

    pub(super) fn remove_component_from_entity_archetype(
        &mut self,
        entity: EntityId,
        component_id: ComponentId,
        storage_type: StorageType,
    ) {
        self.update_entity_archetype_component_membership(entity, |signature| {
            signature.with_component_removed(component_id, storage_type)
        });
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

    pub(super) fn rebuild_archetype_index(&mut self) {
        self.archetype_index = Default::default();
        for entity_index in 0..self.entities.len() {
            let entity = self.entities[entity_index];
            self.assign_entity_archetype_from_component_storage(entity, None);
        }
    }

    fn update_entity_archetype_component_membership(
        &mut self,
        entity: EntityId,
        update: impl FnOnce(&ArchetypeSignature) -> ArchetypeSignature,
    ) {
        let Some(previous) = self.archetype_location_for_entity(entity) else {
            return;
        };
        let Some(current_signature) = self.archetype_index.signature(previous.0).cloned() else {
            return;
        };
        self.assign_entity_archetype_with_signature(
            entity,
            Some(previous),
            update(&current_signature),
        );
    }

    fn assign_entity_archetype_from_component_storage(
        &mut self,
        entity: EntityId,
        previous: Option<(ArchetypeId, usize)>,
    ) {
        let Some(internal) = self.internal_entity(entity) else {
            return;
        };
        let signature = self.archetype_signature_for_internal(internal);
        self.assign_entity_archetype_with_signature(entity, previous, signature);
    }

    fn assign_entity_archetype_with_signature(
        &mut self,
        entity: EntityId,
        previous: Option<(ArchetypeId, usize)>,
        signature: ArchetypeSignature,
    ) {
        let archetype_id = self.archetype_index.id_or_insert(signature);
        let moved = self
            .archetype_index
            .move_entity(entity, previous, archetype_id);
        if let Some((swapped_entity, row)) = moved.swapped_entity {
            self.update_entity_archetype_row(swapped_entity, row);
        }
        let _ = self
            .entity_registry
            .set_location(entity, EntityLocation::new(archetype_id, moved.entity_row));
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
            let _ = self.entity_registry.set_location(entity, location);
        }
    }

    fn archetype_signature_for_internal(&self, internal: InternalEntity) -> ArchetypeSignature {
        let mut table_components = Vec::new();
        let mut sparse_set_components = Vec::new();
        self.component_storage.component_ids_for_entity_by_storage(
            internal,
            &mut table_components,
            &mut sparse_set_components,
        );
        ArchetypeSignature::new(table_components, sparse_set_components)
    }
}
