use std::collections::{HashSet, VecDeque};

use crate::asset::AssetId;

/// A FIFO where each asset owns at most one physical queue slot.
#[derive(Debug, Default)]
pub(super) struct AssetIdOrder {
    order: VecDeque<AssetId>,
    queued: HashSet<AssetId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::resource::ResourceId;

    #[test]
    fn dynamic_scene_asset_reload_order_keeps_one_physical_slot_per_asset() {
        let asset_id = ResourceId::from_stable_label("reload-order");
        let mut order = AssetIdOrder::default();

        for _ in 0..10_000 {
            order.push_back(asset_id);
        }

        assert_eq!(order.len(), 1);
        assert_eq!(order.pop_front(), Some(asset_id));
        assert!(order.is_empty());
    }

    #[test]
    fn dynamic_scene_asset_reload_asset_order_scale_matrix_keeps_linear_physical_work() {
        for asset_count in [1usize, 1_000, 100_000] {
            let mut order = AssetIdOrder::default();
            let asset_ids = (0..asset_count)
                .map(|index| ResourceId::from_stable_label(&format!("reload-scale-{index}")))
                .collect::<Vec<_>>();

            for asset_id in &asset_ids {
                assert!(order.push_back(*asset_id));
                assert!(!order.push_back(*asset_id));
            }
            assert_eq!(order.len(), asset_count);

            for expected in asset_ids {
                assert_eq!(order.pop_front(), Some(expected));
            }
            assert!(order.is_empty());
        }
    }
}

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
