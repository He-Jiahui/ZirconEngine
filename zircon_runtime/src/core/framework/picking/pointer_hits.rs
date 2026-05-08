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
    let mut indexed: Vec<(usize, usize, Real, HitRecord)> = outputs
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

    indexed.into_iter().map(|(_, _, _, hit)| hit).collect()
}

pub fn hovered_hits_for_pointer(outputs: &[PointerHits], pointer: PointerId) -> Vec<HitRecord> {
    let mut hovered = Vec::new();
    for hit in sorted_hits_for_pointer(outputs, pointer) {
        if hit.pickable.is_hoverable {
            hovered.push(hit.clone());
        }
        if hit.pickable.should_block_lower {
            break;
        }
    }
    hovered
}
