use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::render::RenderVirtualGeometryHierarchyNode;

use super::{
    build_node_and_cluster_cull_child_visit_records, build_node_and_cluster_cull_child_work_items,
};
use crate::virtual_geometry::types::{
    VirtualGeometryNodeAndClusterCullChildWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalChildSource,
    VirtualGeometryNodeAndClusterCullTraversalOp, VirtualGeometryNodeAndClusterCullTraversalRecord,
};

const BENCH_ITEM_COUNT: usize = 1_024;
const CHECKS_PER_SAMPLE: usize = 8;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn child_worklists_preserve_authored_order_and_first_duplicate_node() {
    let traversal_records = [
        traversal_record(
            VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
            0,
            3,
        ),
        traversal_record(
            VirtualGeometryNodeAndClusterCullTraversalChildSource::FixedFanout,
            0,
            2,
        ),
    ];
    let child_work_items =
        build_node_and_cluster_cull_child_work_items(&traversal_records, &[7, 8]);
    assert_eq!(child_work_items.len(), 2);
    assert!(child_work_items.capacity() >= child_work_items.len());

    let hierarchy_nodes = [
        hierarchy_node(7, 70, 2),
        hierarchy_node(7, 700, 20),
        hierarchy_node(8, 80, 3),
    ];
    let optimized =
        build_node_and_cluster_cull_child_visit_records(&child_work_items, &hierarchy_nodes, 11);
    let legacy = legacy_build_child_visit_records(&child_work_items, &hierarchy_nodes, 11);

    assert_eq!(optimized, legacy);
    assert_eq!(optimized[0].node_cluster_start, 70);
    assert_eq!(optimized[0].node_cluster_count, 2);
    assert_eq!(optimized[1].node_cluster_start, 80);
    assert_eq!(optimized[1].traversal_index, 12);
}

#[test]
#[ignore = "release-only child hierarchy lookup benchmark"]
fn child_visit_hierarchy_lookup_release_benchmark_evidence() {
    let hierarchy_nodes = (0..BENCH_ITEM_COUNT)
        .map(|node_id| hierarchy_node(node_id as u32, node_id as u32 * 4, 2))
        .collect::<Vec<_>>();
    let child_work_items = (0..BENCH_ITEM_COUNT)
        .map(|item_index| child_work_item((BENCH_ITEM_COUNT - item_index - 1) as u32))
        .collect::<Vec<_>>();
    assert_eq!(
        legacy_build_child_visit_records(&child_work_items, &hierarchy_nodes, 0),
        build_node_and_cluster_cull_child_visit_records(&child_work_items, &hierarchy_nodes, 0)
    );

    for _ in 0..4 {
        black_box(measure_legacy(&child_work_items, &hierarchy_nodes));
        black_box(measure_optimized(&child_work_items, &hierarchy_nodes));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&child_work_items, &hierarchy_nodes));
            optimized_samples.push(measure_optimized(&child_work_items, &hierarchy_nodes));
        } else {
            optimized_samples.push(measure_optimized(&child_work_items, &hierarchy_nodes));
            legacy_samples.push(measure_legacy(&child_work_items, &hierarchy_nodes));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=child_visit_hierarchy_lookup \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
item_count={BENCH_ITEM_COUNT} hierarchy_node_count={BENCH_ITEM_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_lookup=linear_per_child optimized_lookup=frame_hash_index \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "indexed child hierarchy lookup must reduce P95 by at least 75%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(
    child_work_items: &[VirtualGeometryNodeAndClusterCullChildWorkItem],
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_build_child_visit_records(
            black_box(child_work_items),
            black_box(hierarchy_nodes),
            0,
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(
    child_work_items: &[VirtualGeometryNodeAndClusterCullChildWorkItem],
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(build_node_and_cluster_cull_child_visit_records(
            black_box(child_work_items),
            black_box(hierarchy_nodes),
            0,
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_build_child_visit_records(
    child_work_items: &[VirtualGeometryNodeAndClusterCullChildWorkItem],
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
    first_traversal_index: u32,
) -> Vec<VirtualGeometryNodeAndClusterCullTraversalRecord> {
    child_work_items
        .iter()
        .enumerate()
        .map(|(child_index, work_item)| {
            let node = hierarchy_nodes.iter().copied().find(|node| {
                node.instance_index == work_item.instance_index
                    && node.node_id == work_item.child_node_id
            });
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: work_item.instance_index,
                entity: work_item.entity,
                cluster_array_index: work_item.parent_cluster_array_index,
                hierarchy_node_id: Some(work_item.child_node_id),
                node_cluster_start: node.map(|node| node.cluster_start).unwrap_or(0),
                node_cluster_count: node.map(|node| node.cluster_count).unwrap_or(0),
                child_base: 0,
                child_count: 0,
                traversal_index: first_traversal_index
                    .saturating_add(u32::try_from(child_index).unwrap_or(u32::MAX)),
                cluster_budget: work_item.cluster_budget,
                page_budget: work_item.page_budget,
                forced_mip: work_item.forced_mip,
            }
        })
        .collect()
}

fn traversal_record(
    child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource,
    child_base: u32,
    child_count: u32,
) -> VirtualGeometryNodeAndClusterCullTraversalRecord {
    VirtualGeometryNodeAndClusterCullTraversalRecord {
        op: VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
        child_source,
        instance_index: 0,
        entity: 42,
        cluster_array_index: 3,
        hierarchy_node_id: Some(6),
        node_cluster_start: 0,
        node_cluster_count: 0,
        child_base,
        child_count,
        traversal_index: 5,
        cluster_budget: 9,
        page_budget: 7,
        forced_mip: Some(2),
    }
}

fn child_work_item(child_node_id: u32) -> VirtualGeometryNodeAndClusterCullChildWorkItem {
    VirtualGeometryNodeAndClusterCullChildWorkItem {
        instance_index: 0,
        entity: 42,
        parent_cluster_array_index: 3,
        parent_hierarchy_node_id: Some(6),
        child_node_id,
        child_table_index: child_node_id,
        traversal_index: child_node_id,
        cluster_budget: 9,
        page_budget: 7,
        forced_mip: Some(2),
    }
}

fn hierarchy_node(
    node_id: u32,
    cluster_start: u32,
    cluster_count: u32,
) -> RenderVirtualGeometryHierarchyNode {
    RenderVirtualGeometryHierarchyNode {
        instance_index: 0,
        node_id,
        child_base: 0,
        child_count: 0,
        cluster_start,
        cluster_count,
    }
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
