use std::hint::black_box;
use std::time::Instant;

use zircon_plugin_navigation_recast::{RecastBackend, RecastBakeMeshInput, RecastTiledBakeInput};

use super::*;

const BENCHMARK_SAMPLE_COUNT: usize = 21;
const CLONES_PER_SAMPLE: usize = 200_000;

#[test]
#[ignore = "release-only performance evidence"]
fn tiled_bake_workers_share_one_plan_allocation() {
    let plan = benchmark_plan();
    let shared_plan = Arc::new(plan.clone());
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || clone_legacy_plan(&plan),
        || clone_shared_plan(&shared_plan),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins14_tiled_bake_shared_plan clones_per_sample={CLONES_PER_SAMPLE} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_arc_refcount_pairs_per_tile=4 optimized_arc_refcount_pairs_per_tile=1 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95 * 4,
        "shared-plan P95 {optimized_p95}ns must be no more than 80% of cloned-plan P95 {legacy_p95}ns"
    );
}

fn benchmark_plan() -> RecastTiledBakePlan {
    RecastBackend::default()
        .prepare_tiled_bake(RecastTiledBakeInput {
            mesh: RecastBakeMeshInput {
                agent_type: "humanoid".to_string(),
                vertices: vec![
                    [-2.0, 0.0, -2.0],
                    [2.0, 0.0, -2.0],
                    [2.0, 0.0, 2.0],
                    [-2.0, 0.0, 2.0],
                ],
                indices: vec![0, 1, 2, 0, 2, 3],
                triangle_areas: Vec::new(),
                default_area: 1,
            },
            tile_size: 1.0,
        })
        .expect("benchmark tiled plan should be valid")
}

fn clone_legacy_plan(plan: &RecastTiledBakePlan) -> usize {
    let mut observed_tiles = 0_usize;
    for _ in 0..CLONES_PER_SAMPLE {
        let cloned = black_box(plan.clone());
        observed_tiles = observed_tiles.wrapping_add(black_box(cloned.tiles().len()));
    }
    observed_tiles
}

fn clone_shared_plan(plan: &Arc<RecastTiledBakePlan>) -> usize {
    let mut observed_tiles = 0_usize;
    for _ in 0..CLONES_PER_SAMPLE {
        let cloned = black_box(Arc::clone(plan));
        observed_tiles = observed_tiles.wrapping_add(black_box(cloned.tiles().len()));
    }
    observed_tiles
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> usize,
    mut optimized: impl FnMut() -> usize,
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

fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    sorted[(sorted.len() * percentile).div_ceil(100) - 1]
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
