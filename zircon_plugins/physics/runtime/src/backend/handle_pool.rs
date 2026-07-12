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

    pub(super) fn get_pair_mut(&mut self, a: H, b: H) -> Option<(&mut T, &mut T)> {
        let (a_index, a_generation) = unpack_handle(a.raw());
        let (b_index, b_generation) = unpack_handle(b.raw());
        if a_index == b_index {
            return None;
        }
        let (low_index, low_generation, high_index, high_generation, swapped) = if a_index < b_index
        {
            (a_index, a_generation, b_index, b_generation, false)
        } else {
            (b_index, b_generation, a_index, a_generation, true)
        };
        let (low, high) = self.slots.split_at_mut(high_index as usize);
        let low = low.get_mut(low_index as usize)?;
        let high = high.first_mut()?;
        if low.generation != low_generation || high.generation != high_generation {
            return None;
        }
        let low = low.value.as_mut()?;
        let high = high.value.as_mut()?;
        if swapped {
            Some((high, low))
        } else {
            Some((low, high))
        }
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
