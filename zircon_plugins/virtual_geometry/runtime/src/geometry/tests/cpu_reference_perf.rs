use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::asset::{VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset};

use super::{
    build_page_dependency_map, cluster_load_state, known_page_ids, normalized_child_page_ids,
};

const BENCH_LINEAGE_CLUSTER_COUNT: usize = 512;
const BENCH_MEMBERSHIP_PROBE_COUNT: usize = 8_192;
const LINEAGE_CHECKS_PER_SAMPLE: usize = 8;
const MEMBERSHIP_CHECKS_PER_SAMPLE: usize = 64;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn cluster_load_state_reuses_residency_without_changing_forced_mip_semantics() {
    let resident_pages = HashSet::from([10]);

    assert_eq!(
        cluster_load_state(&resident_pages, 10, 3, None),
        (true, true)
    );
    assert_eq!(
        cluster_load_state(&resident_pages, 10, 3, Some(3)),
        (true, true)
    );
    assert_eq!(
        cluster_load_state(&resident_pages, 10, 3, Some(4)),
        (true, false)
    );
    assert_eq!(
        cluster_load_state(&resident_pages, 20, 3, None),
        (false, false)
    );
}

#[test]
fn cpu_reference_cluster_index_preserves_last_duplicate_definition() {
    let asset = VirtualGeometryAsset {
        cluster_headers: vec![
            cluster(1, 10, None),
            cluster(1, 30, None),
            cluster(2, 20, Some(1)),
        ],
        ..VirtualGeometryAsset::default()
    };

    let dependencies = build_page_dependency_map(&asset);
    assert_eq!(dependencies[&20].0, Some(30));
    assert_eq!(dependencies[&30].1, vec![20]);
}

