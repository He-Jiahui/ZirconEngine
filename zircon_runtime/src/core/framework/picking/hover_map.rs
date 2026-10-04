use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

use super::pointer_hits::{hovered_hits_from_sorted, sorted_hits_by_pointer};
use super::{HitRecord, HitTarget, PointerHits, PointerId};

#[derive(Clone, Debug, PartialEq)]
struct PointerHoverState {
    hits: Vec<HitRecord>,
    targets: HashSet<HitTarget>,
}

impl PointerHoverState {
    fn new(hits: Vec<HitRecord>) -> Self {
        let targets = hits.iter().map(|hit| hit.target).collect();
        Self { hits, targets }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
/// 已按优先级与遮挡规则筛出的每指针悬停快照；事件状态机对比相邻帧以产生 Enter/Leave。
pub struct PickingHoverMap {
    hits_by_pointer: Arc<BTreeMap<PointerId, PointerHoverState>>,
}

impl PickingHoverMap {
    pub fn from_outputs(outputs: &[PointerHits]) -> Self {
        let sorted_hits = sorted_hits_by_pointer(outputs);
        Self::from_sorted_hits(sorted_hits)
    }

    pub(super) fn from_sorted_hits(
        sorted_hits_by_pointer: BTreeMap<PointerId, Vec<HitRecord>>,
    ) -> Self {
        let mut hits_by_pointer = BTreeMap::new();
        for (pointer, sorted_hits) in sorted_hits_by_pointer {
            let hits = hovered_hits_from_sorted(sorted_hits);
            if !hits.is_empty() {
                hits_by_pointer.insert(pointer, PointerHoverState::new(hits));
            }
        }
        Self {
            hits_by_pointer: Arc::new(hits_by_pointer),
        }
    }

    pub fn new(pointer: PointerId, hits: Vec<HitRecord>) -> Self {
        let mut map = Self::default();
        map.set_pointer_hits(pointer, hits);
        map
    }

    /// 手工覆盖单指针快照时，调用方须提供预期顺序与遮挡裁剪；这里不会重新解析后端命中。
    pub fn set_pointer_hits(&mut self, pointer: PointerId, hits: Vec<HitRecord>) {
        let hits_by_pointer = Arc::make_mut(&mut self.hits_by_pointer);
        if hits.is_empty() {
            hits_by_pointer.remove(&pointer);
        } else {
            hits_by_pointer.insert(pointer, PointerHoverState::new(hits));
        }
    }

    pub fn remove_pointer(&mut self, pointer: PointerId) {
        Arc::make_mut(&mut self.hits_by_pointer).remove(&pointer);
    }

    pub fn get(&self, pointer: PointerId) -> &[HitRecord] {
        self.hits_by_pointer
            .get(&pointer)
            .map(|state| state.hits.as_slice())
            .unwrap_or(&[])
    }

    pub fn hit(&self, pointer: PointerId, target: HitTarget) -> Option<&HitRecord> {
        self.get(pointer).iter().find(|hit| hit.target == target)
    }

    pub fn is_hovered(&self, pointer: PointerId, target: HitTarget) -> bool {
        self.hits_by_pointer
            .get(&pointer)
            .is_some_and(|state| state.targets.contains(&target))
    }

    pub fn iter(&self) -> impl Iterator<Item = (PointerId, &[HitRecord])> {
        self.hits_by_pointer
            .iter()
            .map(|(pointer, state)| (*pointer, state.hits.as_slice()))
    }

    pub fn pointer_ids(&self) -> impl Iterator<Item = PointerId> + '_ {
        self.hits_by_pointer.keys().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.hits_by_pointer.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.hits_by_pointer, &other.hits_by_pointer)
    }
}

#[cfg(test)]
#[path = "tests/hover_map.rs"]
mod tests;
