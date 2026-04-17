use std::collections::{BTreeMap, BTreeSet};

use zircon_scene::{RenderHybridGiExtract, ViewportCameraSnapshot};

use super::super::super::declarations::{
    VisibilityHistorySnapshot, VisibilityHybridGiFeedback, VisibilityHybridGiProbe,
    VisibilityHybridGiUpdatePlan,
};
use super::hybrid_gi_probe_request_sort_key::hybrid_gi_probe_request_sort_key;
use super::hybrid_gi_probe_sort_key::hybrid_gi_probe_sort_key;
use super::hybrid_gi_probe_visible::hybrid_gi_probe_visible;
use super::hybrid_gi_trace_region_sort_key::hybrid_gi_trace_region_sort_key;
use super::hybrid_gi_trace_region_visible::hybrid_gi_trace_region_visible;
use super::refine_visible_probe_frontier::refine_visible_probe_frontier;
use super::unique_probe_ids::unique_probe_ids;

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

    let mut requested_probes = active_probes
        .iter()
        .flat_map(|probe| {
            if probe.resident {
                children_by_parent
                    .get(&probe.probe_id)
                    .into_iter()
                    .flat_map(|children| children.iter())
                    .filter(|child| !child.resident)
                    .copied()
                    .collect::<Vec<_>>()
            } else {
                vec![*probe]
            }
        })
        .collect::<Vec<_>>();
    requested_probes.sort_by(|left, right| {
        hybrid_gi_probe_request_sort_key(left, right, &scheduled_trace_regions)
    });
    let requested_probe_ids = unique_probe_ids(
        requested_probes.iter().map(|probe| probe.probe_id),
        extract.probe_budget as usize,
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
    let evictable_probe_ids = resident_probe_ids
        .iter()
        .copied()
        .filter(|probe_id| !active_probe_set.contains(probe_id))
        .filter(|probe_id| !previous_requested_probe_ids.contains(probe_id))
        .filter(|probe_id| !merge_back_child_hold_protected_probe_ids.contains(probe_id))
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
