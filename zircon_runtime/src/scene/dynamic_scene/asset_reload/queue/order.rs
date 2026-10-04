use std::collections::{HashSet, VecDeque};

use crate::asset::AssetId;

/// A FIFO where each asset owns at most one physical queue slot.
#[derive(Debug, Default)]
pub(super) struct AssetIdOrder {
    order: VecDeque<AssetId>,
    queued: HashSet<AssetId>,
}

#[cfg(test)]
#[path = "tests/order.rs"]
mod tests;

impl AssetIdOrder {
    pub(super) fn push_back(&mut self, asset_id: AssetId) -> bool {
        if !self.queued.insert(asset_id) {
            return false;
        }
        self.order.push_back(asset_id);
        true
    }

    pub(super) fn pop_front(&mut self) -> Option<AssetId> {
        let asset_id = self.order.pop_front()?;
        let removed = self.queued.remove(&asset_id);
        debug_assert!(removed);
        Some(asset_id)
    }

    pub(super) fn contains(&self, asset_id: AssetId) -> bool {
        self.queued.contains(&asset_id)
    }

    pub(super) fn len(&self) -> usize {
        self.order.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub(super) fn clear(&mut self) {
        self.order.clear();
        self.queued.clear();
    }
}
