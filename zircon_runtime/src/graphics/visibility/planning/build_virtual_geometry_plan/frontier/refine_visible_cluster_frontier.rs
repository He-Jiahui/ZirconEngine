use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::RenderVirtualGeometryCluster;

use super::super::ordering::virtual_geometry_cluster_sort_key;

pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) fn refine_visible_cluster_frontier(
    visible_clusters: &[RenderVirtualGeometryCluster],
    cluster_budget: usize,
    resident_page_set: Option<&BTreeSet<u32>>,
    previous_visible_cluster_ids: Option<&BTreeSet<u32>>,
    previous_requested_page_ids: Option<&BTreeSet<u32>>,
) -> Vec<RenderVirtualGeometryCluster> {
    if cluster_budget == 0 || visible_clusters.is_empty() {
        return Vec::new();
    }

    let visible_by_id = visible_clusters
        .iter()
        .map(|cluster| (cluster.cluster_id, *cluster))
        .collect::<BTreeMap<_, _>>();
    let mut children_by_parent = BTreeMap::<u32, Vec<RenderVirtualGeometryCluster>>::new();
    let mut frontier = visible_clusters
        .iter()
        .copied()
        .filter(|cluster| {
            cluster
                .parent_cluster_id
                .and_then(|parent| visible_by_id.get(&parent))
                .is_none()
        })
        .collect::<Vec<_>>();

    for cluster in visible_clusters.iter().copied() {
        if let Some(parent_cluster_id) = cluster.parent_cluster_id {
            if visible_by_id.contains_key(&parent_cluster_id) {
                children_by_parent
                    .entry(parent_cluster_id)
                    .or_default()
                    .push(cluster);
            }
        }
    }

    frontier.sort_by(virtual_geometry_cluster_sort_key);
    frontier.truncate(cluster_budget);

    loop {
        frontier.sort_by(virtual_geometry_cluster_sort_key);
        let mut refined = false;

        for index in 0..frontier.len() {
            let cluster = frontier[index];
            let mut children = children_by_parent
                .get(&cluster.cluster_id)
                .cloned()
                .unwrap_or_default();
            if children.is_empty() {
                continue;
            }

            children.sort_by(virtual_geometry_cluster_sort_key);
            let proposed_len = frontier.len() - 1 + children.len();
            if proposed_len > cluster_budget {
                continue;
            }
            if resident_page_set.is_some_and(|resident_pages| {
                !children
                    .iter()
                    .all(|child| resident_pages.contains(&child.page_id))
            }) {
                continue;
            }
            if should_hold_split_hysteresis(
                &cluster,
                &children,
                resident_page_set,
                previous_visible_cluster_ids,
                previous_requested_page_ids,
            ) {
                continue;
            }

            frontier.remove(index);
            frontier.extend(children);
            refined = true;
            break;
        }

        if !refined {
            break;
        }
    }

    frontier.sort_by(virtual_geometry_cluster_sort_key);
    frontier.truncate(cluster_budget);
    frontier
}

fn should_hold_split_hysteresis(
    cluster: &RenderVirtualGeometryCluster,
    children: &[RenderVirtualGeometryCluster],
    resident_page_set: Option<&BTreeSet<u32>>,
    previous_visible_cluster_ids: Option<&BTreeSet<u32>>,
    previous_requested_page_ids: Option<&BTreeSet<u32>>,
) -> bool {
    let Some(resident_pages) = resident_page_set else {
        return false;
    };
    if !resident_pages.contains(&cluster.page_id) {
        return false;
    }

    let Some(previous_visible_cluster_ids) = previous_visible_cluster_ids else {
        return false;
    };
    if !previous_visible_cluster_ids.contains(&cluster.cluster_id) {
        return false;
    }

    let Some(previous_requested_page_ids) = previous_requested_page_ids else {
        return false;
    };
    children
        .iter()
        .all(|child| previous_requested_page_ids.contains(&child.page_id))
}
