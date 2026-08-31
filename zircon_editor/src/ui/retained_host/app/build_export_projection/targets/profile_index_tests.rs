use std::hint::black_box;
use std::time::Instant;

use super::*;

const PROFILE_COUNT: usize = 2_048;
const SAMPLE_PAIRS: usize = 21;

fn profile(name: impl Into<String>, output_name: impl Into<String>) -> ExportProfile {
    let mut profile = ExportProfile::default();
    profile.name = name.into();
    profile.output_name = output_name.into();
    profile
}

#[test]
fn optimization_batch_20260826co_editor78_profile_index_preserves_first_duplicate_and_missing() {
    let profiles = vec![
        profile("shared", "first"),
        profile("other", "other"),
        profile("shared", "second"),
    ];
    let index = export_profiles_by_name(&profiles);

    assert_eq!(index.get("shared").unwrap().output_name, "first");
    assert_eq!(index.get("other").unwrap().output_name, "other");
    assert!(!index.contains_key("missing"));
}

#[test]
fn optimization_batch_20260826co_editor78_export_targets_use_one_profile_hash_index() {
    let source = include_str!("../targets.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;
    let compact = source.split_whitespace().collect::<String>();

    assert!(source.contains("export_profiles_by_name"));
    assert!(source.contains("HashMap::with_capacity"));
    assert!(!compact.contains("export_profiles.iter().find"));
}

fn benchmark_profiles() -> (Vec<ExportProfile>, Vec<String>) {
    let profiles = (0..PROFILE_COUNT)
        .map(|index| profile(format!("profile-{index:05}"), format!("output-{index:05}")))
        .collect::<Vec<_>>();
    let names = (0..PROFILE_COUNT)
        .rev()
        .map(|index| format!("profile-{index:05}"))
        .collect::<Vec<_>>();
    (profiles, names)
}

fn legacy_resolve(profiles: &[ExportProfile], names: &[String]) -> usize {
    names
        .iter()
        .filter_map(|name| profiles.iter().find(|profile| profile.name == *name))
        .map(|profile| profile.output_name.len())
        .sum()
}

fn optimized_resolve(profiles: &[ExportProfile], names: &[String]) -> usize {
    let index = export_profiles_by_name(profiles);
    names
        .iter()
        .filter_map(|name| index.get(name.as_str()).copied())
        .map(|profile| profile.output_name.len())
        .sum()
}

fn elapsed_ns(run: impl FnOnce() -> usize) -> u128 {
    let started = Instant::now();
    black_box(run());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826co_editor78_export_profile_hash_index_performance_evidence() {
    let (profiles, names) = benchmark_profiles();
    for _ in 0..3 {
        assert_eq!(
            black_box(legacy_resolve(&profiles, &names)),
            optimized_resolve(&profiles, &names)
        );
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(elapsed_ns(|| legacy_resolve(&profiles, &names)));
            optimized_samples.push(elapsed_ns(|| optimized_resolve(&profiles, &names)));
        } else {
            optimized_samples.push(elapsed_ns(|| optimized_resolve(&profiles, &names)));
            legacy_samples.push(elapsed_ns(|| legacy_resolve(&profiles, &names)));
        }
    }

    let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = nearest_rank(&mut legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = nearest_rank(&mut optimized_samples, 95);
    println!(
        "EDITOR78_EXPORT_PROFILE_HASH_INDEX_BENCH_V1 sample_pairs={} profile_count={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        SAMPLE_PAIRS,
        PROFILE_COUNT,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "profile hash-index p95 must be at least 30% below repeated linear scans: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
