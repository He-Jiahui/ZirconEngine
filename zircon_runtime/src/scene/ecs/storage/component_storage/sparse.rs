use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::scene::ecs::{ChangeTick, ComponentTicks, InternalEntity};

use super::entry::{RawRemoveResult, StoredComponent};

#[derive(Default)]
pub(in crate::scene::ecs::storage) struct SparseComponentStorage {
    entries: HashMap<InternalEntity, SparseEntry>,
}

struct SparseEntry {
    value: StoredComponent,
    ticks: ComponentTicks,
}

impl SparseComponentStorage {
    pub(super) fn insert(
        &mut self,
        entity: InternalEntity,
        value: StoredComponent,
        tick: ChangeTick,
    ) -> Option<StoredComponent> {
        match self.entries.entry(entity) {
            Entry::Occupied(mut occupied) => {
                let entry = occupied.get_mut();
                entry.ticks.set_changed(tick);
                Some(std::mem::replace(&mut entry.value, value))
            }
            Entry::Vacant(vacant) => {
                vacant.insert(SparseEntry {
                    value,
                    ticks: ComponentTicks::new(tick),
                });
                None
            }
        }
    }

    pub(super) fn get<T>(&self, entity: InternalEntity) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entries.get(&entity)?;
        entry.value.downcast_ref::<T>()
    }

    pub(super) fn get_with_ticks<T>(&self, entity: InternalEntity) -> Option<(&T, ComponentTicks)>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entries.get(&entity)?;
        let value = entry.value.downcast_ref::<T>()?;
        Some((value, entry.ticks))
    }

    pub(super) fn get_mut<T>(&mut self, entity: InternalEntity) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entries.get_mut(&entity)?;
        entry.value.downcast_mut::<T>()
    }

    pub(super) fn get_mut_at_tick<T>(
        &mut self,
        entity: InternalEntity,
        tick: ChangeTick,
    ) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entries.get_mut(&entity)?;
        entry.ticks.set_changed(tick);
        entry.value.downcast_mut::<T>()
    }

    pub(super) fn remove(&mut self, entity: InternalEntity) -> Option<RawRemoveResult> {
        let Some(entry) = self.entries.remove(&entity) else {
            return None;
        };
        Some(RawRemoveResult {
            value: entry.value,
            swapped_entity: None,
        })
    }

    pub(super) fn contains(&self, entity: InternalEntity) -> bool {
        self.entries.contains_key(&entity)
    }

    pub(super) fn ticks(&self, entity: InternalEntity) -> Option<ComponentTicks> {
        let entry = self.entries.get(&entity)?;
        Some(entry.ticks)
    }

    pub(super) fn mark_changed(&mut self, entity: InternalEntity, tick: ChangeTick) {
        if let Some(entry) = self.entries.get_mut(&entity) {
            entry.ticks.set_changed(tick);
        }
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}
