use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use rayon::prelude::*;

use super::{parallel_map_indices, TaskPool};
use crate::core::runtime::tasks::TaskPoolDescriptor;

const SAMPLE_PAIRS: usize = 17;
const CALLS_PER_SAMPLE: usize = 50_000;

#[test]
fn optimization_batch_jt_runtime659_single_index_map_is_exact_once() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
    let calls = AtomicUsize::new(0);

    let empty = parallel_map_indices::<usize, _>(&pool, 0, |_| {
        calls.fetch_add(1, Ordering::Relaxed);
        99
    });
    let single = parallel_map_indices(&pool, 1, |index| {
        calls.fetch_add(1, Ordering::Relaxed);
        index + 73
    });

    assert!(empty.is_empty());
    assert_eq!(single, vec![73]);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[test]
fn optimization_batch_jt_runtime659_single_index_map_bypasses_rayon() {
    let source = include_str!("../../parallel_for.rs");
    let implementation = source
        .split("pub fn parallel_map_indices")
        .nth(1)
        .and_then(|source| source.split("pub fn parallel_map_ordered").next())
        .expect("parallel index-map implementation");

    let single = implementation
        .find("1 => vec![task(0)]")
        .expect("single-index direct map path");
    let rayon = implementation
        .find("pool.install")
        .expect("multi-index Rayon path");
    assert!(single < rayon);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_jt_runtime659_single_index_map_fast_path_bench() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
    for _ in 0..4 {
        black_box(measure(&pool, false));
        black_box(measure(&pool, true));
    }

    let mut rayon_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut direct_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            rayon_samples.push(measure(&pool, false));
            direct_samples.push(measure(&pool, true));
        } else {
            direct_samples.push(measure(&pool, true));
            rayon_samples.push(measure(&pool, false));
        }
    }

    let rayon_p95_ns = percentile(&rayon_samples, 95);
    let direct_p95_ns = percentile(&direct_samples, 95);
    println!(
        "RUNTIME659_SINGLE_INDEX_MAP_FAST_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
calls_per_sample={CALLS_PER_SAMPLE} rayon_p95_ns={rayon_p95_ns} \
direct_p95_ns={direct_p95_ns} rayon_raw_ns={} direct_raw_ns={}",
        sample_csv(&rayon_samples),
        sample_csv(&direct_samples),
    );

    assert!(direct_p95_ns.saturating_mul(100) <= rayon_p95_ns.saturating_mul(40));
}

fn measure(pool: &TaskPool, direct: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for call in 0..CALLS_PER_SAMPLE {
        let values = if direct {
            parallel_map_indices(pool, 1, |index| black_box(index + call))
        } else {
            pool.install(|| {
                (0..1_usize)
                    .into_par_iter()
                    .map(|index| black_box(index + call))
                    .collect::<Vec<_>>()
            })
        };
        checksum ^= values[0];
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
