use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_ha_runtime582_all_scope_skips_dependency_frontier_seed() {
    let selected = [true, false, true, true];
    assert!(dependency_frontier_seed(RenderArtifactLoadScope::All, &selected).is_empty());
    assert_eq!(
        dependency_frontier_seed(RenderArtifactLoadScope::Bootstrap, &selected),
        vec![0, 2, 3]
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_ha_runtime582_all_scope_frontier_seed_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const ITERATIONS: usize = 2_048;
    const BLOCKS: usize = 2_048;
    let selected = vec![true; BLOCKS];
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false, &selected, ITERATIONS));
            optimized.push(measure(true, &selected, ITERATIONS));
        } else {
            optimized.push(measure(true, &selected, ITERATIONS));
            legacy.push(measure(false, &selected, ITERATIONS));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME582_ALL_SCOPE_FRONTIER_SEED_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} blocks={BLOCKS} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
        "All-scope frontier bypass must improve P95 by at least 50%"
    );
}

fn measure(optimized: bool, selected: &[bool], iterations: usize) -> u128 {
    let started = Instant::now();
    let mut seeded = 0_usize;
    for _ in 0..iterations {
        let pending = if optimized {
            dependency_frontier_seed(RenderArtifactLoadScope::All, black_box(selected))
        } else {
            black_box(selected)
                .iter()
                .enumerate()
                .filter_map(|(index, selected)| (*selected).then_some(index))
                .collect::<Vec<_>>()
        };
        seeded ^= black_box(pending.len());
    }
    black_box(seeded);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
