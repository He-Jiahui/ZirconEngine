use std::collections::VecDeque;
use std::hint::black_box;
use std::rc::Rc;
use std::time::Instant;

use super::{cached_circular_progress_topology, CircularProgressTopology};

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 524_288;

#[test]
fn optimization_batch_20260826ee_editor120_topology_cache_preserves_front_and_lru_order() {
    let front = topology(24);
    let middle = topology(32);
    let tail = topology(48);
    let mut cache = VecDeque::from([Rc::clone(&front), Rc::clone(&middle), Rc::clone(&tail)]);

    let front_hit = cached_circular_progress_topology(&mut cache, 24).unwrap();
    assert!(Rc::ptr_eq(&front_hit, &front));
    assert!(Rc::ptr_eq(&cache[0], &front));
    assert!(Rc::ptr_eq(&cache[1], &middle));
    assert!(Rc::ptr_eq(&cache[2], &tail));

    let middle_hit = cached_circular_progress_topology(&mut cache, 32).unwrap();
    assert!(Rc::ptr_eq(&middle_hit, &middle));
    assert!(Rc::ptr_eq(&cache[0], &middle));
    assert!(Rc::ptr_eq(&cache[1], &front));
    assert!(Rc::ptr_eq(&cache[2], &tail));
}

#[test]
fn optimization_batch_20260826ee_editor120_topology_cache_checks_front_before_search() {
    let source = include_str!("../pixels.rs");
    let helper_start = source.find("fn cached_circular_progress_topology").unwrap();
    let helper_end = source[helper_start..]
        .find("fn build_circular_progress_topology")
        .map(|offset| helper_start + offset)
        .unwrap();
    let helper_source = &source[helper_start..helper_end];
    let front = helper_source.find("cache.front()").unwrap();
    let position = helper_source.find(".position(").unwrap();
    assert!(front < position);
    assert_eq!(helper_source.matches("cache.remove(").count(), 1);
    assert_eq!(helper_source.matches("cache.push_front(").count(), 1);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ee_editor120_circular_progress_topology_front_hit_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR120_CIRCULAR_PROGRESS_TOPOLOGY_FRONT_HIT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} cache_entries=4 legacy_front_mutations_per_lookup=2 \
optimized_front_mutations_per_lookup=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "front topology cache hit P95 {optimized_p95_ns}ns must be at most 70% of remove-and-reinsert P95 {legacy_p95_ns}ns"
    );
}

fn topology(size: u32) -> Rc<CircularProgressTopology> {
    Rc::new(CircularProgressTopology {
        size,
        ring_pixels: Vec::new(),
    })
}

fn legacy_cached_topology(
    cache: &mut VecDeque<Rc<CircularProgressTopology>>,
    size: u32,
) -> Option<Rc<CircularProgressTopology>> {
    let index = cache.iter().position(|topology| topology.size == size)?;
    let topology = cache.remove(index)?;
    cache.push_front(Rc::clone(&topology));
    Some(topology)
}

fn fixture_cache() -> VecDeque<Rc<CircularProgressTopology>> {
    VecDeque::from([topology(24), topology(32), topology(48), topology(64)])
}

fn measure_legacy() -> u128 {
    let mut cache = fixture_cache();
    let started = Instant::now();
    let mut checksum = 0u32;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(legacy_cached_topology(black_box(&mut cache), 24))
            .unwrap()
            .size;
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized() -> u128 {
    let mut cache = fixture_cache();
    let started = Instant::now();
    let mut checksum = 0u32;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(cached_circular_progress_topology(black_box(&mut cache), 24))
            .unwrap()
            .size;
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
