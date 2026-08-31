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

pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) fn unique_pages(
    visible_clusters: &[RenderVirtualGeometryCluster],
    resident_page_set: &BTreeSet<u32>,
    budget: usize,
) -> Vec<u32> {
    if budget == 0 {
        return Vec::new();
    }

    let page_priorities = aggregate_page_priorities(visible_clusters, resident_page_set);

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

fn aggregate_page_priorities(
    visible_clusters: &[RenderVirtualGeometryCluster],
    resident_page_set: &BTreeSet<u32>,
) -> HashMap<u32, PagePriority> {
    let mut page_priorities = HashMap::<u32, PagePriority>::new();
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
    page_priorities
}

fn update_priority(priority: &mut PagePriority, cluster: &RenderVirtualGeometryCluster) {
    priority.cluster_count = priority.cluster_count.saturating_add(1);
    priority.total_screen_space_error += cluster.screen_space_error.max(0.0);
    priority.min_lod_level = priority.min_lod_level.min(cluster.lod_level);
    priority.min_cluster_id = priority.min_cluster_id.min(cluster.cluster_id);
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::math::Vec3;

    use super::*;

    #[test]
    fn optimization_batch_20260826m_runtime09b_hash_aggregation_preserves_page_ranking() {
        let clusters = vec![
            cluster(1, 10, 0, 3.0),
            cluster(2, 20, 0, 8.0),
            cluster(3, 10, 0, 2.0),
            cluster(4, 20, 0, 9.0),
            cluster(5, 30, 0, 100.0),
            cluster(6, 12, 1, 1.0),
            cluster(6, 11, 1, 1.0),
        ];
        let resident_pages = BTreeSet::from([30]);

        assert_eq!(
            unique_pages(&clusters, &resident_pages, 4),
            vec![20, 10, 11, 12]
        );
        assert!(unique_pages(&clusters, &resident_pages, 0).is_empty());
    }

    #[test]
    fn optimization_batch_20260826m_runtime09b_page_priorities_use_hash_aggregation() {
        let source = include_str!("unique_pages.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("virtual geometry page ranking production source");

        assert!(!production.contains("BTreeMap::<u32, PagePriority>"));
        assert!(production.contains("HashMap::<u32, PagePriority>::new()"));
        assert!(production.contains("fn aggregate_page_priorities"));
        assert!(production.contains("left_page_id.cmp(right_page_id)"));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260826m_runtime09b_page_priority_hash_performance_evidence() {
        fn legacy_aggregate(
            visible_clusters: &[RenderVirtualGeometryCluster],
        ) -> BTreeMap<u32, PagePriority> {
            let mut priorities = BTreeMap::<u32, PagePriority>::new();
            for cluster in visible_clusters {
                let priority = priorities.entry(cluster.page_id).or_insert(PagePriority {
                    cluster_count: 0,
                    total_screen_space_error: 0.0,
                    min_lod_level: cluster.lod_level,
                    min_cluster_id: cluster.cluster_id,
                });
                update_priority(priority, cluster);
            }
            priorities
        }

        let page_count = 32_768_u32;
        let clusters = (0..262_144_u32)
            .map(|index| {
                cluster(
                    index,
                    index % page_count,
                    (index % 8) as u8,
                    (index % 17) as f32,
                )
            })
            .collect::<Vec<_>>();
        let resident_pages = BTreeSet::new();
        let mut legacy_samples = Vec::with_capacity(17);
        let mut hash_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let started = Instant::now();
            black_box(legacy_aggregate(black_box(&clusters)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(aggregate_page_priorities(
                black_box(&clusters),
                black_box(&resident_pages),
            ));
            hash_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        hash_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let hash_p95 = hash_samples[16];
        println!(
            "RUNTIME09B_VIRTUAL_GEOMETRY_PAGE_PRIORITY_HASH_BENCH_V1 clusters={} unique_pages={} legacy_p95_ns={} hash_p95_ns={} legacy_tree_admissions={} hash_admissions={} target_ratio_bp=6000",
            clusters.len(),
            page_count,
            legacy_p95,
            hash_p95,
            clusters.len(),
            clusters.len(),
        );
        assert!(
            hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
            "virtual geometry page hash P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
        );
    }

    fn cluster(
        cluster_id: u32,
        page_id: u32,
        lod_level: u8,
        screen_space_error: f32,
    ) -> RenderVirtualGeometryCluster {
        RenderVirtualGeometryCluster {
            entity: u64::from(cluster_id),
            cluster_id,
            hierarchy_node_id: None,
            page_id,
            lod_level,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            screen_space_error,
        }
    }
}
