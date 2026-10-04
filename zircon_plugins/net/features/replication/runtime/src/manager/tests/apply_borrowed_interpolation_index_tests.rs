use std::{collections::HashMap, hint::black_box, time::Instant};

use zircon_runtime::core::framework::net::{
    NetObjectId, SyncAuthority, SyncComponentDescriptor, SyncDelta, SyncFieldDescriptor,
    SyncFieldValue,
};

use super::super::state::{NetReplicationInterpolationSample, NetReplicationRuntimeState};
use super::NetReplicationRuntimeManager;

const BENCHMARK_KEY_COUNT: usize = 4_096;
const BENCHMARK_LOOKUP_REPEATS: usize = 64;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

type LegacyInterpolationSamples =
    HashMap<(NetObjectId, String, String), Vec<NetReplicationInterpolationSample>>;

#[test]
fn component_scoped_cleanup_preserves_sibling_interpolation_samples() {
    let manager = NetReplicationRuntimeManager::new();
    for component_type in ["Transform", "TransformProxy"] {
        manager.register_component(
            SyncComponentDescriptor::new(component_type, SyncAuthority::Server)
                .with_field(SyncFieldDescriptor::new("x", "f32")),
        );
    }

    let object = NetObjectId::new(17);
    for (sequence, component_type, value) in
        [(1, "Transform", 3.0_f32), (1, "TransformProxy", 9.0_f32)]
    {
        manager
            .apply_delta_at(
                SyncDelta::new(
                    object,
                    component_type,
                    sequence,
                    [SyncFieldValue::new("x", value.to_le_bytes())],
                ),
                100,
            )
            .expect("registered component accepts delta");
    }

    manager.apply_delta_at(SyncDelta::despawn(object, "Transform", 2), 200);

    assert_eq!(
        manager.interpolated_f32_field_with_delay(object, "Transform", "x", 100, 0),
        None
    );
    assert_eq!(
        manager.interpolated_f32_field_with_delay(object, "TransformProxy", "x", 100, 0),
        Some(9.0)
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn borrowed_interpolation_index_release_benchmark_evidence() {
    let (legacy, optimized, queries) = benchmark_indexes();
    assert_eq!(
        legacy_lookup_checksum(&legacy, &queries),
        optimized_lookup_checksum(&optimized, &queries)
    );

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_lookup_checksum(&legacy, &queries),
        || optimized_lookup_checksum(&optimized, &queries),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);
    let lookups_per_sample = BENCHMARK_KEY_COUNT * BENCHMARK_LOOKUP_REPEATS;

    println!(
        "PERF_RESULT task=plugins10_borrowed_interpolation_index keys={BENCHMARK_KEY_COUNT} lookups_per_sample={lookups_per_sample} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_query_string_allocations_per_sample={} optimized_query_string_allocations_per_sample=0 threshold_percent=15 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_ns} optimized_raw_ns={optimized_ns}",
        lookups_per_sample * 2,
    );
    assert!(
        optimized_p95 * 100 <= legacy_p95 * 85,
        "optimized P95 {optimized_p95}ns must be at least 15% lower than legacy P95 {legacy_p95}ns"
    );
}

fn benchmark_indexes() -> (
    LegacyInterpolationSamples,
    NetReplicationRuntimeState,
    Vec<(NetObjectId, String, String)>,
) {
    let mut legacy = LegacyInterpolationSamples::new();
    let mut optimized = NetReplicationRuntimeState::default();
    let mut queries = Vec::with_capacity(BENCHMARK_KEY_COUNT);
    for raw in 1..=BENCHMARK_KEY_COUNT as u64 {
        let object = NetObjectId::new(raw);
        let component_type = "TransformBenchmarkComponent_LongBorrowedLookupKey".to_string();
        let field_name = "position_axis_LongBorrowedLookupKey".to_string();
        let samples = vec![NetReplicationInterpolationSample {
            time_ms: raw,
            bytes: (raw as f32).to_le_bytes().to_vec(),
        }];
        legacy.insert(
            (object, component_type.clone(), field_name.clone()),
            samples.clone(),
        );
        optimized
            .interpolation_samples
            .entry(component_type.clone())
            .or_default()
            .entry(object)
            .or_default()
            .insert(field_name.clone(), samples);
        queries.push((object, component_type, field_name));
    }
    (legacy, optimized, queries)
}

fn legacy_lookup_checksum(
    samples: &LegacyInterpolationSamples,
    queries: &[(NetObjectId, String, String)],
) -> usize {
    let mut matched = 0;
    for _ in 0..BENCHMARK_LOOKUP_REPEATS {
        for (object, component_type, field_name) in black_box(queries) {
            let key = (
                *object,
                black_box(component_type.as_str()).to_string(),
                black_box(field_name.as_str()).to_string(),
            );
            matched += black_box(samples.get(&key).map_or(0, Vec::len));
        }
    }
    black_box(matched)
}

fn optimized_lookup_checksum(
    state: &NetReplicationRuntimeState,
    queries: &[(NetObjectId, String, String)],
) -> usize {
    let mut matched = 0;
    for _ in 0..BENCHMARK_LOOKUP_REPEATS {
        for (object, component_type, field_name) in black_box(queries) {
            matched += black_box(
                state
                    .interpolation_samples
                    .get(black_box(component_type.as_str()))
                    .and_then(|objects| objects.get(object))
                    .and_then(|fields| fields.get(black_box(field_name.as_str())))
                    .map_or(0, Vec::len),
            );
        }
    }
    black_box(matched)
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> usize,
    mut optimized: impl FnMut() -> usize,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let matched = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(matched);
    elapsed
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (sorted.len() * percentile).div_ceil(100) - 1;
    sorted[index]
}
