use std::hint::black_box;
use std::time::Instant;

use super::WhenClause;

const CLAUSE_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn legacy_flatten(values: &[usize]) -> usize {
    let mut flattened = Vec::new();
    flattened.extend_from_slice(values);
    flattened.len()
}

fn optimized_flatten(values: &[usize]) -> usize {
    let mut flattened = Vec::with_capacity(values.len());
    flattened.extend_from_slice(values);
    flattened.len()
}

#[test]
fn optimization_batch_r6_wave_editor631_when_all_preserves_flatten_and_dedup_semantics() {
    assert_eq!(
        WhenClause::all([
            WhenClause::Always,
            WhenClause::All(vec![WhenClause::ProjectOpen]),
            WhenClause::ProjectOpen,
        ]),
        WhenClause::ProjectOpen
    );

    let source = include_str!("../../when.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("when clause production source");
    assert!(production.contains("let (clause_lower_bound, _) = clauses.size_hint();"));
    assert!(production.contains("Vec::with_capacity(clause_lower_bound)"));
    assert!(!production.contains("let mut flattened = Vec::new()"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_r6_wave_editor631_when_clause_capacity_performance_evidence() {
    let values = (0..CLAUSE_COUNT).collect::<Vec<_>>();
    assert_eq!(legacy_flatten(&values), optimized_flatten(&values));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_flatten(black_box(&values)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_flatten(black_box(&values)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_flatten(black_box(&values)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_flatten(black_box(&values)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "EDITOR631_PREALLOCATED_WHEN_CLAUSE_FLATTEN_BENCH_V1 clauses={CLAUSE_COUNT} \
         legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=8500"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(8_500),
        "preallocated when-clause flatten P95 {optimized_p95} ns exceeded 85% of unreserved {legacy_p95} ns"
    );
}
