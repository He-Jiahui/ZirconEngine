use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Instant;

use super::*;

const BENCHMARK_ADMISSIONS: usize = 256;
const BENCHMARK_KEY_BYTES: usize = 2_048;
const BENCHMARK_COMPLETIONS: usize = 4_096;
const BENCHMARK_ITERATIONS: usize = 32;
const BENCHMARK_SAMPLES: usize = 11;

#[derive(Debug)]
struct CountingKey {
    id: usize,
    clones: Arc<AtomicUsize>,
    hashes: Arc<AtomicUsize>,
}

impl CountingKey {
    fn new(id: usize, clones: &Arc<AtomicUsize>, hashes: &Arc<AtomicUsize>) -> Self {
        Self {
            id,
            clones: Arc::clone(clones),
            hashes: Arc::clone(hashes),
        }
    }
}

impl Clone for CountingKey {
    fn clone(&self) -> Self {
        self.clones.fetch_add(1, Ordering::Relaxed);
        Self::new(self.id, &self.clones, &self.hashes)
    }
}

impl PartialEq for CountingKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for CountingKey {}

impl Hash for CountingKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hashes.fetch_add(1, Ordering::Relaxed);
        self.id.hash(state);
    }
}

#[test]
fn runtime59_async_pipeline_single_clone_admission_preserves_queueing() {
    let clones = Arc::new(AtomicUsize::new(0));
    let hashes = Arc::new(AtomicUsize::new(0));
    let mut compiler = PipelineAsyncCompiler::new("single-clone-admission", 1).unwrap();

    assert_eq!(
        compiler.try_queue(CountingKey::new(7, &clones, &hashes), || 19_u32),
        PipelineAsyncQueueResult::Queued
    );
    assert_eq!(clones.load(Ordering::Relaxed), 1);

    let mut completed = Vec::new();
    compiler.finish_pending(|key, result| completed.push((key.id, result)));
    assert_eq!(completed, [(7, Ok(19))]);

    let source = include_str!("../async_compile.rs");
    let try_queue = function_body(
        source,
        "    pub(crate) fn try_queue(",
        "    pub(crate) fn drain_ready(",
    );
    assert_eq!(try_queue.matches("key.clone()").count(), 1);
    assert!(try_queue.contains("request.key"));
}

#[test]
fn runtime59_async_pipeline_single_target_probe_preserves_fifo_boundary() {
    const COMPLETIONS: usize = 64;

    let clones = Arc::new(AtomicUsize::new(0));
    let hashes = Arc::new(AtomicUsize::new(0));
    let mut compiler = PipelineAsyncCompiler::new("single-target-probe", COMPLETIONS).unwrap();
    let (release_first, wait_first) = mpsc::sync_channel(0);
    assert_eq!(
        compiler.try_queue(CountingKey::new(0, &clones, &hashes), move || {
            wait_first.recv().expect("test releases the first compile");
            0_usize
        }),
        PipelineAsyncQueueResult::Queued
    );
    for id in 1..COMPLETIONS {
        assert_eq!(
            compiler.try_queue(CountingKey::new(id, &clones, &hashes), move || id),
            PipelineAsyncQueueResult::Queued
        );
    }

    let (wait_started, observe_wait_started) = mpsc::sync_channel(0);
    compiler.set_target_sync_wait_observer(wait_started);
    let release = std::thread::spawn(move || {
        observe_wait_started
            .recv()
            .expect("target wait should begin");
        release_first
            .send(())
            .expect("blocked compile should still be waiting");
    });
    hashes.store(0, Ordering::Relaxed);
    let target = CountingKey::new(COMPLETIONS - 1, &clones, &hashes);
    let mut completed = Vec::new();

    assert_eq!(
        compiler
            .finish_pending_through(&target, |key, result| { completed.push((key.id, result)) }),
        COMPLETIONS
    );
    release.join().unwrap();

    assert_eq!(completed.len(), COMPLETIONS);
    assert_eq!(completed.first(), Some(&(0, Ok(0))));
    assert_eq!(
        completed.last(),
        Some(&(COMPLETIONS - 1, Ok(COMPLETIONS - 1)))
    );
    assert_eq!(hashes.load(Ordering::Relaxed), COMPLETIONS + 1);

    let source = include_str!("../async_compile.rs");
    let finish = function_body(
        source,
        "    pub(crate) fn finish_pending_through(",
        "    pub(crate) fn set_target_sync_wait_observer(",
    );
    assert!(!finish.contains("while self.pending.contains(target)"));
    assert!(finish.contains("reached_target"));
}

