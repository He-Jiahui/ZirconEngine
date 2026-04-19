use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use zircon_framework::render::RenderVirtualGeometryCluster;

#[derive(Clone, Copy, Debug)]
struct PagePriority {
    cluster_count: u32,
    total_screen_space_error: f32,
    min_lod_level: u8,
    min_cluster_id: u32,
}

pub(in crate::visibility::planning::build_virtual_geometry_plan) fn unique_pages(
    visible_clusters: &[RenderVirtualGeometryCluster],
    resident_page_set: &BTreeSet<u32>,
    budget: usize,
) -> Vec<u32> {
    if budget == 0 {
        return Vec::new();
    }

    let mut page_priorities = BTreeMap::<u32, PagePriority>::new();
    for cluster in visible_clusters {
        if resident_page_set.contains(&cluster.page_id) {
            continue;
        }

        let priority = page_priorities
            .entry(cluster.page_id)
            .or_insert(PagePriority {
                cluster_count: 0,
                total_screen_space_error: 0.0,
                min_lod_level: cluster.lod_level,
                min_cluster_id: cluster.cluster_id,
            });
        priority.cluster_count = priority.cluster_count.saturating_add(1);
        priority.total_screen_space_error += cluster.screen_space_error.max(0.0);
        priority.min_lod_level = priority.min_lod_level.min(cluster.lod_level);
        priority.min_cluster_id = priority.min_cluster_id.min(cluster.cluster_id);
    }

    let mut ranked_pages = page_priorities.into_iter().collect::<Vec<_>>();
    ranked_pages.sort_by(|(left_page_id, left), (right_page_id, right)| {
        right
            .cluster_count
            .cmp(&left.cluster_count)
            .then_with(|| {
                right
                    .total_screen_space_error
                    .partial_cmp(&left.total_screen_space_error)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| left.min_lod_level.cmp(&right.min_lod_level))
            .then_with(|| left.min_cluster_id.cmp(&right.min_cluster_id))
            .then_with(|| left_page_id.cmp(right_page_id))
    });

    ranked_pages
        .into_iter()
        .take(budget)
        .map(|(page_id, _)| page_id)
        .collect()
}
