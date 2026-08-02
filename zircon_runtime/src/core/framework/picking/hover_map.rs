use std::{collections::BTreeMap, sync::Arc};

use super::pointer_hits::{hovered_hits_from_sorted, sorted_hits_by_pointer};
use super::{HitRecord, HitTarget, PointerHits, PointerId};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PickingHoverMap {
    hits_by_pointer: Arc<BTreeMap<PointerId, Vec<HitRecord>>>,
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
                hits_by_pointer.insert(pointer, hits);
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

    pub fn set_pointer_hits(&mut self, pointer: PointerId, hits: Vec<HitRecord>) {
        let hits_by_pointer = Arc::make_mut(&mut self.hits_by_pointer);
        if hits.is_empty() {
            hits_by_pointer.remove(&pointer);
        } else {
            hits_by_pointer.insert(pointer, hits);
        }
    }

    pub fn remove_pointer(&mut self, pointer: PointerId) {
        Arc::make_mut(&mut self.hits_by_pointer).remove(&pointer);
    }

    pub fn get(&self, pointer: PointerId) -> &[HitRecord] {
        self.hits_by_pointer
            .get(&pointer)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn hit(&self, pointer: PointerId, target: HitTarget) -> Option<&HitRecord> {
        self.get(pointer).iter().find(|hit| hit.target == target)
    }

    pub fn is_hovered(&self, pointer: PointerId, target: HitTarget) -> bool {
        self.hit(pointer, target).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = (PointerId, &[HitRecord])> {
        self.hits_by_pointer
            .iter()
            .map(|(pointer, hits)| (*pointer, hits.as_slice()))
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
