use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::render::RenderVirtualGeometryHierarchyNode;

use super::build_node_and_cluster_cull_traversal_records;
use crate::virtual_geometry::types::{
    VirtualGeometryNodeAndClusterCullClusterWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalChildSource,
    VirtualGeometryNodeAndClusterCullTraversalOp, VirtualGeometryNodeAndClusterCullTraversalRecord,
};

const BENCH_ITEM_COUNT: usize = 1_024;
const CHECKS_PER_SAMPLE: usize = 8;
const LEGACY_CHILD_FANOUT: u32 = 4;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn traversal_records_preserve_pair_order_and_first_duplicate_hierarchy_node() {
    let work_items = [cluster_work_item(7, 0), cluster_work_item(8, 2)];
    let hierarchy_nodes = [
        hierarchy_node(7, 10, 2),
        hierarchy_node(7, 90, 9),
        hierarchy_node(8, 20, 3),
    ];

    let optimized = build_node_and_cluster_cull_traversal_records(&work_items, &hierarchy_nodes);
    let legacy = legacy_build_traversal_records(&work_items, &hierarchy_nodes);

    assert_eq!(optimized, legacy);
    assert_eq!(optimized.len(), work_items.len() * 2);
    assert!(optimized.capacity() >= optimized.len());
    assert_eq!(
        optimized[0].op,
        VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode
    );
    assert_eq!(
        optimized[1].op,
        VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild
    );
    assert_eq!(optimized[1].child_base, 10);
    assert_eq!(optimized[1].child_count, 2);
    assert_eq!(
        optimized[3].op,
        VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster
    );
}

#[test]
#[ignore = "release-only traversal hierarchy lookup benchmark"]
fn traversal_hierarchy_lookup_release_benchmark_evidence() {
    let work_items = (0..BENCH_ITEM_COUNT)
        .map(|item_index| cluster_work_item((BENCH_ITEM_COUNT - item_index - 1) as u32, 0))
        .collect::<Vec<_>>();
    let hierarchy_nodes = (0..BENCH_ITEM_COUNT)
        .map(|node_id| hierarchy_node(node_id as u32, node_id as u32 * 4, 2))
        .collect::<Vec<_>>();
    assert_eq!(
        legacy_build_traversal_records(&work_items, &hierarchy_nodes),
        build_node_and_cluster_cull_traversal_records(&work_items, &hierarchy_nodes)
    );

    for _ in 0..4 {
        black_box(measure_legacy(&work_items, &hierarchy_nodes));
        black_box(measure_optimized(&work_items, &hierarchy_nodes));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&work_items, &hierarchy_nodes));
            optimized_samples.push(measure_optimized(&work_items, &hierarchy_nodes));
        } else {
            optimized_samples.push(measure_optimized(&work_items, &hierarchy_nodes));
            legacy_samples.push(measure_legacy(&work_items, &hierarchy_nodes));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=traversal_hierarchy_lookup \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
item_count={BENCH_ITEM_COUNT} hierarchy_node_count={BENCH_ITEM_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_lookup=linear_per_enqueue optimized_lookup=frame_hash_index \
legacy_preallocated_records=0 optimized_preallocated_records={} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        BENCH_ITEM_COUNT * 2,
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "indexed traversal hierarchy lookup must reduce P95 by at least 75%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(
    work_items: &[VirtualGeometryNodeAndClusterCullClusterWorkItem],
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_build_traversal_records(
            black_box(work_items),
            black_box(hierarchy_nodes),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(
    work_items: &[VirtualGeometryNodeAndClusterCullClusterWorkItem],
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(build_node_and_cluster_cull_traversal_records(
            black_box(work_items),
            black_box(hierarchy_nodes),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_build_traversal_records(
    work_items: &[VirtualGeometryNodeAndClusterCullClusterWorkItem],
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> Vec<VirtualGeometryNodeAndClusterCullTraversalRecord> {
    let mut records = Vec::new();
    let mut traversal_index = 0_u32;
    let mut stored_cluster_count = 0_u32;
    for work_item in work_items {
        records.push(legacy_record(
            *work_item,
            VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
            traversal_index,
            hierarchy_nodes,
        ));
        traversal_index = traversal_index.saturating_add(1);
        let op = if stored_cluster_count < work_item.cluster_budget {
            stored_cluster_count = stored_cluster_count.saturating_add(1);
            VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster
        } else {
            VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild
        };
        records.push(legacy_record(
            *work_item,
            op,
            traversal_index,
            hierarchy_nodes,
        ));
        traversal_index = traversal_index.saturating_add(1);
    }
    records
}

fn legacy_record(
    work_item: VirtualGeometryNodeAndClusterCullClusterWorkItem,
    op: VirtualGeometryNodeAndClusterCullTraversalOp,
    traversal_index: u32,
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> VirtualGeometryNodeAndClusterCullTraversalRecord {
    let (child_source, child_base, child_count) = match op {
        VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild => {
            hierarchy_child_range(work_item, hierarchy_nodes).unwrap_or((
                VirtualGeometryNodeAndClusterCullTraversalChildSource::FixedFanout,
                work_item
                    .cluster_array_index
                    .saturating_mul(LEGACY_CHILD_FANOUT),
                LEGACY_CHILD_FANOUT,
            ))
        }
        VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode
        | VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster => (
            VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
            0,
            0,
        ),
    };
    VirtualGeometryNodeAndClusterCullTraversalRecord {
        op,
        child_source,
        instance_index: work_item.instance_index,
        entity: work_item.entity,
        cluster_array_index: work_item.cluster_array_index,
        hierarchy_node_id: work_item.hierarchy_node_id,
        node_cluster_start: 0,
        node_cluster_count: 0,
        child_base,
        child_count,
        traversal_index,
        cluster_budget: work_item.cluster_budget,
        page_budget: work_item.page_budget,
        forced_mip: work_item.forced_mip,
    }
}

fn hierarchy_child_range(
    work_item: VirtualGeometryNodeAndClusterCullClusterWorkItem,
    hierarchy_nodes: &[RenderVirtualGeometryHierarchyNode],
) -> Option<(
    VirtualGeometryNodeAndClusterCullTraversalChildSource,
    u32,
    u32,
)> {
    let hierarchy_node_id = work_item.hierarchy_node_id?;
    let node = hierarchy_nodes.iter().find(|node| {
        node.instance_index == work_item.instance_index && node.node_id == hierarchy_node_id
    })?;
    (node.child_count > 0).then_some((
        VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
        node.child_base,
        node.child_count,
    ))
}

fn cluster_work_item(
    hierarchy_node_id: u32,
    cluster_budget: u32,
) -> VirtualGeometryNodeAndClusterCullClusterWorkItem {
    VirtualGeometryNodeAndClusterCullClusterWorkItem {
        instance_index: 0,
        entity: 42,
        cluster_array_index: hierarchy_node_id,
        hierarchy_node_id: Some(hierarchy_node_id),
        cluster_budget,
        page_budget: 8,
        forced_mip: None,
    }
}

fn hierarchy_node(
    node_id: u32,
    child_base: u32,
    child_count: u32,
) -> RenderVirtualGeometryHierarchyNode {
    RenderVirtualGeometryHierarchyNode {
        instance_index: 0,
        node_id,
        child_base,
        child_count,
        cluster_start: 0,
        cluster_count: 0,
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
