//! 把稳定目标 ID 绑定到当前评估所用的稠密目标；重复 ID 仅在目标相同时可复用槽。
use std::collections::HashMap;

use zircon_runtime::core::framework::animation::AnimationTargetId;

use super::{TargetSlot, TargetTableError};

/// Per-evaluation dense binding table from stable identity to a resolved runtime target.
#[derive(Clone, Debug)]
pub struct TargetTable<T> {
    slots: TargetSlotMap,
    targets: Vec<T>,
}

const INLINE_TARGET_CAPACITY: usize = 32;

#[derive(Clone, Debug)]
enum TargetSlotMap {
    Inline(Vec<(AnimationTargetId, TargetSlot)>),
    Indexed(HashMap<AnimationTargetId, TargetSlot>),
}

impl Default for TargetSlotMap {
    fn default() -> Self {
        Self::Inline(Vec::new())
    }
}

impl TargetSlotMap {
    fn get(&self, target_id: AnimationTargetId) -> Option<TargetSlot> {
        match self {
            Self::Inline(entries) => entries
                .iter()
                .find_map(|(id, slot)| (*id == target_id).then_some(*slot)),
            Self::Indexed(indexed) => indexed.get(&target_id).copied(),
        }
    }

    fn insert(&mut self, target_id: AnimationTargetId, slot: TargetSlot) {
        match self {
            Self::Inline(entries) if entries.len() < INLINE_TARGET_CAPACITY => {
                entries.push((target_id, slot));
            }
            Self::Inline(entries) => {
                let mut indexed = HashMap::with_capacity(entries.len().saturating_mul(2));
                indexed.extend(entries.drain(..));
                indexed.insert(target_id, slot);
                *self = Self::Indexed(indexed);
            }
            Self::Indexed(indexed) => {
                indexed.insert(target_id, slot);
            }
        }
    }
}

impl<T> Default for TargetTable<T> {
    fn default() -> Self {
        Self {
            slots: TargetSlotMap::default(),
            targets: Vec::new(),
        }
    }
}

impl<T> TargetTable<T>
where
    T: Clone + Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// 同一 ID 再次绑定相同目标返回原槽；不同目标返回冲突，调用方不得覆盖既有身份。
    pub fn bind(
        &mut self,
        target_id: AnimationTargetId,
        target: T,
    ) -> Result<TargetSlot, TargetTableError> {
        if let Some(slot) = self.slots.get(target_id) {
            let existing = &self.targets[slot.index() as usize];
            return if existing == &target {
                Ok(slot)
            } else {
                Err(TargetTableError::ConflictingBinding { target_id })
            };
        }

        let index =
            u32::try_from(self.targets.len()).map_err(|_| TargetTableError::CapacityExceeded)?;
        let slot = TargetSlot::new(index);
        self.targets.push(target);
        self.slots.insert(target_id, slot);
        Ok(slot)
    }

    pub fn slot(&self, target_id: AnimationTargetId) -> Option<TargetSlot> {
        self.slots.get(target_id)
    }

    /// 只接受此表此前返回的槽；该类型本身无法证明槽属于当前表。
    pub fn target(&self, slot: TargetSlot) -> Option<&T> {
        self.targets.get(slot.index() as usize)
    }
}

#[cfg(test)]
#[path = "tests/target_table_optimization_batch_20260830co_tests.rs"]
mod optimization_batch_20260830co_tests;
