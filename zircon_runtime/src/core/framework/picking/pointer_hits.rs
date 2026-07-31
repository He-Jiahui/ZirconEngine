use std::collections::BTreeMap;

use crate::core::math::Real;

use super::{HitRecord, PointerId};

#[derive(Clone, Debug, PartialEq)]
pub struct PointerHits {
    pub pointer: PointerId,
    pub hits: Vec<HitRecord>,
    pub order: Real,
}

impl PointerHits {
    pub fn new(pointer: PointerId, hits: Vec<HitRecord>, order: Real) -> Self {
        Self {
            pointer,
            hits,
            order,
        }
    }
}

pub fn sorted_hits_for_pointer(outputs: &[PointerHits], pointer: PointerId) -> Vec<HitRecord> {
    let mut indexed: Vec<IndexedHit> = outputs
        .iter()
        .enumerate()
        .filter(|(_, output)| output.pointer == pointer)
        .flat_map(|(output_index, output)| {
            output
                .hits
                .iter()
                .cloned()
                .enumerate()
                .map(move |(hit_index, hit)| (output_index, hit_index, output.order, hit))
        })
        .collect();

    sort_indexed_hits(&mut indexed);
    indexed.into_iter().map(|(_, _, _, hit)| hit).collect()
}

pub(super) fn sorted_hits_by_pointer(
    outputs: &[PointerHits],
) -> BTreeMap<PointerId, Vec<HitRecord>> {
    let mut indexed_by_pointer = BTreeMap::<PointerId, Vec<IndexedHit>>::new();
    for (output_index, output) in outputs.iter().enumerate() {
        indexed_by_pointer
            .entry(output.pointer)
            .or_default()
            .extend(
                output
                    .hits
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|(hit_index, hit)| (output_index, hit_index, output.order, hit)),
            );
    }

    indexed_by_pointer
        .into_iter()
        .map(|(pointer, mut indexed)| {
            sort_indexed_hits(&mut indexed);
            let hits = indexed.into_iter().map(|(_, _, _, hit)| hit).collect();
            (pointer, hits)
        })
        .collect()
}

type IndexedHit = (usize, usize, Real, HitRecord);

fn sort_indexed_hits(indexed: &mut [IndexedHit]) {
    indexed.sort_by(|left, right| {
        left.3
            .target
            .priority()
            .cmp(&right.3.target.priority())
            .then_with(|| right.2.total_cmp(&left.2))
            .then_with(|| left.3.hit.depth.total_cmp(&right.3.hit.depth))
            .then_with(|| left.0.cmp(&right.0))
            .then_with(|| left.1.cmp(&right.1))
    });
}

pub fn hovered_hits_for_pointer(outputs: &[PointerHits], pointer: PointerId) -> Vec<HitRecord> {
    hovered_hits_from_sorted(sorted_hits_for_pointer(outputs, pointer))
}

pub(super) fn hovered_hits_from_sorted(sorted_hits: Vec<HitRecord>) -> Vec<HitRecord> {
    let mut hovered = Vec::new();
    for hit in sorted_hits {
        let should_block_lower = hit.pickable.should_block_lower;
        if hit.pickable.is_hoverable {
            hovered.push(hit);
        }
        if should_block_lower {
            break;
        }
    }
    hovered
}
