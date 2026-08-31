use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::asset::{
    VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset, VirtualGeometryHierarchyNodeAsset,
};
use zircon_runtime::core::framework::render::RenderVirtualGeometryBvhVisualizationInstance;

use super::{
    append_bvh_visualization_instance_if_enabled, render_bvh_visualization_instance,
    subtree_leaf_cluster_index,
};
use crate::virtual_geometry::nanite::cpu_reference::{
    VirtualGeometryCpuReferenceConfig, VirtualGeometryCpuReferenceFrame,
    VirtualGeometryCpuReferenceLeafCluster,
};

const BENCH_TREE_DEPTH: usize = 8;
const CHECKS_PER_SAMPLE: usize = 8;
const DISABLED_CHECKS_PER_SAMPLE: usize = 2;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn bvh_visualization_observer_is_empty_when_disabled_and_complete_when_enabled() {
    let (asset, resident_page_ids) = binary_tree_asset(3);
    let frame = VirtualGeometryCpuReferenceFrame::from_asset(
        7,
        &asset,
        &resident_page_ids,
        VirtualGeometryCpuReferenceConfig::default(),
    );
    let mut output = Vec::new();

    append_bvh_visualization_instance_if_enabled(&mut output, false, 0, &asset, &frame);
    assert!(output.is_empty());

    append_bvh_visualization_instance_if_enabled(&mut output, true, 0, &asset, &frame);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].nodes.len(), asset.hierarchy_buffer.len());
}

#[test]
fn subtree_leaf_cluster_index_preserves_legacy_leaf_order_for_every_node() {
    let (asset, resident_page_ids) = binary_tree_asset(4);
    let frame = VirtualGeometryCpuReferenceFrame::from_asset(
        7,
        &asset,
        &resident_page_ids,
        VirtualGeometryCpuReferenceConfig::default(),
    );

    assert_eq!(
        optimized_subtree_digest(&asset, frame.leaf_clusters()),
        legacy_subtree_digest(&asset, frame.leaf_clusters())
    );
}

