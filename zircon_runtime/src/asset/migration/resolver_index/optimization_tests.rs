use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use zircon_runtime_interface::project::RelPath;

use super::{
    sort_hint_identities, sort_locator_identities, PersistedHintIdentity, PersistedSourceIdentity,
};
use crate::asset::AssetUri;

const IDENTITY_COUNT: usize = 4_096;
const UNIQUE_IDENTITIES: usize = 512;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 16;

fn rel(path: &str) -> RelPath {
    RelPath::parse(path).expect("fixture relative path should parse")
}

fn source_fixture() -> Vec<PersistedSourceIdentity> {
    (0..IDENTITY_COUNT)
        .rev()
        .map(|index| {
            let key = index % UNIQUE_IDENTITIES;
            PersistedSourceIdentity {
                locator: AssetUri::parse(&format!("res://assets/{key:04}.asset"))
                    .expect("fixture locator should parse"),
                project_hint: rel(&format!("project/assets/{key:04}")),
                physical_path: PathBuf::from(format!("D:/assets/{key:04}.asset")),
                physical_root: PathBuf::from("D:/assets"),
                logical_root: rel("project"),
                root_relative: rel(&format!("assets/{key:04}.asset")),
            }
        })
        .collect()
}

fn hint_fixture() -> Vec<PersistedHintIdentity> {
    (0..IDENTITY_COUNT)
        .rev()
        .map(|index| {
            let key = index % UNIQUE_IDENTITIES;
            PersistedHintIdentity {
                locator: AssetUri::parse(&format!("res://assets/{key:04}.asset"))
                    .expect("fixture locator should parse"),
                physical_path: PathBuf::from(format!("D:/assets/{key:04}.asset")),
                physical_root: PathBuf::from("D:/assets"),
            }
        })
        .collect()
}

fn legacy_source_sort(
    mut identities: Vec<PersistedSourceIdentity>,
) -> Vec<PersistedSourceIdentity> {
    identities.sort_by(|left, right| {
        left.project_hint
            .cmp(&right.project_hint)
            .then_with(|| left.physical_root.cmp(&right.physical_root))
            .then_with(|| left.physical_path.cmp(&right.physical_path))
            .then_with(|| left.logical_root.cmp(&right.logical_root))
            .then_with(|| left.root_relative.cmp(&right.root_relative))
    });
    identities.dedup_by(|left, right| {
        left.project_hint == right.project_hint
            && left.physical_root == right.physical_root
            && left.physical_path == right.physical_path
            && left.logical_root == right.logical_root
            && left.root_relative == right.root_relative
    });
    identities
}

fn legacy_hint_sort(mut identities: Vec<PersistedHintIdentity>) -> Vec<PersistedHintIdentity> {
    identities.sort_by(|left, right| {
        left.locator
            .cmp(&right.locator)
            .then_with(|| left.physical_root.cmp(&right.physical_root))
            .then_with(|| left.physical_path.cmp(&right.physical_path))
    });
    identities.dedup_by(|left, right| {
        left.locator == right.locator
            && left.physical_root == right.physical_root
            && left.physical_path == right.physical_path
    });
    identities
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
fn runtime04_resolver_identity_unstable_sort_preserves_deduplicated_order() {
    let expected_sources = legacy_source_sort(source_fixture());
    let mut optimized_sources = source_fixture();
    sort_locator_identities(&mut optimized_sources);
    assert_eq!(optimized_sources, expected_sources);

    let expected_hints = legacy_hint_sort(hint_fixture());
    let mut optimized_hints = hint_fixture();
    sort_hint_identities(&mut optimized_hints);
    assert_eq!(optimized_hints, expected_hints);
    assert_eq!(optimized_sources.len(), UNIQUE_IDENTITIES);
    assert_eq!(optimized_hints.len(), UNIQUE_IDENTITIES);
}

#[test]
fn runtime04_resolver_identity_unstable_sort_source_contract() {
    let source = include_str!("../resolver_index.rs");
    assert!(source.matches("sort_unstable_by").count() >= 2);
    assert!(!source.contains("identities.sort_by("));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_resolver_identity_unstable_sort_bench() {
    let legacy_source_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_source_sort(source_fixture()));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_source_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut values = source_fixture();
                sort_locator_identities(&mut values);
                black_box(values);
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_hint_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_hint_sort(hint_fixture()));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_hint_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut values = hint_fixture();
                sort_hint_identities(&mut values);
                black_box(values);
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_source_p95 = percentile_95(legacy_source_samples);
    let optimized_source_p95 = percentile_95(optimized_source_samples);
    let legacy_hint_p95 = percentile_95(legacy_hint_samples);
    let optimized_hint_p95 = percentile_95(optimized_hint_samples);
    println!(
        "RUNTIME04_RESOLVER_IDENTITY_UNSTABLE_SORT_BENCH_V1 source_legacy_p95_ns={} source_optimized_p95_ns={} hint_legacy_p95_ns={} hint_optimized_p95_ns={} samples={} iterations={} identities={} unique_identities={} stable_sorts=2->0",
        legacy_source_p95,
        optimized_source_p95,
        legacy_hint_p95,
        optimized_hint_p95,
        SAMPLE_COUNT,
        ITERATIONS,
        IDENTITY_COUNT,
        UNIQUE_IDENTITIES,
    );
    assert!(
        optimized_source_p95.saturating_mul(100) <= legacy_source_p95.saturating_mul(95),
        "optimized source identity p95 should be at most 95% of legacy p95"
    );
    assert!(
        optimized_hint_p95.saturating_mul(100) <= legacy_hint_p95.saturating_mul(95),
        "optimized hint identity p95 should be at most 95% of legacy p95"
    );
}
