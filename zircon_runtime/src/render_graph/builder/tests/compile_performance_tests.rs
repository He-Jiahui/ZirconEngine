use std::hint::black_box;
use std::time::Instant;

#[derive(Clone)]
struct BenchmarkPass {
    dependencies: Vec<usize>,
    stable_metadata: [u64; 4],
}

#[test]
fn optimization_batch_hl_runtime588_compile_moves_manual_dependencies() {
    let source = include_str!("../compile.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();
    let compile = production
        .split("pub fn compile")
        .nth(1)
        .expect("render graph compile body")
        .split("fn validate_unique_pass_names")
        .next()
        .expect("compile ends before validation helpers");

    assert!(production.contains("pub fn compile(mut self)"));
    assert!(compile.contains("std::mem::take(&mut pass.dependencies)"));
    assert!(!compile.contains("pass.dependencies.clone()"));
}

fn benchmark_passes(pass_count: usize, dependencies_per_pass: usize) -> Vec<BenchmarkPass> {
    (0..pass_count)
        .map(|pass_index| BenchmarkPass {
            dependencies: (0..dependencies_per_pass)
                .map(|offset| pass_index.saturating_sub(offset + 1))
                .collect(),
            stable_metadata: [pass_index as u64; 4],
        })
        .collect()
}

fn legacy_clone_dependencies(passes: Vec<BenchmarkPass>) -> Vec<Vec<usize>> {
    passes
        .iter()
        .map(|pass| {
            black_box(pass.stable_metadata);
            pass.dependencies.clone()
        })
        .collect()
}

fn move_dependencies(mut passes: Vec<BenchmarkPass>) -> Vec<Vec<usize>> {
    passes
        .iter_mut()
        .map(|pass| {
            black_box(pass.stable_metadata);
            std::mem::take(&mut pass.dependencies)
        })
        .collect()
}

fn measure_dependency_projection(
    input: Vec<BenchmarkPass>,
    project: fn(Vec<BenchmarkPass>) -> Vec<Vec<usize>>,
) -> u128 {
    let started = Instant::now();
    let dependencies = project(input);
    black_box(&dependencies);
    drop(dependencies);
    started.elapsed().as_nanos().max(1)
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_hl_runtime588_manual_dependency_move_performance_evidence() {
    const SAMPLE_PAIRS: usize = 21;
    const PASS_COUNT: usize = 8_192;
    const DEPENDENCIES_PER_PASS: usize = 24;

    let seed = benchmark_passes(PASS_COUNT, DEPENDENCIES_PER_PASS);
    assert_eq!(
        move_dependencies(seed.clone()),
        legacy_clone_dependencies(seed.clone())
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy_input = seed.clone();
        let optimized_input = seed.clone();
        if pair % 2 == 0 {
            legacy_samples.push(measure_dependency_projection(
                legacy_input,
                legacy_clone_dependencies,
            ));
            optimized_samples.push(measure_dependency_projection(
                optimized_input,
                move_dependencies,
            ));
        } else {
            optimized_samples.push(measure_dependency_projection(
                optimized_input,
                move_dependencies,
            ));
            legacy_samples.push(measure_dependency_projection(
                legacy_input,
                legacy_clone_dependencies,
            ));
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "RUNTIME588_RENDER_GRAPH_DEPENDENCY_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             passes_per_sample={PASS_COUNT} dependencies_per_pass={DEPENDENCIES_PER_PASS} \
             legacy_dependency_vec_clones={PASS_COUNT} optimized_dependency_vec_clones=0 \
             legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95 * 100 <= legacy_p95 * 55,
        "moved dependency P95 {optimized_p95}ns exceeded 55% of cloned P95 {legacy_p95}ns"
    );
}