#[test]
#[ignore = "release-only disabled BVH observer benchmark"]
fn disabled_bvh_observer_release_benchmark_evidence() {
    let (asset, resident_page_ids) = binary_tree_asset(BENCH_TREE_DEPTH);
    let frame = VirtualGeometryCpuReferenceFrame::from_asset(
        7,
        &asset,
        &resident_page_ids,
        VirtualGeometryCpuReferenceConfig::default(),
    );

    for _ in 0..4 {
        black_box(measure_disabled_legacy(&asset, &frame));
        black_box(measure_disabled_optimized(&asset, &frame));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_disabled_legacy(&asset, &frame));
            optimized_samples.push(measure_disabled_optimized(&asset, &frame));
        } else {
            optimized_samples.push(measure_disabled_optimized(&asset, &frame));
            legacy_samples.push(measure_disabled_legacy(&asset, &frame));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=disabled_bvh_observer_gate \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={DISABLED_CHECKS_PER_SAMPLE} \
tree_depth={BENCH_TREE_DEPTH} node_count={} leaf_count={} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_disabled_behavior=full_observer_projection optimized_disabled_behavior=early_return \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        asset.hierarchy_buffer.len(),
        frame.leaf_clusters().len(),
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns,
        "disabled BVH observer gate must reduce P95 by at least 95%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

#[test]
#[ignore = "release-only BVH subtree leaf index benchmark"]
fn bvh_subtree_leaf_index_release_benchmark_evidence() {
    let (asset, resident_page_ids) = binary_tree_asset(BENCH_TREE_DEPTH);
    let frame = VirtualGeometryCpuReferenceFrame::from_asset(
        7,
        &asset,
        &resident_page_ids,
        VirtualGeometryCpuReferenceConfig::default(),
    );
    assert_eq!(
        optimized_subtree_digest(&asset, frame.leaf_clusters()),
        legacy_subtree_digest(&asset, frame.leaf_clusters())
    );

    for _ in 0..4 {
        black_box(measure_subtree_legacy(&asset, &frame));
        black_box(measure_subtree_optimized(&asset, &frame));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_subtree_legacy(&asset, &frame));
            optimized_samples.push(measure_subtree_optimized(&asset, &frame));
        } else {
            optimized_samples.push(measure_subtree_optimized(&asset, &frame));
            legacy_samples.push(measure_subtree_legacy(&asset, &frame));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=bvh_subtree_leaf_index \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
tree_depth={BENCH_TREE_DEPTH} node_count={} leaf_count={} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_subtree_traversals=per_node legacy_leaf_scans=per_node \
optimized_reverse_parent_index=single optimized_leaf_ancestor_walks=single \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        asset.hierarchy_buffer.len(),
        frame.leaf_clusters().len(),
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns.saturating_mul(3),
        "single BVH subtree leaf index must reduce P95 by at least 85%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_disabled_legacy(
    asset: &VirtualGeometryAsset,
    frame: &VirtualGeometryCpuReferenceFrame,
) -> u128 {
    let started = Instant::now();
    for _ in 0..DISABLED_CHECKS_PER_SAMPLE {
        black_box(render_bvh_visualization_instance(
            0,
            black_box(asset),
            black_box(frame),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_disabled_optimized(
    asset: &VirtualGeometryAsset,
    frame: &VirtualGeometryCpuReferenceFrame,
) -> u128 {
    let started = Instant::now();
    for _ in 0..DISABLED_CHECKS_PER_SAMPLE {
        let mut output = Vec::<RenderVirtualGeometryBvhVisualizationInstance>::new();
        append_bvh_visualization_instance_if_enabled(
            black_box(&mut output),
            false,
            0,
            black_box(asset),
            black_box(frame),
        );
        black_box(output);
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_subtree_legacy(
    asset: &VirtualGeometryAsset,
    frame: &VirtualGeometryCpuReferenceFrame,
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_subtree_digest(
            black_box(asset),
            black_box(frame.leaf_clusters()),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_subtree_optimized(
    asset: &VirtualGeometryAsset,
    frame: &VirtualGeometryCpuReferenceFrame,
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(optimized_subtree_digest(
            black_box(asset),
            black_box(frame.leaf_clusters()),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn optimized_subtree_digest(
    asset: &VirtualGeometryAsset,
    leaf_clusters: &[VirtualGeometryCpuReferenceLeafCluster],
) -> HashMap<u32, Vec<u32>> {
    let nodes_by_id = asset
        .hierarchy_buffer
        .iter()
        .map(|node| (node.node_id, node))
        .collect::<HashMap<_, _>>();
    subtree_leaf_cluster_index(&nodes_by_id, leaf_clusters)
        .into_iter()
        .map(|(node_id, clusters)| {
            (
                node_id,
                clusters
                    .into_iter()
                    .map(VirtualGeometryCpuReferenceLeafCluster::cluster_id)
                    .collect(),
            )
        })
        .collect()
}

fn legacy_subtree_digest(
    asset: &VirtualGeometryAsset,
    leaf_clusters: &[VirtualGeometryCpuReferenceLeafCluster],
) -> HashMap<u32, Vec<u32>> {
    let nodes_by_id = asset
        .hierarchy_buffer
        .iter()
        .map(|node| (node.node_id, node))
        .collect::<BTreeMap<_, _>>();
    asset
        .hierarchy_buffer
        .iter()
        .map(|node| {
            let mut subtree_node_ids = BTreeSet::new();
            legacy_collect_subtree_node_ids(node.node_id, &nodes_by_id, &mut subtree_node_ids);
            let cluster_ids = leaf_clusters
                .iter()
                .filter(|cluster| subtree_node_ids.contains(&cluster.node_id()))
                .map(VirtualGeometryCpuReferenceLeafCluster::cluster_id)
                .collect();
            (node.node_id, cluster_ids)
        })
        .collect()
}

fn legacy_collect_subtree_node_ids(
    node_id: u32,
    nodes_by_id: &BTreeMap<u32, &VirtualGeometryHierarchyNodeAsset>,
    subtree_node_ids: &mut BTreeSet<u32>,
) {
    if !subtree_node_ids.insert(node_id) {
        return;
    }
    let Some(node) = nodes_by_id.get(&node_id).copied() else {
        return;
    };
    for &child_node_id in &node.child_node_ids {
        legacy_collect_subtree_node_ids(child_node_id, nodes_by_id, subtree_node_ids);
    }
}

fn binary_tree_asset(depth: usize) -> (VirtualGeometryAsset, Vec<u32>) {
    let node_count = (1_usize << (depth + 1)) - 1;
    let first_leaf = (1_usize << depth) - 1;
    let mut hierarchy_buffer = Vec::with_capacity(node_count);
    let mut cluster_headers = Vec::with_capacity(node_count - first_leaf);
    let mut resident_page_ids = Vec::with_capacity(node_count - first_leaf);
    for node_index in 0..node_count {
        let is_leaf = node_index >= first_leaf;
        let child_node_ids = if is_leaf {
            Vec::new()
        } else {
            vec![(node_index * 2 + 1) as u32, (node_index * 2 + 2) as u32]
        };
        let cluster_start = cluster_headers.len() as u32;
        if is_leaf {
            let cluster_id = cluster_headers.len() as u32;
            let page_id = cluster_id + 1_000;
            cluster_headers.push(VirtualGeometryClusterHeaderAsset {
                cluster_id,
                hierarchy_node_id: node_index as u32,
                page_id,
                lod_level: depth as u8,
                parent_cluster_id: None,
                bounds_center: [node_index as f32, 0.0, 0.0],
                bounds_radius: 0.5,
                screen_space_error: 1.0,
            });
            resident_page_ids.push(page_id);
        }
        hierarchy_buffer.push(VirtualGeometryHierarchyNodeAsset {
            node_id: node_index as u32,
            parent_node_id: (node_index != 0).then_some(((node_index - 1) / 2) as u32),
            child_node_ids,
            cluster_start,
            cluster_count: u32::from(is_leaf),
            page_id: node_index as u32 + 1_000,
            mip_level: depth as u8,
            bounds_center: [node_index as f32, 0.0, 0.0],
            bounds_radius: 1.0,
            screen_space_error: 1.0,
        });
    }
    (
        VirtualGeometryAsset {
            hierarchy_buffer,
            cluster_headers,
            root_page_table: resident_page_ids.clone(),
            ..VirtualGeometryAsset::default()
        },
        resident_page_ids,
    )
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
