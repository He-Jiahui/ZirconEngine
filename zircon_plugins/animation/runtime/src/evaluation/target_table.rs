use std::collections::BTreeMap;

use zircon_runtime::core::framework::animation::AnimationTargetId;

use super::{TargetSlot, TargetTableError};

/// Per-evaluation dense binding table from stable identity to a resolved runtime target.
#[derive(Clone, Debug)]
pub struct TargetTable<T> {
    slots: BTreeMap<AnimationTargetId, TargetSlot>,
    targets: Vec<T>,
}

impl<T> Default for TargetTable<T> {
    fn default() -> Self {
        Self {
            slots: BTreeMap::new(),
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

    pub fn bind(
        &mut self,
        target_id: AnimationTargetId,
        target: T,
    ) -> Result<TargetSlot, TargetTableError> {
        if let Some(slot) = self.slots.get(&target_id).copied() {
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
        self.slots.get(&target_id).copied()
    }

    pub fn target(&self, slot: TargetSlot) -> Option<&T> {
        self.targets.get(slot.index() as usize)
    }
}
