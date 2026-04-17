use std::collections::{BTreeMap, BTreeSet};

use zircon_scene::RenderHybridGiProbe;

use crate::types::EditorOrRuntimeFrame;

const CHILD_SPECIFICITY_BOOST: f32 = 0.3;
const RESIDENT_CHILD_ATTENUATION: f32 = 0.78;

pub(super) fn hybrid_gi_hierarchy_resolve_weight(
    frame: &EditorOrRuntimeFrame,
    source: &RenderHybridGiProbe,
) -> f32 {
    let Some(extract) = frame.extract.lighting.hybrid_global_illumination.as_ref() else {
        return 1.0;
    };

    let resident_probe_ids = frame
        .hybrid_gi_prepare
        .as_ref()
        .map(|prepare| {
            prepare
                .resident_probes
                .iter()
                .map(|probe| probe.probe_id)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if resident_probe_ids.is_empty() {
        return 1.0;
    }

    let probes_by_id = extract
        .probes
        .iter()
        .copied()
        .map(|probe| (probe.probe_id, probe))
        .collect::<BTreeMap<_, _>>();
    let resident_child_count =
        resident_descendant_count(&probes_by_id, &resident_probe_ids, source.probe_id);
    let resident_parent_depth =
        resident_parent_depth(&probes_by_id, &resident_probe_ids, source.probe_id);

    let specificity_weight = 1.0 + resident_parent_depth as f32 * CHILD_SPECIFICITY_BOOST;
    let attenuation_weight = if resident_child_count == 0 {
        1.0
    } else {
        RESIDENT_CHILD_ATTENUATION.powi(resident_child_count as i32)
    };
    (specificity_weight * attenuation_weight).clamp(0.25, 2.0)
}

fn resident_descendant_count(
    probes_by_id: &BTreeMap<u32, RenderHybridGiProbe>,
    resident_probe_ids: &BTreeSet<u32>,
    probe_id: u32,
) -> usize {
    let mut count = 0usize;
    let mut stack = probes_by_id
        .values()
        .filter(|probe| probe.parent_probe_id == Some(probe_id))
        .map(|probe| probe.probe_id)
        .collect::<Vec<_>>();
    let mut visited_probe_ids = BTreeSet::new();

    while let Some(candidate_probe_id) = stack.pop() {
        if !visited_probe_ids.insert(candidate_probe_id) {
            continue;
        }
        if resident_probe_ids.contains(&candidate_probe_id) {
            count += 1;
        }
        stack.extend(
            probes_by_id
                .values()
                .filter(|probe| probe.parent_probe_id == Some(candidate_probe_id))
                .map(|probe| probe.probe_id),
        );
    }

    count
}

fn resident_parent_depth(
    probes_by_id: &BTreeMap<u32, RenderHybridGiProbe>,
    resident_probe_ids: &BTreeSet<u32>,
    probe_id: u32,
) -> usize {
    let mut depth = 0usize;
    let mut current_probe_id = probe_id;

    loop {
        let Some(parent_probe_id) = probes_by_id
            .get(&current_probe_id)
            .and_then(|probe| probe.parent_probe_id)
        else {
            break;
        };
        if resident_probe_ids.contains(&parent_probe_id) {
            depth += 1;
        }
        current_probe_id = parent_probe_id;
    }

    depth
}
