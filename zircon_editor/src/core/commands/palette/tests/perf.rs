use std::hint::black_box;
use std::time::Instant;

const DESCRIPTOR_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn legacy_projection(ids: &[String]) -> (usize, usize) {
    let mut seeds = Vec::new();
    let mut enablement = Vec::new();
    for id in ids {
        seeds.push(id.clone());
        enablement.push(id.len());
    }
    (seeds.len(), enablement.len())
}

fn optimized_projection(ids: &[String]) -> (usize, usize) {
    let mut seeds = Vec::with_capacity(ids.len());
    let mut enablement = Vec::with_capacity(ids.len());
    for id in ids {
        seeds.push(id.clone());
        enablement.push(id.len());
    }
    (seeds.len(), enablement.len())
}

#[test]
fn optimization_batch_ir_editor629_palette_catalog_reserves_descriptor_bounds() {
    let source = include_str!("../../palette.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("command palette production source");

    assert!(production.contains("let (descriptor_lower_bound, _) = descriptors.size_hint();"));
    assert!(production.contains("Vec::with_capacity(descriptor_lower_bound)"));
    assert!(!production.contains("let mut seeds = Vec::new()"));
    assert!(!production.contains("let mut enablement = Vec::new()"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_ir_editor629_palette_catalog_capacity_performance_evidence() {
    let ids = (0..DESCRIPTOR_COUNT)
        .map(|index| format!("editor.command.generated.long.identifier.{index:05}"))
        .collect::<Vec<_>>();
    assert_eq!(legacy_projection(&ids), optimized_projection(&ids));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_projection(black_box(&ids)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_projection(black_box(&ids)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_projection(black_box(&ids)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_projection(black_box(&ids)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "EDITOR629_PREALLOCATED_PALETTE_CATALOG_BENCH_V1 descriptors={DESCRIPTOR_COUNT} \
         vectors=2 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=8500"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(8_500),
        "preallocated palette catalog P95 {optimized_p95} ns exceeded 85% of unreserved {legacy_p95} ns"
    );
}
