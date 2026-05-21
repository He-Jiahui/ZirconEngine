use std::collections::{BTreeMap, BTreeSet};

use super::{
    hovered_hits_for_pointer, sorted_hits_for_pointer, HitTarget, PointerHits, PointerId, RayMap,
};

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
        let ray_count_by_pointer = ray_count_by_pointer(ray_map);
        let pointers = report_pointer_ids(ray_map, outputs)
            .into_iter()
            .map(|pointer| {
                PickingPointerPipelineReport::from_pointer(
                    pointer,
                    *ray_count_by_pointer.get(&pointer).unwrap_or(&0),
                    outputs,
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
    fn from_pointer(pointer: PointerId, ray_count: usize, outputs: &[PointerHits]) -> Self {
        let sorted_hits = sorted_hits_for_pointer(outputs, pointer);
        let hovered_hits = hovered_hits_for_pointer(outputs, pointer);
        let backend_output_count = outputs
            .iter()
            .filter(|output| output.pointer == pointer)
            .count();
        let raw_hit_count = outputs
            .iter()
            .filter(|output| output.pointer == pointer)
            .map(|output| output.hits.len())
            .sum();

        Self {
            pointer,
            ray_count,
            backend_output_count,
            raw_hit_count,
            sorted_hit_count: sorted_hits.len(),
            hovered_hit_count: hovered_hits.len(),
            non_hoverable_hit_count: sorted_hits
                .iter()
                .filter(|hit| !hit.pickable.is_hoverable)
                .count(),
            top_target: sorted_hits.first().map(|hit| hit.target),
            blocking_target: sorted_hits
                .iter()
                .find(|hit| hit.pickable.should_block_lower)
                .map(|hit| hit.target),
        }
    }
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
