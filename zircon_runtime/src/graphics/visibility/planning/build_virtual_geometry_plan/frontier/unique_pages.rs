use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};

use crate::core::framework::render::RenderVirtualGeometryCluster;

#[derive(Clone, Copy, Debug)]
struct PagePriority {
    cluster_count: u32,
    total_screen_space_error: f32,
    min_lod_level: u8,
    min_cluster_id: u32,
}

const DENSE_PAGE_PRIORITY_MAX: usize = 1_048_576;

enum PagePriorityIndex {
    Dense(Vec<Option<PagePriority>>),
    Sparse(HashMap<u32, PagePriority>),
}

impl PagePriorityIndex {
    fn into_entries(self) -> Vec<(u32, PagePriority)> {
        match self {
            Self::Dense(priorities) => priorities
                .into_iter()
                .enumerate()
                .filter_map(|(page_id, priority)| {
                    priority.map(|priority| (page_id as u32, priority))
                })
                .collect(),
            Self::Sparse(priorities) => priorities.into_iter().collect(),
        }
    }
}

pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) fn unique_pages(
    visible_clusters: &[RenderVirtualGeometryCluster],
    resident_page_set: &BTreeSet<u32>,
    budget: usize,
) -> Vec<u32> {
    if budget == 0 {
        return Vec::new();
    }

    let page_priorities = aggregate_page_priorities(visible_clusters, resident_page_set);

    let mut ranked_pages = page_priorities.into_entries();
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

fn aggregate_page_priorities(
    visible_clusters: &[RenderVirtualGeometryCluster],
    resident_page_set: &BTreeSet<u32>,
) -> PagePriorityIndex {
    let max_page_id = visible_clusters
        .iter()
        .map(|cluster| cluster.page_id as usize)
        .max();
    if let Some(max_page_id) =
        max_page_id.filter(|max_page_id| *max_page_id <= DENSE_PAGE_PRIORITY_MAX)
    {
        let mut page_priorities = Vec::with_capacity(max_page_id.saturating_add(1));
        page_priorities.resize_with(max_page_id.saturating_add(1), || None);
        for cluster in visible_clusters {
            if resident_page_set.contains(&cluster.page_id) {
                continue;
            }

            let priority = page_priorities[cluster.page_id as usize].get_or_insert(PagePriority {
                cluster_count: 0,
                total_screen_space_error: 0.0,
                min_lod_level: cluster.lod_level,
                min_cluster_id: cluster.cluster_id,
            });
            update_priority(priority, cluster);
        }
        return PagePriorityIndex::Dense(page_priorities);
    }

    let mut page_priorities =
        HashMap::<u32, PagePriority>::with_capacity(visible_clusters.len().div_ceil(2));
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
        update_priority(priority, cluster);
    }
    PagePriorityIndex::Sparse(page_priorities)
}

fn update_priority(priority: &mut PagePriority, cluster: &RenderVirtualGeometryCluster) {
    priority.cluster_count = priority.cluster_count.saturating_add(1);
    priority.total_screen_space_error += cluster.screen_space_error.max(0.0);
    priority.min_lod_level = priority.min_lod_level.min(cluster.lod_level);
    priority.min_cluster_id = priority.min_cluster_id.min(cluster.cluster_id);
}

#[cfg(test)]
#[path = "tests/unique_pages_optimization_tests.rs"]
mod optimization_tests;
