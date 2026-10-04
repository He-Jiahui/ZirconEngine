use std::hint::black_box;
use std::time::Instant;

use crate::asset::{asset_kind_for_imported_asset, AssetKind, AssetUri};

use super::{
    builtin_resource, builtin_resources, reset_builtin_construction_stats,
    take_builtin_construction_stats, BuiltinConstructionStats,
};

const EXPECTED_BUILTINS: [(&str, AssetKind); 5] = [
    ("builtin://cube", AssetKind::Model),
    ("builtin://missing-model", AssetKind::Model),
    ("builtin://material/default", AssetKind::Material),
    ("builtin://missing-material", AssetKind::Material),
    ("builtin://shader/pbr.wgsl", AssetKind::Shader),
];
const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 128;

#[test]
fn lazy_builtin_lookup_constructs_only_the_selected_payload() {
    for (selected_index, (locator_text, expected_kind)) in EXPECTED_BUILTINS.iter().enumerate() {
        reset_builtin_construction_stats();
        let locator = AssetUri::parse(locator_text).expect("builtin locator");
        let asset = builtin_resource(&locator, canonical_builtin_locator_matches)
            .expect("registered builtin payload");

        assert_eq!(asset_kind_for_imported_asset(&asset), *expected_kind);
        let stats = take_builtin_construction_stats();
        let mut expected_by_descriptor = [0; EXPECTED_BUILTINS.len()];
        expected_by_descriptor[selected_index] = 1;
        assert_eq!(stats.payload_constructions, 1);
        assert_eq!(stats.by_descriptor, expected_by_descriptor);
        assert!(stats.owned_heap_bytes_proxy > 0);
    }
}

#[test]
fn missing_builtin_lookup_constructs_no_payload() {
    for locator_text in [
        "builtin://not-registered",
        "builtin://cube#unexpected",
        "res://cube",
    ] {
        reset_builtin_construction_stats();
        let locator = AssetUri::parse(locator_text).expect("builtin locator");

        assert!(builtin_resource(&locator, canonical_builtin_locator_matches).is_none());
        assert_eq!(
            take_builtin_construction_stats(),
            BuiltinConstructionStats::default()
        );
    }
}

#[test]
fn bootstrap_and_lazy_lookup_publish_the_same_five_payloads_and_kinds() {
    reset_builtin_construction_stats();
    let bootstrap = builtin_resources();
    let bootstrap_stats = take_builtin_construction_stats();

    assert_eq!(bootstrap.len(), EXPECTED_BUILTINS.len());
    assert_eq!(
        bootstrap_stats.payload_constructions,
        EXPECTED_BUILTINS.len()
    );
    assert_eq!(bootstrap_stats.by_descriptor, [1; EXPECTED_BUILTINS.len()]);
    for ((locator_text, asset), (expected_locator, expected_kind)) in
        bootstrap.iter().zip(EXPECTED_BUILTINS)
    {
        assert_eq!(*locator_text, expected_locator);
        assert_eq!(asset_kind_for_imported_asset(asset), expected_kind);
    }

    reset_builtin_construction_stats();
    for (locator_text, expected_asset) in &bootstrap {
        let locator = AssetUri::parse(locator_text).expect("builtin locator");
        let lazy_asset = builtin_resource(&locator, canonical_builtin_locator_matches)
            .expect("registered builtin payload");
        assert_eq!(&lazy_asset, expected_asset);
    }
    let lazy_stats = take_builtin_construction_stats();
    assert_eq!(lazy_stats.payload_constructions, EXPECTED_BUILTINS.len());
    assert_eq!(lazy_stats.by_descriptor, [1; EXPECTED_BUILTINS.len()]);
    assert_eq!(
        lazy_stats.owned_heap_bytes_proxy,
        bootstrap_stats.owned_heap_bytes_proxy
    );
}

