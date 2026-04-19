use std::collections::BTreeMap;

use zircon_framework::render::RenderHybridGiProbe;

use super::super::ordering::hybrid_gi_probe_sort_key;

pub(in crate::visibility::planning::build_hybrid_gi_plan) fn refine_visible_probe_frontier(
    visible_probes: &[RenderHybridGiProbe],
) -> Vec<RenderHybridGiProbe> {
    if visible_probes.is_empty() {
        return Vec::new();
    }

    let visible_by_id = visible_probes
        .iter()
        .map(|probe| (probe.probe_id, *probe))
        .collect::<BTreeMap<_, _>>();
    let mut children_by_parent = BTreeMap::<u32, Vec<RenderHybridGiProbe>>::new();
    let mut frontier = visible_probes
        .iter()
        .copied()
        .filter(|probe| {
            probe
                .parent_probe_id
                .and_then(|parent_probe_id| visible_by_id.get(&parent_probe_id))
                .is_none()
        })
        .collect::<Vec<_>>();

    for probe in visible_probes.iter().copied() {
        if let Some(parent_probe_id) = probe.parent_probe_id {
            if visible_by_id.contains_key(&parent_probe_id) {
                children_by_parent
                    .entry(parent_probe_id)
                    .or_default()
                    .push(probe);
            }
        }
    }

    frontier.sort_by(hybrid_gi_probe_sort_key);

    loop {
        frontier.sort_by(hybrid_gi_probe_sort_key);
        let mut refined = false;

        for index in 0..frontier.len() {
            let probe = frontier[index];
            let mut children = children_by_parent
                .get(&probe.probe_id)
                .cloned()
                .unwrap_or_default();
            if !probe.resident
                || children.is_empty()
                || children.iter().any(|child| !child.resident)
            {
                continue;
            }
            children.sort_by(hybrid_gi_probe_sort_key);

            frontier.remove(index);
            frontier.extend(children);
            refined = true;
            break;
        }

        if !refined {
            break;
        }
    }

    frontier.sort_by(hybrid_gi_probe_sort_key);
    frontier
}
