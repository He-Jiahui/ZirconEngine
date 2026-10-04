//! 以 index/generation 管理 provider 对象生命周期；删除后复用空槽，并通过拆分切片取得两个不同槽位的可变引用。

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

    // TODO: [CR-PHYSICS-BACKEND-0005] high_index 在 split_at_mut 前未做长度检查，只有 high_index > slots.len() 时会 panic；当前两个调用遍历已创建的约束，create_constraint 已验证端点且 destroy_body 拒绝仍被引用的 body，未发现生产路径可传入越界句柄。
    // 未来若扩大调用面，应先边界检查或显式固定前置条件。证据：builtin/runtime.rs::create_constraint/destroy_body/step、jolt/runtime.rs::create_constraint/destroy_body/project_constraints。
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

    // BUG: [CR-PHYSICS-BACKEND-0007] generation 从 1 起步并在 remove 时 wrapping_add().max(1)，同一槽经 2^32−1 次 remove/insert 后会再次成为 generation 1。
    // 仍保留的旧 Copy 句柄随即被 get 接受并可指向新对象；该路径需要极高 churn，但当前没有耗尽保护。证据：insert 复用 free_indices、remove 回绕、get 只比较 index/generation；关联 PHY4-P1-016。
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
