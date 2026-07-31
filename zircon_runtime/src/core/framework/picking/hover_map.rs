use std::collections::BTreeMap;

use super::pointer_hits::{hovered_hits_from_sorted, sorted_hits_by_pointer};
use super::{HitRecord, HitTarget, PointerHits, PointerId};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PickingHoverMap {
    hits_by_pointer: BTreeMap<PointerId, Vec<HitRecord>>,
}

impl PickingHoverMap {
    pub fn from_outputs(outputs: &[PointerHits]) -> Self {
        let mut map = Self::default();
        for (pointer, sorted_hits) in sorted_hits_by_pointer(outputs) {
            let hits = hovered_hits_from_sorted(sorted_hits);
            if !hits.is_empty() {
                map.hits_by_pointer.insert(pointer, hits);
            }
        }
        map
    }

    pub fn new(pointer: PointerId, hits: Vec<HitRecord>) -> Self {
        let mut map = Self::default();
        map.set_pointer_hits(pointer, hits);
        map
    }

    pub fn set_pointer_hits(&mut self, pointer: PointerId, hits: Vec<HitRecord>) {
        if hits.is_empty() {
            self.hits_by_pointer.remove(&pointer);
        } else {
            self.hits_by_pointer.insert(pointer, hits);
        }
    }

    pub fn remove_pointer(&mut self, pointer: PointerId) {
        self.hits_by_pointer.remove(&pointer);
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
}