#[test]
fn residency_uses_the_lazy_builtin_lookup() {
    let source = include_str!("../../../project_asset_manager/loading/ensure_resident.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");

    assert!(implementation.contains("builtin_resource("));
    assert!(!implementation.contains("builtin_resources()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn lazy_builtin_lookup_release_evidence() {
    for (name, locator_text, expected_lazy_constructions) in [
        ("first", EXPECTED_BUILTINS[0].0, LOOKUPS_PER_SAMPLE),
        ("last", EXPECTED_BUILTINS[4].0, LOOKUPS_PER_SAMPLE),
        ("missing", "builtin://not-registered", 0),
    ] {
        benchmark_case(name, locator_text, expected_lazy_constructions);
    }
}

#[derive(Clone, Copy)]
struct Measurement {
    elapsed_ns: u128,
    stats: BuiltinConstructionStats,
}

fn benchmark_case(name: &str, locator_text: &str, expected_lazy_constructions: usize) {
    let locator = AssetUri::parse(locator_text).expect("builtin locator");
    black_box(measure_lookup(&locator, false));
    black_box(measure_lookup(&locator, true));

    let mut eager = Vec::with_capacity(SAMPLE_PAIRS);
    let mut lazy = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            eager.push(measure_lookup(&locator, false));
            lazy.push(measure_lookup(&locator, true));
        } else {
            lazy.push(measure_lookup(&locator, true));
            eager.push(measure_lookup(&locator, false));
        }
    }

    let eager_times = eager
        .iter()
        .map(|measurement| measurement.elapsed_ns)
        .collect::<Vec<_>>();
    let lazy_times = lazy
        .iter()
        .map(|measurement| measurement.elapsed_ns)
        .collect::<Vec<_>>();
    let eager_stats = eager[0].stats;
    let lazy_stats = lazy[0].stats;
    assert!(eager.iter().all(|sample| sample.stats == eager_stats));
    assert!(lazy.iter().all(|sample| sample.stats == lazy_stats));
    assert_eq!(
        eager_stats.payload_constructions,
        LOOKUPS_PER_SAMPLE * EXPECTED_BUILTINS.len()
    );
    assert_eq!(
        lazy_stats.payload_constructions,
        expected_lazy_constructions
    );
    assert!(lazy_stats.owned_heap_bytes_proxy <= eager_stats.owned_heap_bytes_proxy);

    let eager_p50_ns = percentile(&eager_times, 50);
    let eager_p95_ns = percentile(&eager_times, 95);
    let eager_p99_ns = percentile(&eager_times, 99);
    let lazy_p50_ns = percentile(&lazy_times, 50);
    let lazy_p95_ns = percentile(&lazy_times, 95);
    let lazy_p99_ns = percentile(&lazy_times, 99);
    println!(
        "RUNTIME_LAZY_BUILTIN_PAYLOAD_LOOKUP_BENCH_V1 case={name} \
sample_pairs={SAMPLE_PAIRS} lookups_per_sample={LOOKUPS_PER_SAMPLE} \
percentile_method=nearest_rank eager_p50_ns={eager_p50_ns} eager_p95_ns={eager_p95_ns} \
eager_p99_ns={eager_p99_ns} lazy_p50_ns={lazy_p50_ns} lazy_p95_ns={lazy_p95_ns} \
lazy_p99_ns={lazy_p99_ns} eager_payload_constructions={} lazy_payload_constructions={} \
eager_owned_heap_bytes_proxy={} lazy_owned_heap_bytes_proxy={} eager_raw_ns={} lazy_raw_ns={}",
        eager_stats.payload_constructions,
        lazy_stats.payload_constructions,
        eager_stats.owned_heap_bytes_proxy,
        lazy_stats.owned_heap_bytes_proxy,
        sample_csv(&eager_times),
        sample_csv(&lazy_times),
    );
    assert!(
        lazy_p95_ns.saturating_mul(100) <= eager_p95_ns.saturating_mul(70),
        "lazy {name} lookup P95 {lazy_p95_ns}ns must be at most 70% of eager P95 {eager_p95_ns}ns"
    );
}

fn measure_lookup(locator: &AssetUri, lazy: bool) -> Measurement {
    reset_builtin_construction_stats();
    let started = Instant::now();
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let asset = if lazy {
            builtin_resource(locator, canonical_builtin_locator_matches)
        } else {
            eager_builtin_resource(locator)
        };
        black_box(asset);
    }
    Measurement {
        elapsed_ns: started.elapsed().as_nanos().max(1),
        stats: take_builtin_construction_stats(),
    }
}

fn eager_builtin_resource(locator: &AssetUri) -> Option<crate::asset::ImportedAsset> {
    builtin_resources()
        .into_iter()
        .find_map(|(candidate, asset)| {
            canonical_builtin_locator_matches(locator, candidate).then_some(asset)
        })
}

fn canonical_builtin_locator_matches(locator: &AssetUri, candidate: &str) -> bool {
    let Some(remainder) = candidate.strip_prefix("builtin://") else {
        return false;
    };
    let (path, label) = match remainder.split_once('#') {
        Some((path, label)) => (path, Some(label)),
        None => (remainder, None),
    };
    locator.path() == path && locator.label() == label
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