#[test]
#[ignore = "release-only CPU reference resident membership benchmark"]
fn cpu_reference_resident_membership_release_benchmark_evidence() {
    let resident_page_ids = (0..BENCH_MEMBERSHIP_PROBE_COUNT as u32)
        .step_by(2)
        .collect::<Vec<_>>();
    let probes = (0..BENCH_MEMBERSHIP_PROBE_COUNT as u32).collect::<Vec<_>>();
    assert_eq!(
        membership_digest_optimized(&resident_page_ids, &probes),
        membership_digest_legacy(&resident_page_ids, &probes)
    );

    for _ in 0..4 {
        black_box(measure_membership_legacy(&resident_page_ids, &probes));
        black_box(measure_membership_optimized(&resident_page_ids, &probes));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_membership_legacy(&resident_page_ids, &probes));
            optimized_samples.push(measure_membership_optimized(&resident_page_ids, &probes));
        } else {
            optimized_samples.push(measure_membership_optimized(&resident_page_ids, &probes));
            legacy_samples.push(measure_membership_legacy(&resident_page_ids, &probes));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=cpu_reference_cluster_residency_reuse \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={MEMBERSHIP_CHECKS_PER_SAMPLE} \
resident_count={} probe_count={BENCH_MEMBERSHIP_PROBE_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_membership_lookups_per_cluster=2 optimized_membership_lookups_per_cluster=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        resident_page_ids.len(),
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns.saturating_mul(13),
        "reused CPU reference residency lookup must reduce P95 by at least 35%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

#[test]
#[ignore = "release-only CPU reference page dependency index benchmark"]
fn cpu_reference_page_dependency_index_release_benchmark_evidence() {
    let asset = VirtualGeometryAsset {
        cluster_headers: (0..BENCH_LINEAGE_CLUSTER_COUNT as u32)
            .map(|cluster_id| cluster(cluster_id, 10, cluster_id.checked_sub(1)))
            .collect(),
        ..VirtualGeometryAsset::default()
    };
    assert_eq!(
        build_page_dependency_map(&asset),
        legacy_page_dependency_map(&asset)
    );

    for _ in 0..4 {
        black_box(measure_lineage_legacy(&asset));
        black_box(measure_lineage_optimized(&asset));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_lineage_legacy(&asset));
            optimized_samples.push(measure_lineage_optimized(&asset));
        } else {
            optimized_samples.push(measure_lineage_optimized(&asset));
            legacy_samples.push(measure_lineage_legacy(&asset));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=cpu_reference_page_dependency_hash_indexes \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={LINEAGE_CHECKS_PER_SAMPLE} \
cluster_count={BENCH_LINEAGE_CLUSTER_COUNT} lineage_shape=single_page_chain \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_cluster_index=btree legacy_cycle_index=btree \
optimized_cluster_index=preallocated_hash optimized_cycle_index=hash ordered_output=btree \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
        "hashed CPU reference dependency indexes must reduce P95 by at least 25%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_membership_legacy(resident_page_ids: &[u32], probes: &[u32]) -> u128 {
    let started = Instant::now();
    for _ in 0..MEMBERSHIP_CHECKS_PER_SAMPLE {
        black_box(membership_digest_legacy(
            black_box(resident_page_ids),
            black_box(probes),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_membership_optimized(resident_page_ids: &[u32], probes: &[u32]) -> u128 {
    let started = Instant::now();
    for _ in 0..MEMBERSHIP_CHECKS_PER_SAMPLE {
        black_box(membership_digest_optimized(
            black_box(resident_page_ids),
            black_box(probes),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn membership_digest_legacy(resident_page_ids: &[u32], probes: &[u32]) -> usize {
    let resident_pages = resident_page_ids.iter().copied().collect::<BTreeSet<_>>();
    probes
        .iter()
        .filter(|&&page_id| {
            let loaded = resident_pages.contains(&page_id);
            let selected = resident_pages.contains(&page_id);
            loaded as usize + selected as usize == 2
        })
        .count()
}

fn membership_digest_optimized(resident_page_ids: &[u32], probes: &[u32]) -> usize {
    let resident_pages = resident_page_ids.iter().copied().collect::<BTreeSet<_>>();
    let mut resident_page_index = HashSet::with_capacity(resident_pages.len());
    resident_page_index.extend(resident_pages.iter().copied());
    black_box(resident_pages);
    probes
        .iter()
        .filter(|&&page_id| {
            cluster_load_state(&resident_page_index, page_id, 0, None) == (true, true)
        })
        .count()
}

fn measure_lineage_legacy(asset: &VirtualGeometryAsset) -> u128 {
    let started = Instant::now();
    for _ in 0..LINEAGE_CHECKS_PER_SAMPLE {
        black_box(legacy_page_dependency_map(black_box(asset)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_lineage_optimized(asset: &VirtualGeometryAsset) -> u128 {
    let started = Instant::now();
    for _ in 0..LINEAGE_CHECKS_PER_SAMPLE {
        black_box(build_page_dependency_map(black_box(asset)));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_page_dependency_map(
    asset: &VirtualGeometryAsset,
) -> BTreeMap<u32, (Option<u32>, Vec<u32>)> {
    let mut dependencies = known_page_ids(asset)
        .into_iter()
        .map(|page_id| (page_id, (None, Vec::new())))
        .collect::<BTreeMap<_, _>>();
    let clusters_by_id = asset
        .cluster_headers
        .iter()
        .map(|cluster| (cluster.cluster_id, cluster))
        .collect::<BTreeMap<_, _>>();
    for cluster in &asset.cluster_headers {
        let Some(parent_page_id) = legacy_nearest_parent_page(cluster, &clusters_by_id) else {
            continue;
        };
        dependencies.entry(cluster.page_id).or_default().0 = Some(parent_page_id);
        dependencies
            .entry(parent_page_id)
            .or_default()
            .1
            .push(cluster.page_id);
    }
    for (_, child_page_ids) in dependencies.values_mut() {
        *child_page_ids = normalized_child_page_ids(child_page_ids);
    }
    dependencies
}

fn legacy_nearest_parent_page(
    cluster: &VirtualGeometryClusterHeaderAsset,
    clusters_by_id: &BTreeMap<u32, &VirtualGeometryClusterHeaderAsset>,
) -> Option<u32> {
    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = BTreeSet::new();
    while let Some(parent_cluster_id) = current_parent_cluster_id {
        if !visited_cluster_ids.insert(parent_cluster_id) {
            break;
        }
        let parent_cluster = clusters_by_id.get(&parent_cluster_id).copied()?;
        if parent_cluster.page_id != cluster.page_id {
            return Some(parent_cluster.page_id);
        }
        current_parent_cluster_id = parent_cluster.parent_cluster_id;
    }
    None
}

fn cluster(
    cluster_id: u32,
    page_id: u32,
    parent_cluster_id: Option<u32>,
) -> VirtualGeometryClusterHeaderAsset {
    VirtualGeometryClusterHeaderAsset {
        cluster_id,
        hierarchy_node_id: cluster_id,
        page_id,
        lod_level: 0,
        parent_cluster_id,
        bounds_center: [0.0, 0.0, 0.0],
        bounds_radius: 0.5,
        screen_space_error: 1.0,
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
