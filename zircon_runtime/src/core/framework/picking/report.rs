use std::collections::{BTreeMap, BTreeSet};

use super::pointer_hits::sorted_hits_by_pointer;
use super::{HitRecord, HitTarget, PointerHits, PointerId, RayMap};

#[cfg(test)]
mod single_pass_tests;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PickingPipelineReport {
    pub ray_count: usize,
    pub pointer_count: usize,
    pub backend_output_count: usize,
    pub raw_hit_count: usize,
    pub hovered_hit_count: usize,
    pub blocked_pointer_count: usize,
    pub pointers: Vec<PickingPointerPipelineReport>,
}

impl PickingPipelineReport {
    pub fn from_outputs(outputs: &[PointerHits]) -> Self {
        Self::from_ray_map_and_outputs(&RayMap::default(), outputs)
    }

    pub fn from_ray_map_and_outputs(ray_map: &RayMap, outputs: &[PointerHits]) -> Self {
        let sorted_hits = sorted_hits_by_pointer(outputs);
        Self::from_ray_map_outputs_and_sorted_hits(ray_map, outputs, &sorted_hits)
    }

    pub(super) fn from_ray_map_outputs_and_sorted_hits(
        ray_map: &RayMap,
        outputs: &[PointerHits],
        sorted_hits_by_pointer: &BTreeMap<PointerId, Vec<HitRecord>>,
    ) -> Self {
        let ray_count_by_pointer = ray_count_by_pointer(ray_map);
        let output_counts_by_pointer = output_counts_by_pointer(outputs);
        let pointers = report_pointer_ids(ray_map, outputs)
            .into_iter()
            .map(|pointer| {
                let (backend_output_count, raw_hit_count) = output_counts_by_pointer
                    .get(&pointer)
                    .copied()
                    .unwrap_or_default();
                PickingPointerPipelineReport::from_pointer(
                    pointer,
                    *ray_count_by_pointer.get(&pointer).unwrap_or(&0),
                    sorted_hits_by_pointer
                        .get(&pointer)
                        .map(Vec::as_slice)
                        .unwrap_or(&[]),
                    backend_output_count,
                    raw_hit_count,
                )
            })
            .collect::<Vec<_>>();

        let hovered_hit_count = pointers
            .iter()
            .map(|pointer| pointer.hovered_hit_count)
            .sum();
        let blocked_pointer_count = pointers
            .iter()
            .filter(|pointer| pointer.blocking_target.is_some())
            .count();

        Self {
            ray_count: ray_map.len(),
            pointer_count: pointers.len(),
            backend_output_count: outputs.len(),
            raw_hit_count: outputs.iter().map(|output| output.hits.len()).sum(),
            hovered_hit_count,
            blocked_pointer_count,
            pointers,
        }
    }

    pub fn pointer(&self, pointer: PointerId) -> Option<&PickingPointerPipelineReport> {
        self.pointers
            .iter()
            .find(|pointer_report| pointer_report.pointer == pointer)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PickingPointerPipelineReport {
    pub pointer: PointerId,
    pub ray_count: usize,
    pub backend_output_count: usize,
    pub raw_hit_count: usize,
    pub sorted_hit_count: usize,
    pub hovered_hit_count: usize,
    pub non_hoverable_hit_count: usize,
    pub top_target: Option<HitTarget>,
    pub blocking_target: Option<HitTarget>,
}

impl PickingPointerPipelineReport {
    fn from_pointer(
        pointer: PointerId,
        ray_count: usize,
        sorted_hits: &[HitRecord],
        backend_output_count: usize,
        raw_hit_count: usize,
    ) -> Self {
        let summary = summarize_pointer_hits(sorted_hits);

        Self {
            pointer,
            ray_count,
            backend_output_count,
            raw_hit_count,
            sorted_hit_count: sorted_hits.len(),
            hovered_hit_count: summary.hovered_hit_count,
            non_hoverable_hit_count: summary.non_hoverable_hit_count,
            top_target: sorted_hits.first().map(|hit| hit.target),
            blocking_target: summary.blocking_target,
        }
    }
}

struct PointerHitSummary {
    hovered_hit_count: usize,
    non_hoverable_hit_count: usize,
    blocking_target: Option<HitTarget>,
}

fn summarize_pointer_hits(sorted_hits: &[HitRecord]) -> PointerHitSummary {
    let mut summary = PointerHitSummary {
        hovered_hit_count: 0,
        non_hoverable_hit_count: 0,
        blocking_target: None,
    };
    for hit in sorted_hits {
        if summary.blocking_target.is_none() && hit.pickable.is_hoverable {
            summary.hovered_hit_count += 1;
        }
        if !hit.pickable.is_hoverable {
            summary.non_hoverable_hit_count += 1;
        }
        if summary.blocking_target.is_none() && hit.pickable.should_block_lower {
            summary.blocking_target = Some(hit.target);
        }
    }
    summary
}

fn output_counts_by_pointer(outputs: &[PointerHits]) -> BTreeMap<PointerId, (usize, usize)> {
    let mut counts = BTreeMap::new();
    for output in outputs {
        let (backend_output_count, raw_hit_count) = counts.entry(output.pointer).or_default();
        *backend_output_count += 1;
        *raw_hit_count += output.hits.len();
    }
    counts
}

fn report_pointer_ids(ray_map: &RayMap, outputs: &[PointerHits]) -> BTreeSet<PointerId> {
    ray_map
        .iter()
        .map(|(ray_id, _)| ray_id.pointer)
        .chain(outputs.iter().map(|output| output.pointer))
        .collect()
}

fn ray_count_by_pointer(ray_map: &RayMap) -> BTreeMap<PointerId, usize> {
    let mut counts = BTreeMap::new();
    for (ray_id, _) in ray_map.iter() {
        *counts.entry(ray_id.pointer).or_insert(0) += 1;
    }
    counts
}
