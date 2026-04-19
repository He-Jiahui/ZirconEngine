use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::{
    RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion, ViewportCameraSnapshot,
};

use super::super::super::declarations::{
    VisibilityHistorySnapshot, VisibilityHybridGiFeedback, VisibilityHybridGiProbe,
    VisibilityHybridGiUpdatePlan,
};
use super::frontier::{refine_visible_probe_frontier, unique_probe_ids};
use super::ordering::{
    hybrid_gi_probe_request_sort_key, hybrid_gi_probe_sort_key, hybrid_gi_trace_region_sort_key,
};
use super::visibility::{hybrid_gi_probe_visible, hybrid_gi_trace_region_visible};

pub(crate) fn build_hybrid_gi_plan(
    extract: Option<&RenderHybridGiExtract>,
    visible_entities: &BTreeSet<u64>,
    camera: &ViewportCameraSnapshot,
    previous: Option<&VisibilityHistorySnapshot>,
) -> (
    Vec<VisibilityHybridGiProbe>,
    VisibilityHybridGiUpdatePlan,
    VisibilityHybridGiFeedback,
    Vec<u32>,
) {
    let Some(extract) = extract else {
        return (
            Vec::new(),
            VisibilityHybridGiUpdatePlan::default(),
            VisibilityHybridGiFeedback::default(),
            Vec::new(),
        );
    };

    let resident_probe_ids = extract
        .probes
        .iter()
        .filter(|probe| probe.resident)
        .map(|probe| probe.probe_id)
        .collect::<Vec<_>>();

    let mut visible_probes = extract
        .probes
        .iter()
        .filter(|probe| visible_entities.contains(&probe.entity))
        .filter(|probe| hybrid_gi_probe_visible(probe, camera))
        .copied()
        .collect::<Vec<_>>();
    visible_probes.sort_by(hybrid_gi_probe_sort_key);
    let active_probes = refine_visible_probe_frontier(&visible_probes);
    let visible_probes_by_id = visible_probes
        .iter()
        .copied()
        .map(|probe| (probe.probe_id, probe))
        .collect::<BTreeMap<_, _>>();

    let hybrid_gi_active_probes = active_probes
        .iter()
        .map(|probe| VisibilityHybridGiProbe {
            entity: probe.entity,
            probe_id: probe.probe_id,
            resident: probe.resident,
            ray_budget: probe.ray_budget,
        })
        .collect::<Vec<_>>();

    let mut scheduled_trace_regions = extract
        .trace_regions
        .iter()
        .filter(|region| visible_entities.contains(&region.entity))
        .filter(|region| hybrid_gi_trace_region_visible(region, camera))
        .copied()
        .collect::<Vec<_>>();
    scheduled_trace_regions.sort_by(hybrid_gi_trace_region_sort_key);
    scheduled_trace_regions.truncate(extract.tracing_budget as usize);
    let scheduled_trace_region_ids = scheduled_trace_regions
        .iter()
        .map(|region| region.region_id)
        .collect::<Vec<_>>();

    let children_by_parent = visible_probes
        .iter()
        .filter_map(|probe| {
            probe
                .parent_probe_id
                .map(|parent_probe_id| (parent_probe_id, *probe))
        })
        .fold(
            BTreeMap::<u32, Vec<_>>::new(),
            |mut map, (parent_probe_id, probe)| {
                map.entry(parent_probe_id).or_default().push(probe);
                map
            },
        );
    let previous_requested_probe_ids = previous
        .map(|history| {
            history
                .hybrid_gi_requested_probes
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let requested_probe_groups = active_probes
        .iter()
        .map(|probe| {
            let mut group = if probe.resident {
                collect_nonresident_descendants(&children_by_parent, probe.probe_id)
            } else {
                vec![*probe]
            };
            group.sort_by(|left, right| {
                hybrid_gi_probe_request_sort_key(
                    left,
                    right,
                    &scheduled_trace_regions,
                    &visible_probes_by_id,
                    &previous_requested_probe_ids,
                )
            });
            group
        })
        .filter(|group| !group.is_empty())
        .collect::<Vec<_>>();
    let requested_probes = interleave_requested_probe_groups(
        &requested_probe_groups,
        &scheduled_trace_regions,
        &visible_probes_by_id,
        &previous_requested_probe_ids,
    );
    let requested_probe_ids = unique_probe_ids(
        requested_probes.iter().map(|probe| probe.probe_id),
        extract.probe_budget as usize,
    );
    let previous_active_probe_ids = previous
        .map(|history| {
            history
                .hybrid_gi_active_probe_ids
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let dirty_requested_probe_ids = requested_probe_ids
        .iter()
        .copied()
        .filter(|probe_id| !previous_requested_probe_ids.contains(probe_id))
        .collect::<Vec<_>>();

    let active_probe_set = hybrid_gi_active_probes
        .iter()
        .map(|probe| probe.probe_id)
        .collect::<BTreeSet<_>>();
    let merge_back_child_hold_protected_probe_ids = visible_probes
        .iter()
        .filter(|probe| probe.resident)
        .filter(|probe| previous_active_probe_ids.contains(&probe.probe_id))
        .filter(|probe| !active_probe_set.contains(&probe.probe_id))
        .filter(|probe| {
            probe
                .parent_probe_id
                .is_some_and(|parent_probe_id| active_probe_set.contains(&parent_probe_id))
        })
        .map(|probe| probe.probe_id)
        .collect::<BTreeSet<_>>();
    let requested_frontier_probe_ids = requested_frontier_probe_ids(
        &requested_probe_ids,
        &visible_probes_by_id,
        &active_probe_set,
    );
    let requested_frontier_hold_protected_probe_ids = visible_probes
        .iter()
        .filter(|probe| probe.resident)
        .filter(|probe| !active_probe_set.contains(&probe.probe_id))
        .filter(|probe| {
            visible_frontier_probe_id_for_probe(
                probe.probe_id,
                &visible_probes_by_id,
                &active_probe_set,
            )
            .is_some_and(|frontier_probe_id| {
                requested_frontier_probe_ids.contains(&frontier_probe_id)
            })
        })
        .filter(|probe| {
            !has_hidden_resident_descendant_probe(
                probe.probe_id,
                &children_by_parent,
                &active_probe_set,
            )
        })
        .map(|probe| probe.probe_id)
        .collect::<BTreeSet<_>>();
    let evictable_probe_ids = resident_probe_ids
        .iter()
        .copied()
        .filter(|probe_id| !active_probe_set.contains(probe_id))
        .filter(|probe_id| !previous_requested_probe_ids.contains(probe_id))
        .filter(|probe_id| !merge_back_child_hold_protected_probe_ids.contains(probe_id))
        .filter(|probe_id| !requested_frontier_hold_protected_probe_ids.contains(probe_id))
        .collect::<Vec<_>>();

    let update_plan = VisibilityHybridGiUpdatePlan {
        resident_probe_ids,
        requested_probe_ids: requested_probe_ids.clone(),
        dirty_requested_probe_ids: dirty_requested_probe_ids.clone(),
        scheduled_trace_region_ids: scheduled_trace_region_ids.clone(),
        evictable_probe_ids: evictable_probe_ids.clone(),
    };
    let feedback = VisibilityHybridGiFeedback {
        active_probe_ids: hybrid_gi_active_probes
            .iter()
            .map(|probe| probe.probe_id)
            .collect(),
        requested_probe_ids: requested_probe_ids.clone(),
        scheduled_trace_region_ids: scheduled_trace_region_ids.clone(),
        evictable_probe_ids: evictable_probe_ids.clone(),
    };

    (
        hybrid_gi_active_probes,
        update_plan,
        feedback,
        requested_probe_ids,
    )
}

fn collect_nonresident_descendants(
    children_by_parent: &BTreeMap<u32, Vec<RenderHybridGiProbe>>,
    root_probe_id: u32,
) -> Vec<RenderHybridGiProbe> {
    let mut descendants = Vec::new();
    let mut visited_probe_ids = BTreeSet::new();
    let mut stack = children_by_parent
        .get(&root_probe_id)
        .cloned()
        .unwrap_or_default();

    while let Some(probe) = stack.pop() {
        if !visited_probe_ids.insert(probe.probe_id) {
            continue;
        }
        if !probe.resident {
            descendants.push(probe);
        }
        if let Some(children) = children_by_parent.get(&probe.probe_id) {
            stack.extend(children.iter().copied());
        }
    }

    descendants
}

fn interleave_requested_probe_groups(
    requested_probe_groups: &[Vec<RenderHybridGiProbe>],
    scheduled_trace_regions: &[RenderHybridGiTraceRegion],
    visible_probes_by_id: &BTreeMap<u32, RenderHybridGiProbe>,
    previous_requested_probe_ids: &BTreeSet<u32>,
) -> Vec<RenderHybridGiProbe> {
    let mut requested_probes = Vec::new();
    let mut round_index = 0usize;

    loop {
        let mut round = requested_probe_groups
            .iter()
            .filter_map(|group| group.get(round_index).copied())
            .collect::<Vec<_>>();
        if round.is_empty() {
            break;
        }
        round.sort_by(|left, right| {
            hybrid_gi_probe_request_sort_key(
                left,
                right,
                scheduled_trace_regions,
                visible_probes_by_id,
                previous_requested_probe_ids,
            )
        });
        requested_probes.extend(round);
        round_index += 1;
    }

    requested_probes
}

fn requested_frontier_probe_ids(
    requested_probe_ids: &[u32],
    visible_probes_by_id: &BTreeMap<u32, RenderHybridGiProbe>,
    active_probe_set: &BTreeSet<u32>,
) -> BTreeSet<u32> {
    requested_probe_ids
        .iter()
        .filter_map(|probe_id| {
            visible_frontier_probe_id_for_probe(*probe_id, visible_probes_by_id, active_probe_set)
        })
        .collect()
}

fn visible_frontier_probe_id_for_probe(
    probe_id: u32,
    visible_probes_by_id: &BTreeMap<u32, RenderHybridGiProbe>,
    active_probe_set: &BTreeSet<u32>,
) -> Option<u32> {
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

    loop {
        let Some(parent_probe_id) = visible_probes_by_id
            .get(&current_probe_id)
            .and_then(|probe| probe.parent_probe_id)
        else {
            break;
        };
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        if active_probe_set.contains(&parent_probe_id) {
            return Some(parent_probe_id);
        }
        current_probe_id = parent_probe_id;
    }

    None
}

fn has_hidden_resident_descendant_probe(
    probe_id: u32,
    children_by_parent: &BTreeMap<u32, Vec<RenderHybridGiProbe>>,
    active_probe_set: &BTreeSet<u32>,
) -> bool {
    let mut visited_probe_ids = BTreeSet::new();
    let mut stack = children_by_parent
        .get(&probe_id)
        .cloned()
        .unwrap_or_default();

    while let Some(candidate_probe) = stack.pop() {
        if !visited_probe_ids.insert(candidate_probe.probe_id) {
            continue;
        }
        if candidate_probe.resident && !active_probe_set.contains(&candidate_probe.probe_id) {
            return true;
        }
        if let Some(children) = children_by_parent.get(&candidate_probe.probe_id) {
            stack.extend(children.iter().copied());
        }
    }

    false
}
