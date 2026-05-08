use std::collections::HashMap;

use crate::scene::ecs::{
    ArchetypeId, DespawnedEntity, EntityLocation, EntityRegistryError, InternalEntity,
    StableEntityLocation,
};
use crate::scene::EntityId;

const FIRST_GENERATION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityRegistry {
    slots: Vec<EntitySlot>,
    free_slots: Vec<u32>,
    stable_to_internal: HashMap<EntityId, InternalEntity>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EntitySlot {
    generation: u32,
    stable_id: Option<EntityId>,
    location: Option<EntityLocation>,
}

impl EntityRegistry {
    pub fn spawn(
        &mut self,
        stable_id: EntityId,
        location: EntityLocation,
    ) -> Result<InternalEntity, EntityRegistryError> {
        if self.stable_to_internal.contains_key(&stable_id) {
            return Err(EntityRegistryError::DuplicateStableId(stable_id));
        }

        let slot_index = self.allocate_slot();
        let slot = &mut self.slots[slot_index as usize];
        let internal = InternalEntity::new(slot_index, slot.generation);
        slot.stable_id = Some(stable_id);
        slot.location = Some(location);
        self.stable_to_internal.insert(stable_id, internal);
        Ok(internal)
    }

    pub fn despawn(&mut self, stable_id: EntityId) -> Result<DespawnedEntity, EntityRegistryError> {
        let internal = self
            .stable_to_internal
            .remove(&stable_id)
            .ok_or(EntityRegistryError::MissingStableId(stable_id))?;
        let slot = self
            .slots
            .get_mut(internal.index() as usize)
            .ok_or(EntityRegistryError::InvalidInternalEntity(internal))?;
        if slot.generation != internal.generation() || slot.stable_id != Some(stable_id) {
            return Err(EntityRegistryError::InvalidInternalEntity(internal));
        }

        let location = slot.location.take().unwrap_or_default();
        slot.stable_id = None;
        slot.generation = next_generation(slot.generation);
        self.free_slots.push(internal.index());
        Ok(DespawnedEntity {
            stable_id,
            internal,
            location,
        })
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.free_slots.clear();
        self.stable_to_internal.clear();
    }

    pub fn rebuild_from_stable_ids<I>(&mut self, stable_ids: I) -> Result<(), EntityRegistryError>
    where
        I: IntoIterator<Item = EntityId>,
    {
        self.clear();
        for (row, stable_id) in stable_ids.into_iter().enumerate() {
            self.spawn(stable_id, EntityLocation::new(ArchetypeId::EMPTY, row))?;
        }
        Ok(())
    }

    pub fn set_location(
        &mut self,
        stable_id: EntityId,
        location: EntityLocation,
    ) -> Result<(), EntityRegistryError> {
        let internal = self
            .internal_for_stable(stable_id)
            .ok_or(EntityRegistryError::MissingStableId(stable_id))?;
        let slot = self
            .slots
            .get_mut(internal.index() as usize)
            .ok_or(EntityRegistryError::InvalidInternalEntity(internal))?;
        if slot.generation != internal.generation() || slot.stable_id != Some(stable_id) {
            return Err(EntityRegistryError::InvalidInternalEntity(internal));
        }
        slot.location = Some(location);
        Ok(())
    }

    pub fn contains_internal(&self, internal: InternalEntity) -> bool {
        self.location_for_internal(internal).is_ok()
    }

    pub fn contains_stable(&self, stable_id: EntityId) -> bool {
        self.stable_to_internal.contains_key(&stable_id)
    }

    pub fn internal_for_stable(&self, stable_id: EntityId) -> Option<InternalEntity> {
        self.stable_to_internal.get(&stable_id).copied()
    }

    pub fn location_for_stable(&self, stable_id: EntityId) -> Option<StableEntityLocation> {
        self.internal_for_stable(stable_id)
            .and_then(|internal| self.location_for_internal(internal).ok())
    }

    pub fn location_for_internal(
        &self,
        internal: InternalEntity,
    ) -> Result<StableEntityLocation, EntityRegistryError> {
        let slot = self
            .slots
            .get(internal.index() as usize)
            .ok_or(EntityRegistryError::InvalidInternalEntity(internal))?;
        if slot.generation != internal.generation() {
            return Err(EntityRegistryError::InvalidInternalEntity(internal));
        }
        let stable_id = slot
            .stable_id
            .ok_or(EntityRegistryError::InvalidInternalEntity(internal))?;
        let location = slot
            .location
            .ok_or(EntityRegistryError::InvalidInternalEntity(internal))?;
        Ok(StableEntityLocation {
            stable_id,
            internal,
            location,
        })
    }

    pub fn len(&self) -> usize {
        self.stable_to_internal.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stable_to_internal.is_empty()
    }

    fn allocate_slot(&mut self) -> u32 {
        if let Some(slot_index) = self.free_slots.pop() {
            return slot_index;
        }
        let slot_index = self.slots.len() as u32;
        self.slots.push(EntitySlot::default());
        slot_index
    }
}

impl Default for EntityRegistry {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free_slots: Vec::new(),
            stable_to_internal: HashMap::new(),
        }
    }
}

impl Default for EntitySlot {
    fn default() -> Self {
        Self {
            generation: FIRST_GENERATION,
            stable_id: None,
            location: None,
        }
    }
}

fn next_generation(generation: u32) -> u32 {
    generation.checked_add(1).unwrap_or(FIRST_GENERATION)
}
