use std::cmp::Reverse;

use crate::types::HybridGiPrepareUpdateRequest;

use super::super::HybridGiRuntimeState;

const ANCESTOR_TRACE_SUPPORT_FALLOFF: f32 = 0.78;
const TRACE_SUPPORT_SORT_SCALE: f32 = 1024.0;

pub(super) fn collect_pending_updates(
    runtime: &HybridGiRuntimeState,
) -> Vec<HybridGiPrepareUpdateRequest> {
    let mut pending_updates = runtime
        .pending_updates
        .iter()
        .filter(|update| !has_pending_ancestor_update(runtime, update.probe_id))
        .cloned()
        .collect::<Vec<_>>();
    pending_updates.sort_by_key(|update| {
        (
            Reverse(lineage_trace_support_sort_key(runtime, update.probe_id)),
            Reverse(resident_descendant_count(runtime, update.probe_id)),
            Reverse(descendant_count(runtime, update.probe_id)),
            probe_depth(runtime, update.probe_id),
            update.generation,
            update.probe_id,
        )
    });

    pending_updates
        .into_iter()
        .map(|update| HybridGiPrepareUpdateRequest {
            probe_id: update.probe_id,
            ray_budget: update.ray_budget,
            generation: update.generation,
        })
        .collect::<Vec<_>>()
}

fn has_pending_ancestor_update(runtime: &HybridGiRuntimeState, probe_id: u32) -> bool {
    let mut current_probe_id = probe_id;

    while let Some(parent_probe_id) = runtime.probe_parent_probes.get(&current_probe_id).copied() {
        if runtime.pending_probes.contains(&parent_probe_id) {
            return true;
        }
        current_probe_id = parent_probe_id;
    }

    false
}

fn resident_descendant_count(runtime: &HybridGiRuntimeState, probe_id: u32) -> usize {
    descendant_probe_ids(runtime, probe_id)
        .into_iter()
        .filter(|descendant_probe_id| runtime.resident_slots.contains_key(descendant_probe_id))
        .count()
}

fn descendant_count(runtime: &HybridGiRuntimeState, probe_id: u32) -> usize {
    descendant_probe_ids(runtime, probe_id).len()
}

fn descendant_probe_ids(runtime: &HybridGiRuntimeState, probe_id: u32) -> Vec<u32> {
    let mut stack = runtime
        .probe_parent_probes
        .iter()
        .filter_map(|(&candidate_probe_id, &parent_probe_id)| {
            (parent_probe_id == probe_id).then_some(candidate_probe_id)
        })
        .collect::<Vec<_>>();
    let mut descendants = Vec::new();

    while let Some(candidate_probe_id) = stack.pop() {
        if descendants.contains(&candidate_probe_id) {
            continue;
        }
        descendants.push(candidate_probe_id);
        stack.extend(runtime.probe_parent_probes.iter().filter_map(
            |(&grandchild_probe_id, &parent_probe_id)| {
                (parent_probe_id == candidate_probe_id).then_some(grandchild_probe_id)
            },
        ));
    }

    descendants
}

fn probe_depth(runtime: &HybridGiRuntimeState, probe_id: u32) -> usize {
    let mut depth = 0usize;
    let mut current_probe_id = probe_id;

    while let Some(parent_probe_id) = runtime.probe_parent_probes.get(&current_probe_id).copied() {
        depth += 1;
        current_probe_id = parent_probe_id;
    }

    depth
}

fn lineage_trace_support_sort_key(runtime: &HybridGiRuntimeState, probe_id: u32) -> u32 {
    (lineage_trace_support_score(runtime, probe_id) * TRACE_SUPPORT_SORT_SCALE).round() as u32
}

fn lineage_trace_support_score(runtime: &HybridGiRuntimeState, probe_id: u32) -> f32 {
    let mut total_support = 0.0_f32;
    let mut lineage_weight = 1.0_f32;
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = vec![probe_id];

    loop {
        total_support +=
            single_probe_trace_support_score(runtime, current_probe_id) * lineage_weight;
        let Some(parent_probe_id) = runtime.probe_parent_probes.get(&current_probe_id).copied()
        else {
            break;
        };
        if visited_probe_ids.contains(&parent_probe_id) {
            break;
        }
        visited_probe_ids.push(parent_probe_id);
        lineage_weight *= ANCESTOR_TRACE_SUPPORT_FALLOFF;
        current_probe_id = parent_probe_id;
    }

    total_support
}

fn single_probe_trace_support_score(runtime: &HybridGiRuntimeState, probe_id: u32) -> f32 {
    let Some(probe) = runtime.probe_scene_data.get(&probe_id) else {
        return 0.0;
    };

    runtime
        .scheduled_trace_regions
        .iter()
        .filter_map(|region_id| runtime.trace_region_scene_data.get(region_id))
        .map(|region| {
            let reach = probe.radius_q.saturating_add(region.radius_q).max(1) as f32;
            let max_distance = (reach * 3.0).max(1.0);
            let distance_to_region = abs_diff_u32(probe.position_x_q, region.center_x_q)
                + abs_diff_u32(probe.position_y_q, region.center_y_q)
                + abs_diff_u32(probe.position_z_q, region.center_z_q);
            if distance_to_region >= max_distance {
                return 0.0;
            }

            let proximity = 1.0 - distance_to_region / max_distance;
            proximity * proximity * (region.coverage_q.min(255) as f32 / 128.0)
        })
        .sum()
}

fn abs_diff_u32(left: u32, right: u32) -> f32 {
    left.abs_diff(right) as f32
}
