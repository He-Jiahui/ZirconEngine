#[cfg(test)]
use std::cell::Cell;
use std::collections::{BTreeMap, HashMap};

use crate::core::math::Real;

use super::{HitRecord, PointerId};

#[cfg(test)]
#[path = "pointer_hits/tests/hash_grouping_tests.rs"]
mod hash_grouping_tests;

#[cfg(test)]
#[path = "pointer_hits/tests/optimization_batch_iz_runtime638_tests.rs"]
mod optimization_batch_iz_runtime638_tests;

#[derive(Clone, Debug, PartialEq)]
/// 一批后端为单个指针返回的命中记录及批次顺序键，供跨批次排序和悬停投影使用。
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

/// 合并指定指针的后端命中，并按目标优先级、批次顺序、命中深度及来源索引稳定排序。
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

/// 为整帧结果一次性按指针分组排序，供悬停投影和报告共享同一顺序。
pub(super) fn sorted_hits_by_pointer(
    outputs: &[PointerHits],
) -> BTreeMap<PointerId, Vec<HitRecord>> {
    #[cfg(test)]
    SORTED_HIT_PROJECTION_BUILDS.with(|count| count.set(count.get() + 1));

    let mut indexed_by_pointer =
        HashMap::<PointerId, Vec<IndexedHit>>::with_capacity(outputs.len());
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
        .collect::<BTreeMap<_, _>>()
}

type IndexedHit = (usize, usize, Real, HitRecord);

fn sort_indexed_hits(indexed: &mut [IndexedHit]) {
    #[cfg(test)]
    SORTED_HIT_POINTER_GROUP_SORTS.with(|count| count.set(count.get() + 1));

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
    let sorted_hits = sorted_hits_for_pointer(outputs, pointer);
    hovered_hits_from_sorted(sorted_hits)
}

/// 从已排序命中中收集可悬停目标；首个阻挡命中会被纳入（若可悬停）后停止向下遍历。
pub(super) fn hovered_hits_from_sorted(sorted_hits: Vec<HitRecord>) -> Vec<HitRecord> {
    let mut hovered = Vec::with_capacity(sorted_hits.len());
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

#[cfg(test)]
std::thread_local! {
    static SORTED_HIT_PROJECTION_BUILDS: Cell<usize> = const { Cell::new(0) };
    static SORTED_HIT_POINTER_GROUP_SORTS: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_sorted_hit_projection_metrics() {
    SORTED_HIT_PROJECTION_BUILDS.with(|count| count.set(0));
    SORTED_HIT_POINTER_GROUP_SORTS.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn sorted_hit_projection_metrics() -> (usize, usize) {
    let builds = SORTED_HIT_PROJECTION_BUILDS.with(Cell::get);
    let pointer_group_sorts = SORTED_HIT_POINTER_GROUP_SORTS.with(Cell::get);
    (builds, pointer_group_sorts)
}
