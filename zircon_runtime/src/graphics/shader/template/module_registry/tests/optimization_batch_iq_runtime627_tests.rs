use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

const MODULE_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn legacy_traversal(tokens: &[String]) -> (usize, usize, usize) {
    let mut visited = HashSet::new();
    let mut modules = HashMap::new();
    let mut ordered = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if visited.insert(token.as_str()) {
            modules.insert(token.as_str(), index);
            ordered.push(token.as_str());
        }
    }
    (visited.len(), modules.len(), ordered.len())
}

fn optimized_traversal(tokens: &[String]) -> (usize, usize, usize) {
    let module_capacity = tokens.len();
    let mut visited = HashSet::with_capacity(module_capacity);
    let mut modules = HashMap::with_capacity(module_capacity);
    let mut ordered = Vec::with_capacity(module_capacity);
    for (index, token) in tokens.iter().enumerate() {
        if visited.insert(token.as_str()) {
            modules.insert(token.as_str(), index);
            ordered.push(token.as_str());
        }
    }
    (visited.len(), modules.len(), ordered.len())
}

#[test]
fn optimization_batch_iq_runtime627_shader_module_traversal_preallocates_known_bounds() {
    let source = include_str!("../../module_registry.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("shader module registry production source");

    assert!(production.contains("source_modules.len().saturating_add(pending.len())"));
    assert!(production.contains("HashSet::with_capacity(traversal_capacity)"));
    assert!(production.contains("HashMap::with_capacity(traversal_capacity)"));
    assert!(production.contains("let module_capacity = self.modules.len();"));
    assert!(production.contains("Vec::with_capacity(module_capacity)"));
    assert!(production.contains("HashSet::with_capacity(module_capacity)"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_iq_runtime627_shader_module_traversal_performance_evidence() {
    let tokens = (0..MODULE_COUNT)
        .map(|index| format!("zr/generated/long_shader_module_{index:05}.wgsl"))
        .collect::<Vec<_>>();
    assert_eq!(legacy_traversal(&tokens), optimized_traversal(&tokens));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_traversal(black_box(&tokens)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_traversal(black_box(&tokens)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_traversal(black_box(&tokens)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_traversal(black_box(&tokens)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "RUNTIME627_PREALLOCATED_SHADER_MODULE_TRAVERSAL_BENCH_V1 modules={MODULE_COUNT} \
         collections=3 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} \
         target_ratio_bp=8500"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(8_500),
        "preallocated shader traversal P95 {optimized_p95} ns exceeded 85% of unreserved {legacy_p95} ns"
    );
}
