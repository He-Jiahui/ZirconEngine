use crate::scene::ecs::{
    ArchetypeId, EntityLocation, EntityRegistryError, InternalEntity, StableEntityLocation,
};
use crate::scene::EntityId;

use super::World;

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
    ) -> Result<InternalEntity, String> {
        let row = self
            .entities
            .iter()
            .position(|candidate| *candidate == entity)
            .unwrap_or(self.entities.len());
        self.entity_registry
            .spawn(entity, EntityLocation::new(ArchetypeId::EMPTY, row))
            .map_err(entity_registry_error_to_string)
    }

    pub(super) fn unregister_stable_entity(&mut self, entity: EntityId) {
        let _ = self.entity_registry.despawn(entity);
    }

    pub(super) fn refresh_stable_entity_locations(&mut self) {
        for (row, entity) in self.entities.iter().copied().enumerate() {
            let _ = self
                .entity_registry
                .set_location(entity, EntityLocation::new(ArchetypeId::EMPTY, row));
        }
    }

    pub(super) fn rebuild_entity_registry(&mut self) {
        self.entity_registry
            .rebuild_from_stable_ids(self.entities.iter().copied())
            .expect("world entity list must not contain duplicate stable ids");
    }
}

fn entity_registry_error_to_string(error: EntityRegistryError) -> String {
    error.to_string()
}
