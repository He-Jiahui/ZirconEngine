use crate::scene::ecs::{
    ArchetypeId, ArchetypeSignature, EntityLocation, InternalEntity, StableEntityLocation,
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

    pub(super) fn unregister_stable_entity(&mut self, entity: EntityId) {
        let _ = self.entity_registry.despawn(entity);
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
        let previous = match self.entity_registry.location_for_stable(entity) {
            Some(location) => Some(location.location.archetype_id),
            None => None,
        };
        self.assign_entity_archetype(entity, previous);
    }

    pub(super) fn rebuild_archetype_index(&mut self) {
        self.archetype_index = Default::default();
        for entity_index in 0..self.entities.len() {
            let entity = self.entities[entity_index];
            self.assign_entity_archetype(entity, None);
        }
    }

    fn assign_entity_archetype(&mut self, entity: EntityId, previous: Option<ArchetypeId>) {
        let Some(internal) = self.internal_entity(entity) else {
            return;
        };
        let signature = self.archetype_signature_for_internal(internal);
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
