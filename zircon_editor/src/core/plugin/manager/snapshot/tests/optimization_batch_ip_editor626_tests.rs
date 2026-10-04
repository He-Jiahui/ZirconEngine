use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

const PACKAGE_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn package_ids() -> Vec<String> {
    (0..PACKAGE_COUNT)
        .map(|index| format!("editor.snapshot.long_package_id_{index:05}"))
        .collect()
}

fn legacy_membership_hits(ids: &[String]) -> usize {
    let packages = ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    ids.iter()
        .filter(|id| packages.contains(id.as_str()))
        .count()
}

fn optimized_membership_hits(ids: &[String]) -> usize {
    let mut packages = HashSet::with_capacity(ids.len());
    packages.extend(ids.iter().map(String::as_str));
    ids.iter()
        .filter(|id| packages.contains(id.as_str()))
        .count()
}

#[test]
fn optimization_batch_ip_editor626_plugin_snapshot_uses_preallocated_hash_membership() {
    let source = include_str!("../../snapshot.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("plugin manager snapshot production source");

    assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(production.contains("HashSet::with_capacity(catalog.registrations().len())"));
    assert!(production.contains("HashSet::with_capacity(entries.len())"));
    assert!(production.contains("registration.package_manifest.id.as_str()"));
    assert!(production.contains("entry.package_id.as_str()"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_ip_editor626_plugin_snapshot_membership_performance_evidence() {
    let ids = package_ids();
    assert_eq!(
        legacy_membership_hits(&ids),
        optimized_membership_hits(&ids)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_membership_hits(black_box(&ids)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_membership_hits(black_box(&ids)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_membership_hits(black_box(&ids)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_membership_hits(black_box(&ids)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "EDITOR626_HASH_PLUGIN_SNAPSHOT_MEMBERSHIP_BENCH_V1 packages={PACKAGE_COUNT} \
         legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=4000"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(4_000),
        "hash snapshot membership P95 {optimized_p95} ns exceeded 40% of tree {legacy_p95} ns"
    );
}
