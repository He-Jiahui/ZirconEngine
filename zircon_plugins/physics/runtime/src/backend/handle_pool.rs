use std::marker::PhantomData;

use super::handles::ArenaHandle;

const INITIAL_GENERATION: u32 = 1;

pub(super) struct HandlePool<T, H> {
    slots: Vec<HandleSlot<T>>,
    free_indices: Vec<u32>,
    marker: PhantomData<H>,
}

struct HandleSlot<T> {
    generation: u32,
    value: Option<T>,
}

impl<T, H> Default for HandlePool<T, H> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free_indices: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<T, H> HandlePool<T, H>
where
    H: ArenaHandle,
{
    pub(super) fn insert(&mut self, value: T) -> Option<H> {
        if let Some(index) = self.free_indices.pop() {
            let slot = &mut self.slots[index as usize];
            slot.value = Some(value);
            return Some(H::from_raw(pack_handle(index, slot.generation)));
        }

        let index = u32::try_from(self.slots.len()).ok()?;
        self.slots.push(HandleSlot {
            generation: INITIAL_GENERATION,
            value: Some(value),
        });
        Some(H::from_raw(pack_handle(index, INITIAL_GENERATION)))
    }

    pub(super) fn get(&self, handle: H) -> Option<&T> {
        let (index, generation) = unpack_handle(handle.raw());
        self.slots
            .get(index as usize)
            .filter(|slot| slot.generation == generation)
            .and_then(|slot| slot.value.as_ref())
    }

    pub(super) fn get_mut(&mut self, handle: H) -> Option<&mut T> {
        let (index, generation) = unpack_handle(handle.raw());
        self.slots
            .get_mut(index as usize)
            .filter(|slot| slot.generation == generation)
            .and_then(|slot| slot.value.as_mut())
    }

    pub(super) fn remove(&mut self, handle: H) -> Option<T> {
        let (index, generation) = unpack_handle(handle.raw());
        let slot = self.slots.get_mut(index as usize)?;
        if slot.generation != generation {
            return None;
        }
        let value = slot.value.take()?;
        slot.generation = slot.generation.wrapping_add(1).max(INITIAL_GENERATION);
        self.free_indices.push(index);
        Some(value)
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (H, &T)> {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            let value = slot.value.as_ref()?;
            let index = u32::try_from(index).ok()?;
            Some((H::from_raw(pack_handle(index, slot.generation)), value))
        })
    }

    pub(super) fn iter_mut(&mut self) -> impl Iterator<Item = (H, &mut T)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(index, slot)| {
                let value = slot.value.as_mut()?;
                let index = u32::try_from(index).ok()?;
                Some((H::from_raw(pack_handle(index, slot.generation)), value))
            })
    }
}

fn pack_handle(index: u32, generation: u32) -> u64 {
    u64::from(generation) << u32::BITS | u64::from(index)
}

fn unpack_handle(raw: u64) -> (u32, u32) {
    (raw as u32, (raw >> u32::BITS) as u32)
}
