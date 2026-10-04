use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

const PACKAGE_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn package_ids() -> Vec<String> {
    (0..PACKAGE_COUNT)
        .map(|index| format!("editor.catalog.long_faulted_package_{index:05}"))
        .collect()
}

fn legacy_faulted_hits(ids: &[String]) -> usize {
    let faulted = ids.iter().cloned().collect::<BTreeSet<_>>();
    ids.iter()
        .filter(|id| faulted.contains(id.as_str()))
        .count()
}

fn optimized_faulted_hits(ids: &[String]) -> usize {
    let mut faulted = HashSet::with_capacity(ids.len());
    faulted.extend(ids.iter().cloned());
    ids.iter()
        .filter(|id| faulted.contains(id.as_str()))
        .count()
}

#[test]
fn optimization_batch_iq_editor627_catalog_snapshot_uses_hash_faulted_package_index() {
    let source = include_str!("../../catalog_snapshot.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("plugin catalog snapshot production source");

    assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(production.contains("faulted_packages: HashSet<String>"));
    assert!(production.contains("HashSet::with_capacity(catalog.registrations().len())"));
    assert!(production.contains("self.faulted_packages.contains(package_id)"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_iq_editor627_faulted_package_index_performance_evidence() {
    let ids = package_ids();
    assert_eq!(legacy_faulted_hits(&ids), optimized_faulted_hits(&ids));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_faulted_hits(black_box(&ids)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_faulted_hits(black_box(&ids)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_faulted_hits(black_box(&ids)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_faulted_hits(black_box(&ids)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "EDITOR627_HASH_FAULTED_PACKAGE_INDEX_BENCH_V1 packages={PACKAGE_COUNT} \
         legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=4000"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(4_000),
        "hash faulted-package P95 {optimized_p95} ns exceeded 40% of tree {legacy_p95} ns"
    );
}