#[test]
#[ignore = "release performance gate; run through the managed Runtime59 validator"]
fn runtime59_async_pipeline_single_clone_admission_release_benchmark() {
    let source = (0..BENCHMARK_ADMISSIONS)
        .map(|id| BenchKey {
            id,
            payload: vec![id as u8; BENCHMARK_KEY_BYTES],
        })
        .collect::<Vec<_>>();
    assert_eq!(
        admission_projection(source.clone(), true),
        admission_projection(source.clone(), false)
    );

    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_admission_projection(&source, true));
            optimized_samples.push(measure_admission_projection(&source, false));
        } else {
            optimized_samples.push(measure_admission_projection(&source, false));
            retired_samples.push(measure_admission_projection(&source, true));
        }
    }

    let retired_p95 = nearest_rank(&retired_samples, 95);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let reduction_basis_points = reduction_basis_points(retired_p95, optimized_p95);
    println!(
        "RUNTIME59_SINGLE_CLONE_PIPELINE_ADMISSION_BENCH_V1 samples={} sample_order=alternating percentile_method=nearest_rank admissions={} key_bytes={} iterations={} retired_key_clones={} optimized_key_clones={} retired_cloned_payload_bytes={} optimized_cloned_payload_bytes={} retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} retired_ns={} optimized_ns={}",
        BENCHMARK_SAMPLES,
        BENCHMARK_ADMISSIONS,
        BENCHMARK_KEY_BYTES,
        BENCHMARK_ITERATIONS,
        BENCHMARK_ADMISSIONS * 2,
        BENCHMARK_ADMISSIONS,
        BENCHMARK_ADMISSIONS * BENCHMARK_KEY_BYTES * 2,
        BENCHMARK_ADMISSIONS * BENCHMARK_KEY_BYTES,
        retired_p95,
        optimized_p95,
        reduction_basis_points,
        join_samples(&retired_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= retired_p95.saturating_mul(70),
        "single-clone pipeline admission P95 must be at most 70% of retired: retired={retired_p95}ns optimized={optimized_p95}ns"
    );
}

#[test]
#[ignore = "release performance gate; run through the managed Runtime59 validator"]
fn runtime59_async_pipeline_single_target_probe_release_benchmark() {
    let completions = (0..BENCHMARK_COMPLETIONS).collect::<Vec<_>>();
    let pending = completions.iter().copied().collect::<HashSet<_>>();
    let target = BENCHMARK_COMPLETIONS - 1;
    assert_eq!(
        target_drain_projection(pending.clone(), &completions, target, true),
        target_drain_projection(pending.clone(), &completions, target, false)
    );

    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_target_drain(&pending, &completions, target, true));
            optimized_samples.push(measure_target_drain(&pending, &completions, target, false));
        } else {
            optimized_samples.push(measure_target_drain(&pending, &completions, target, false));
            retired_samples.push(measure_target_drain(&pending, &completions, target, true));
        }
    }

    let retired_p95 = nearest_rank(&retired_samples, 95);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let reduction_basis_points = reduction_basis_points(retired_p95, optimized_p95);
    println!(
        "RUNTIME59_SINGLE_PROBE_TARGET_COMPLETION_DRAIN_BENCH_V1 samples={} sample_order=alternating percentile_method=nearest_rank completions={} iterations={} retired_target_hash_probes={} optimized_target_hash_probes=1 retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} retired_ns={} optimized_ns={}",
        BENCHMARK_SAMPLES,
        BENCHMARK_COMPLETIONS,
        BENCHMARK_ITERATIONS,
        BENCHMARK_COMPLETIONS,
        retired_p95,
        optimized_p95,
        reduction_basis_points,
        join_samples(&retired_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= retired_p95.saturating_mul(75),
        "single-probe target drain P95 must be at most 75% of retired: retired={retired_p95}ns optimized={optimized_p95}ns"
    );
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct BenchKey {
    id: usize,
    payload: Vec<u8>,
}

fn admission_projection(keys: Vec<BenchKey>, retired: bool) -> (HashSet<BenchKey>, Vec<BenchKey>) {
    let mut pending = HashSet::with_capacity(keys.len());
    let mut requests = Vec::with_capacity(keys.len());
    for key in keys {
        pending.insert(key.clone());
        requests.push(if retired { key.clone() } else { key });
    }
    (pending, requests)
}

fn measure_admission_projection(source: &[BenchKey], retired: bool) -> u128 {
    let mut elapsed = 0;
    for _ in 0..BENCHMARK_ITERATIONS {
        let keys = source.to_vec();
        let started = Instant::now();
        black_box(admission_projection(keys, retired));
        elapsed += started.elapsed().as_nanos();
    }
    elapsed
}

fn target_drain_projection(
    mut pending: HashSet<usize>,
    completions: &[usize],
    target: usize,
    retired: bool,
) -> Vec<usize> {
    let mut completed = Vec::with_capacity(completions.len());
    if retired {
        let mut index = 0;
        while pending.contains(&target) {
            let key = completions[index];
            pending.remove(&key);
            completed.push(key);
            index += 1;
        }
    } else if pending.contains(&target) {
        for &key in completions {
            let reached_target = key == target;
            pending.remove(&key);
            completed.push(key);
            if reached_target {
                break;
            }
        }
    }
    completed
}

fn measure_target_drain(
    source_pending: &HashSet<usize>,
    completions: &[usize],
    target: usize,
    retired: bool,
) -> u128 {
    let mut elapsed = 0;
    for _ in 0..BENCHMARK_ITERATIONS {
        let pending = source_pending.clone();
        let started = Instant::now();
        black_box(target_drain_projection(
            pending,
            completions,
            target,
            retired,
        ));
        elapsed += started.elapsed().as_nanos();
    }
    elapsed
}

fn function_body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|body| body.split(end).next())
        .expect("function source should remain available")
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100).max(1);
    sorted[rank - 1]
}

fn reduction_basis_points(retired_ns: u128, optimized_ns: u128) -> u128 {
    if retired_ns == 0 {
        return 0;
    }
    retired_ns
        .saturating_sub(optimized_ns)
        .saturating_mul(10_000)
        / retired_ns
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
