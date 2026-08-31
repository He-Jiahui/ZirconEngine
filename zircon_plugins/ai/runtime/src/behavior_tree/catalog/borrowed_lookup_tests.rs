use std::hint::black_box;
use std::time::Instant;

use super::{
    BehaviorNodeCatalog, BehaviorNodeCategory, BehaviorNodeDescriptor, BehaviorNodeSemantics,
    FrozenBehaviorNodeCatalog,
};
use zircon_runtime::plugin::PluginModuleId;

const CATALOG_ENTRY_COUNT: usize = 256;
const BENCHMARK_LOOKUP_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn borrowed_catalog_lookup_preserves_slot_identity_and_missing_result() {
    let (catalog, ids) = frozen_catalog();
    let first = catalog.resolve(ids[0].as_str()).expect("first slot");
    let last = catalog
        .resolve(ids[CATALOG_ENTRY_COUNT - 1].as_str())
        .expect("last slot");

    assert_eq!(
        catalog.get(first).map(|entry| entry.id()),
        Some(ids[0].as_str())
    );
    assert_eq!(
        catalog.get(last).map(|entry| entry.id()),
        Some(ids[CATALOG_ENTRY_COUNT - 1].as_str())
    );
    assert_eq!(catalog.resolve("missing.behavior.node"), None);
}

#[test]
fn behavior_catalog_resolve_uses_borrowed_extension_key() {
    let catalog_source = include_str!("../catalog.rs");
    let resolve = catalog_source
        .split("pub fn resolve(&self, id: &str)")
        .nth(1)
        .and_then(|body| body.split("pub fn get(").next())
        .expect("behavior catalog resolve body");
    let extension_source = include_str!(
        "../../../../../../zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs"
    );

    assert!(resolve.contains("resolve_borrowed(id)"));
    assert!(!resolve.contains("id.to_string()"));
    assert!(extension_source.contains("pub fn resolve_borrowed<Q>"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn borrowed_behavior_catalog_lookup_release_benchmark_evidence() {
    let (catalog, ids) = frozen_catalog();
    assert_eq!(
        legacy_lookup_checksum(&catalog, &ids),
        borrowed_lookup_checksum(&catalog, &ids)
    );

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_lookup_checksum(black_box(&catalog), black_box(&ids)),
        || borrowed_lookup_checksum(black_box(&catalog), black_box(&ids)),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_borrowed_behavior_catalog_lookup entries={CATALOG_ENTRY_COUNT} lookups={BENCHMARK_LOOKUP_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_string_allocations_per_sample={BENCHMARK_LOOKUP_COUNT} optimized_string_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
    );
}

fn frozen_catalog() -> (FrozenBehaviorNodeCatalog, Vec<String>) {
    let owner = PluginModuleId::from_raw(73);
    let ids = (0..CATALOG_ENTRY_COUNT)
        .map(|index| format!("behavior.node.{index:04}.{}", "x".repeat(48)))
        .collect::<Vec<_>>();
    let mut catalog = BehaviorNodeCatalog::default();
    for id in &ids {
        catalog
            .add_node(
                owner,
                BehaviorNodeDescriptor::new(
                    id,
                    id,
                    BehaviorNodeCategory::Task,
                    BehaviorNodeSemantics::Wait,
                ),
            )
            .expect("unique benchmark node");
    }
    (catalog.freeze(), ids)
}

fn legacy_lookup_checksum(catalog: &FrozenBehaviorNodeCatalog, ids: &[String]) -> u64 {
    let mut checksum = 0_u64;
    for lookup in 0..BENCHMARK_LOOKUP_COUNT {
        let id = ids[lookup % ids.len()].as_str();
        let slot = catalog
            .descriptors
            .resolve(&id.to_string())
            .expect("legacy slot");
        checksum += u64::from(slot.raw()) + 1;
    }
    checksum
}

fn borrowed_lookup_checksum(catalog: &FrozenBehaviorNodeCatalog, ids: &[String]) -> u64 {
    let mut checksum = 0_u64;
    for lookup in 0..BENCHMARK_LOOKUP_COUNT {
        let id = ids[lookup % ids.len()].as_str();
        let slot = catalog.resolve(id).expect("borrowed slot");
        checksum += u64::from(slot.raw()) + 1;
    }
    checksum
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> u64,
    mut optimized: impl FnMut() -> u64,
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

fn benchmark_sample(operation: &mut impl FnMut() -> u64) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
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
