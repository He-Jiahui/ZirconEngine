use std::collections::{HashMap, HashSet};

use crate::scene::ecs::ArchetypeId;
use crate::scene::EntityId;

use super::despawned::DespawnedEntity;
use super::error::EntityRegistryError;
use super::internal::InternalEntity;
use super::location::EntityLocation;
use super::slot::{next_generation, EntitySlot};
use super::stable_location::StableEntityLocation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EntityRegistry {
    slots: Vec<EntitySlot>,
    free_slots: Vec<u32>,
    stable_to_internal: HashMap<EntityId, InternalEntity>,
    max_slots: u32,
}

impl EntityRegistry {
    pub(crate) fn spawn(
        &mut self,
        stable_id: EntityId,
        location: EntityLocation,
    ) -> Result<InternalEntity, EntityRegistryError> {
        if self.stable_to_internal.contains_key(&stable_id) {
            return Err(EntityRegistryError::DuplicateStableId(stable_id));
        }

        self.spawn_prevalidated(stable_id, location)
    }

    pub(crate) fn spawn_prevalidated(
        &mut self,
        stable_id: EntityId,
        location: EntityLocation,
    ) -> Result<InternalEntity, EntityRegistryError> {
        if self.stable_to_internal.contains_key(&stable_id) {
            return Err(EntityRegistryError::DuplicateStableId(stable_id));
        }

        let slot_index = self.allocate_slot()?;
        let slot = &mut self.slots[slot_index as usize];
        let internal = InternalEntity::new(slot_index, slot.generation);
        slot.stable_id = Some(stable_id);
        slot.location = Some(location);
        self.stable_to_internal.insert(stable_id, internal);
        Ok(internal)
    }

    pub(crate) fn despawn(
        &mut self,
        stable_id: EntityId,
    ) -> Result<DespawnedEntity, EntityRegistryError> {
        let Some(internal) = self.stable_to_internal.get(&stable_id).copied() else {
            return Err(EntityRegistryError::MissingStableId(stable_id));
        };
        let Some(slot) = self.slots.get_mut(internal.index() as usize) else {
            return Err(EntityRegistryError::InvalidInternalEntity);
        };
        if slot.generation != internal.generation() || slot.stable_id != Some(stable_id) {
            return Err(EntityRegistryError::InvalidInternalEntity);
        }

        let location = match slot.location.take() {
            Some(location) => location,
            None => EntityLocation::default(),
        };
        slot.stable_id = None;
        let reusable_slot = if let Some(next_generation) = next_generation(slot.generation) {
            slot.generation = next_generation;
            true
        } else {
            // Reusing this slot would make a stale InternalEntity valid after a wrap.
            false
        };
        self.stable_to_internal.remove(&stable_id);
        if reusable_slot {
            self.free_slots.push(internal.index());
        }
        Ok(DespawnedEntity {
            stable_id,
            internal,
            location,
        })
    }

    pub(crate) fn clear(&mut self) {
        self.slots.clear();
        self.free_slots.clear();
        self.stable_to_internal.clear();
    }

    // 仅用于投影重建：先验证整批身份与容量，再清空运行时槽；旧内部句柄不能跨重建复用。
    pub(crate) fn rebuild_from_stable_ids<I>(
        &mut self,
        stable_ids: I,
    ) -> Result<(), EntityRegistryError>
    where
        I: IntoIterator<Item = EntityId>,
    {
        let stable_ids = stable_ids.into_iter().collect::<Vec<_>>();
        let mut unique_stable_ids = HashSet::with_capacity(stable_ids.len());
        for stable_id in &stable_ids {
            if !unique_stable_ids.insert(*stable_id) {
                return Err(EntityRegistryError::DuplicateStableId(*stable_id));
            }
        }
        self.ensure_total_slot_count(stable_ids.len())?;
        self.clear();
        for (row, stable_id) in stable_ids.into_iter().enumerate() {
            self.spawn(stable_id, EntityLocation::new(ArchetypeId::EMPTY, row))?;
        }
        Ok(())
    }

