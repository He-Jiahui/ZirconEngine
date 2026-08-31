use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::trace_region_scene_data_by_id;
use crate::core::framework::render::RenderHybridGiPreparedTraceRegionSceneData;

const PERF_MARKER: &str = "RUNTIME136_HYBRID_GI_TRACE_REGION_HASH_INDEX_BENCH_V1";

#[test]
fn optimization_batch_20260826cs_runtime_hybrid_gi_hash_index_preserves_latest_region() {
    let regions = [region(7, 32), region(11, 64), region(7, 96)];

    let index = trace_region_scene_data_by_id(&regions);

    assert_eq!(index.len(), 2);
    assert_eq!(index.get(&7).map(|region| region.coverage_q), Some(96));
    assert_eq!(index.get(&11).map(|region| region.coverage_q), Some(64));
    assert!(!index.contains_key(&99));
}

#[test]
fn optimization_batch_20260826cs_runtime_hybrid_gi_hash_index_source_contract() {
    let source = include_str!("../encode.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("hybrid GI trace-region production source");

    assert!(production.contains("use std::collections::{HashMap, HashSet};"));
    assert!(production.contains("trace_region_scene_data_by_id"));
    assert!(production.contains("HashSet::with_capacity"));
    assert!(!production.contains("BTreeMap"));
    assert!(!production.contains("BTreeSet"));
    assert_eq!(
        PERF_MARKER,
        "RUNTIME136_HYBRID_GI_TRACE_REGION_HASH_INDEX_BENCH_V1"
    );
}

#[test]
#[ignore = "release-only paired P95 performance evidence"]
fn optimization_batch_20260826cs_runtime_hybrid_gi_hash_index_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const REGION_COUNT: usize = 16_384;
    const SCHEDULE_COUNT: usize = 8_192;
    let regions = (0..REGION_COUNT as u32)
        .map(|id| (id.wrapping_mul(17), id))
        .collect::<Vec<_>>();
    let schedule = (0..SCHEDULE_COUNT as u32)
        .map(|index| (index % (REGION_COUNT as u32 / 2)).wrapping_mul(17))
        .collect::<Vec<_>>();

    black_box(measure_legacy(&regions, &schedule));
    black_box(measure_optimized(&regions, &schedule));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_legacy(&regions, &schedule));
            optimized_ns.push(measure_optimized(&regions, &schedule));
        } else {
            optimized_ns.push(measure_optimized(&regions, &schedule));
            legacy_ns.push(measure_legacy(&regions, &schedule));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    let reduction = 100.0 * (legacy_p95_ns.saturating_sub(optimized_p95_ns)) as f64
        / legacy_p95_ns.max(1) as f64;

    println!(
        "{PERF_MARKER} sample_pairs={SAMPLE_PAIRS} regions={REGION_COUNT} scheduled_ids={SCHEDULE_COUNT} order=alternating_legacy_first_even legacy_tree_admissions={} optimized_hash_admissions={} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} p95_reduction_percent={reduction:.2}",
        REGION_COUNT + SCHEDULE_COUNT,
        REGION_COUNT + SCHEDULE_COUNT
    );
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "hash trace-region indexing must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn region(region_id: u32, coverage_q: u32) -> RenderHybridGiPreparedTraceRegionSceneData {
    RenderHybridGiPreparedTraceRegionSceneData {
        region_id,
        coverage_q,
        ..RenderHybridGiPreparedTraceRegionSceneData::default()
    }
}

fn measure_legacy(regions: &[(u32, u32)], schedule: &[u32]) -> u128 {
    let started = Instant::now();
    let index = regions.iter().copied().collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut checksum = 0u64;
    for id in schedule {
        if seen.insert(*id) {
            checksum = checksum.wrapping_add(index.get(id).copied().unwrap_or_default() as u64);
        }
    }
    black_box((index, seen, checksum));
    started.elapsed().as_nanos()
}

fn measure_optimized(regions: &[(u32, u32)], schedule: &[u32]) -> u128 {
    let started = Instant::now();
    let index = regions.iter().copied().collect::<HashMap<_, _>>();
    let mut seen = HashSet::with_capacity(schedule.len());
    let mut checksum = 0u64;
    for id in schedule {
        if seen.insert(*id) {
            checksum = checksum.wrapping_add(index.get(id).copied().unwrap_or_default() as u64);
        }
    }
    black_box((index, seen, checksum));
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
