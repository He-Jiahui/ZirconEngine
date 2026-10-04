use std::collections::HashMap;

use super::{BoxedRuntimeSceneSystem, SystemStage};

/// 运行时系统的有序槽位；执行中 take 只留空洞，restore 回原位，永久 remove 才压缩索引。
#[derive(Default)]
pub(super) struct RuntimeSystemSlots {
    slots: Vec<RuntimeSystemSlot>,
    indices: HashMap<String, usize>,
}

impl RuntimeSystemSlots {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn contains(&self, id: &str) -> bool {
        self.indices.contains_key(id)
    }

    pub(super) fn iter(&self) -> RuntimeSystems<'_> {
        RuntimeSystems {
            slots: self.slots.iter(),
        }
    }

    pub(super) fn take(&mut self, id: &str) -> Option<BoxedRuntimeSceneSystem> {
        let index = *self.indices.get(id)?;
        self.slots.get_mut(index)?.system.take()
    }

    pub(super) fn remove(&mut self, id: &str) -> Option<BoxedRuntimeSceneSystem> {
        let index = *self.indices.get(id)?;
        let slot = self.slots.remove(index);
        self.indices.remove(id);
        self.rebuild_indices_from(index);
        slot.system
    }

    pub(super) fn restore(&mut self, system: BoxedRuntimeSceneSystem) {
        let id = system.id();
        let index = *self
            .indices
            .get(id)
            .expect("taken runtime system must retain its registry slot");
        let slot = &mut self.slots[index];
        debug_assert!(slot.system.is_none());
        debug_assert_eq!(slot.id, id);
        slot.system = Some(system);
    }

    pub(super) fn insert(&mut self, system: BoxedRuntimeSceneSystem) {
        let slot = RuntimeSystemSlot::new(system);
        let insert_index = match self
            .slots
            .binary_search_by(|existing| compare_slots(existing, &slot))
        {
            Ok(index) | Err(index) => index,
        };
        let id = slot.id.clone();
        self.slots.insert(insert_index, slot);
        self.indices.insert(id, insert_index);
        self.rebuild_indices_from(insert_index + 1);
    }

    fn rebuild_indices_from(&mut self, start: usize) {
        for (index, slot) in self.slots.iter().enumerate().skip(start) {
            self.indices.insert(slot.id.clone(), index);
        }
    }
}

struct RuntimeSystemSlot {
    id: String,
    stage: SystemStage,
    order: i32,
    system: Option<BoxedRuntimeSceneSystem>,
}

impl RuntimeSystemSlot {
    fn new(system: BoxedRuntimeSceneSystem) -> Self {
        Self {
            id: system.id().to_owned(),
            stage: system.stage(),
            order: system.order(),
            system: Some(system),
        }
    }
}

fn compare_slots(left: &RuntimeSystemSlot, right: &RuntimeSystemSlot) -> std::cmp::Ordering {
    left.stage
        .rank()
        .cmp(&right.stage.rank())
        .then(left.order.cmp(&right.order))
        .then(left.id.cmp(&right.id))
}

pub(crate) struct RuntimeSystems<'registry> {
    slots: std::slice::Iter<'registry, RuntimeSystemSlot>,
}

impl RuntimeSystems<'_> {
    pub(crate) fn iter(&self) -> Self {
        Self {
            slots: self.slots.clone(),
        }
    }
}

impl<'registry> Iterator for RuntimeSystems<'registry> {
    type Item = &'registry BoxedRuntimeSceneSystem;

    fn next(&mut self) -> Option<Self::Item> {
        for slot in self.slots.by_ref() {
            if let Some(system) = slot.system.as_ref() {
                return Some(system);
            }
        }
        None
    }
}

#[cfg(test)]
#[path = "tests/runtime_system_slots.rs"]
mod tests;