    pub(crate) fn set_location(
        &mut self,
        stable_id: EntityId,
        location: EntityLocation,
    ) -> Result<(), EntityRegistryError> {
        let Some(internal) = self.internal_for_stable(stable_id) else {
            return Err(EntityRegistryError::MissingStableId(stable_id));
        };
        let Some(slot) = self.slots.get_mut(internal.index() as usize) else {
            return Err(EntityRegistryError::InvalidInternalEntity);
        };
        if slot.generation != internal.generation() || slot.stable_id != Some(stable_id) {
            return Err(EntityRegistryError::InvalidInternalEntity);
        }
        slot.location = Some(location);
        Ok(())
    }

    pub(crate) fn contains_internal(&self, internal: InternalEntity) -> bool {
        self.location_for_internal(internal).is_ok()
    }

    pub(crate) fn contains_stable(&self, stable_id: EntityId) -> bool {
        self.stable_to_internal.contains_key(&stable_id)
    }

    pub(crate) fn internal_for_stable(&self, stable_id: EntityId) -> Option<InternalEntity> {
        self.stable_to_internal.get(&stable_id).copied()
    }

    pub(crate) fn location_for_stable(&self, stable_id: EntityId) -> Option<StableEntityLocation> {
        let Some(internal) = self.internal_for_stable(stable_id) else {
            return None;
        };
        self.location_for_internal(internal).ok()
    }

    pub(crate) fn location_for_internal(
        &self,
        internal: InternalEntity,
    ) -> Result<StableEntityLocation, EntityRegistryError> {
        let Some(slot) = self.slots.get(internal.index() as usize) else {
            return Err(EntityRegistryError::InvalidInternalEntity);
        };
        if slot.generation != internal.generation() {
            return Err(EntityRegistryError::InvalidInternalEntity);
        }
        let Some(stable_id) = slot.stable_id else {
            return Err(EntityRegistryError::InvalidInternalEntity);
        };
        let Some(location) = slot.location else {
            return Err(EntityRegistryError::InvalidInternalEntity);
        };
        Ok(StableEntityLocation {
            stable_id,
            internal,
            location,
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.stable_to_internal.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.stable_to_internal.is_empty()
    }

    pub(crate) fn ensure_capacity_for_additional(
        &self,
        additional_entities: usize,
    ) -> Result<(), EntityRegistryError> {
        let reusable_slots = self.free_slots.len();
        let new_slots = if additional_entities > reusable_slots {
            additional_entities - reusable_slots
        } else {
            0
        };
        let Some(total_slots) = self.slots.len().checked_add(new_slots) else {
            return Err(self.capacity_error());
        };
        self.ensure_total_slot_count(total_slots)
    }

    fn allocate_slot(&mut self) -> Result<u32, EntityRegistryError> {
        if let Some(slot_index) = self.free_slots.pop() {
            return Ok(slot_index);
        }
        self.ensure_capacity_for_additional(1)?;
        let slot_index = u32::try_from(self.slots.len()).map_err(|_| self.capacity_error())?;
        debug_assert_ne!(slot_index, InternalEntity::INVALID_INDEX);
        self.slots.push(EntitySlot::default());
        Ok(slot_index)
    }

    fn ensure_total_slot_count(&self, total_slots: usize) -> Result<(), EntityRegistryError> {
        if total_slots > self.max_slots as usize {
            return Err(self.capacity_error());
        }
        Ok(())
    }

    fn capacity_error(&self) -> EntityRegistryError {
        EntityRegistryError::SlotCapacityExhausted {
            max_slots: self.max_slots,
        }
    }

    fn with_max_slots(max_slots: u32) -> Self {
        Self {
            slots: Vec::new(),
            free_slots: Vec::new(),
            stable_to_internal: HashMap::new(),
            max_slots,
        }
    }
}

impl Default for EntityRegistry {
    fn default() -> Self {
        Self::with_max_slots(InternalEntity::INVALID_INDEX)
    }
}

#[cfg(test)]
#[path = "tests/registry.rs"]
mod tests;
