use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

use super::pointer_hits::{hovered_hits_from_sorted, sorted_hits_by_pointer};
use super::{HitRecord, HitTarget, PickingDomain, PickingPointerKey, PointerHits, PointerId};

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
    hits_by_pointer: Arc<BTreeMap<PickingPointerKey, PointerHoverState>>,
}

impl PickingHoverMap {
    pub fn from_outputs(outputs: &[PointerHits]) -> Self {
        let sorted_hits = sorted_hits_by_pointer(outputs);
        Self::from_sorted_hits(sorted_hits)
    }

    pub(super) fn from_sorted_hits(
        sorted_hits_by_pointer: BTreeMap<PickingPointerKey, Vec<HitRecord>>,
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

    pub fn in_viewport(
        pointer: PointerId,
        viewport: crate::core::framework::render::RenderViewportHandle,
        hits: Vec<HitRecord>,
    ) -> Self {
        let mut map = Self::default();
        map.set_key_hits(PickingPointerKey::in_viewport(pointer, viewport), hits);
        map
    }

    /// Explicit legacy-domain mutation; it never overwrites a viewport domain.
    pub fn set_pointer_hits(&mut self, pointer: PointerId, hits: Vec<HitRecord>) {
        self.set_key_hits(PickingPointerKey::legacy(pointer), hits);
    }

    pub fn set_key_hits(&mut self, key: PickingPointerKey, hits: Vec<HitRecord>) {
        let hits_by_pointer = Arc::make_mut(&mut self.hits_by_pointer);
        if hits.is_empty() {
            hits_by_pointer.remove(&key);
        } else {
            hits_by_pointer.insert(key, PointerHoverState::new(hits));
        }
    }

    pub fn remove_pointer(&mut self, pointer: PointerId) {
        Arc::make_mut(&mut self.hits_by_pointer).retain(|key, _| key.pointer != pointer);
    }

    pub fn remove_key(&mut self, key: PickingPointerKey) {
        Arc::make_mut(&mut self.hits_by_pointer).remove(&key);
    }

    fn unique_key(&self, pointer: PointerId) -> Option<PickingPointerKey> {
        let first = PickingPointerKey::legacy(pointer);
        let last = PickingPointerKey::in_viewport(
            pointer,
            crate::core::framework::render::RenderViewportHandle::new(u64::MAX),
        );
        let mut keys = self
            .hits_by_pointer
            .range(first..=last)
            .map(|(key, _)| *key);
        let key = keys.next()?;
        keys.next().is_none().then_some(key)
    }

    /// Compatibility query: multiple domains for this pointer are ambiguous.
    pub fn get(&self, pointer: PointerId) -> &[HitRecord] {
        self.unique_key(pointer)
            .map(|key| self.get_key(key))
            .unwrap_or(&[])
    }

    pub fn get_key(&self, key: PickingPointerKey) -> &[HitRecord] {
        self.hits_by_pointer
            .get(&key)
            .map(|state| state.hits.as_slice())
            .unwrap_or(&[])
    }

    pub fn hit(&self, pointer: PointerId, target: HitTarget) -> Option<&HitRecord> {
        self.get(pointer).iter().find(|hit| hit.target == target)
    }

    pub fn hit_key(&self, key: PickingPointerKey, target: HitTarget) -> Option<&HitRecord> {
        self.get_key(key).iter().find(|hit| hit.target == target)
    }

    pub fn is_hovered(&self, pointer: PointerId, target: HitTarget) -> bool {
        self.unique_key(pointer)
            .is_some_and(|key| self.is_hovered_key(key, target))
    }

    pub fn is_hovered_key(&self, key: PickingPointerKey, target: HitTarget) -> bool {
        self.hits_by_pointer
            .get(&key)
            .is_some_and(|state| state.targets.contains(&target))
    }

    pub fn iter(&self) -> impl Iterator<Item = (PointerId, &[HitRecord])> {
        self.iter_keys().map(|(key, hits)| (key.pointer, hits))
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = (PickingPointerKey, &[HitRecord])> {
        self.hits_by_pointer
            .iter()
            .map(|(key, state)| (*key, state.hits.as_slice()))
    }

    pub fn pointer_ids(&self) -> impl Iterator<Item = PointerId> + '_ {
        self.hits_by_pointer
            .keys()
            .map(|key| key.pointer)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
    }

    pub(super) fn retain_mode(&mut self, scoped: bool) {
        if self
            .hits_by_pointer
            .keys()
            .any(|key| matches!(key.domain, PickingDomain::Viewport(_)) != scoped)
        {
            Arc::make_mut(&mut self.hits_by_pointer)
                .retain(|key, _| matches!(key.domain, PickingDomain::Viewport(_)) == scoped);
        }
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
