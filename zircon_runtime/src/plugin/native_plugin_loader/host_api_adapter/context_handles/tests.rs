use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

use super::super::super::benchmark_harness::{
    BenchmarkLatencySample, BenchmarkMeasurement, BenchmarkRunMetadata,
    BenchmarkWorkerCompletionGate, BenchmarkWorkerStartGate,
};
use super::{HostContextRegistry, HOST_CONTEXT_PAGE_SLOTS};

use crate::plugin::RuntimeExtensionRegistry;

use super::super::registration_policy::{
    NativeHostApiV4RegistrationPolicy, NativeHostApiV4RegistrationScope,
};
use super::{context_for_v4, context_snapshot};

#[test]
fn native_host_v4_registration_scope_drop_waits_for_an_in_flight_context_pin() {
    let mut registry = RuntimeExtensionRegistry::default();
    let scope = NativeHostApiV4RegistrationScope::new(
        &mut registry,
        "weather.runtime",
        NativeHostApiV4RegistrationPolicy::default(),
    )
    .unwrap();
    let handle = scope.handle();
    let pin = context_for_v4(handle).expect("V4 registration context should be available");

    let release = thread::spawn(move || {
        while !pin.is_closing() {
            thread::yield_now();
        }
        drop(pin);
    });

    drop(scope);
    release
        .join()
        .expect("in-flight V4 registration context should release");
    assert!(context_snapshot(handle.raw()).is_none());
}

struct DropProbe {
    dropped: Arc<AtomicBool>,
}

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::Release);
    }
}

#[test]
fn stale_generation_cannot_resolve_reused_slot() {
    let registry = HostContextRegistry::default();
    let first = registry.insert(Arc::new("first"));

    assert!(registry.remove(first));
    let second = registry.insert(Arc::new("second"));

    assert_eq!(
        HostContextRegistry::<&str>::slot_index(first),
        HostContextRegistry::<&str>::slot_index(second)
    );
    assert_ne!(
        first, second,
        "slot reuse must advance the encoded generation"
    );
    assert!(
        registry.get(first).is_none(),
        "stale handle must stay invalid after slot reuse"
    );
    assert_eq!(registry.get(second).as_deref(), Some(&"second"));
}

#[test]
fn remove_blocks_new_lookups_while_in_flight_arc_finishes() {
    let registry = Arc::new(HostContextRegistry::default());
    let context_dropped = Arc::new(AtomicBool::new(false));
    let handle = registry.insert(Arc::new(DropProbe {
        dropped: context_dropped.clone(),
    }));
    let acquired = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let worker = {
        let registry = registry.clone();
        let acquired = acquired.clone();
        let release = release.clone();
        thread::spawn(move || {
            let _context = registry.get(handle).expect("in-flight lookup");
            acquired.wait();
            release.wait();
        })
    };

    acquired.wait();
    assert!(registry.remove(handle));
    assert!(
        registry.get(handle).is_none(),
        "remove must reject all new lookups"
    );
    assert!(
        !context_dropped.load(Ordering::Acquire),
        "the in-flight lookup Arc must retain the removed context"
    );
    release.wait();

    worker.join().expect("in-flight lookup worker");
    assert!(
        context_dropped.load(Ordering::Acquire),
        "the context should release after the final in-flight lookup Arc exits"
    );
}

#[test]
fn parallel_stable_lookups_never_acquire_writer_lock() {
    const THREADS: usize = 16;
    const LOOKUPS_PER_THREAD: usize = 16_384;

    let registry = Arc::new(HostContextRegistry::default());
    let handle = registry.insert(Arc::new(41_u64));
    let writer_acquires_before = registry.writer_acquire_count();
    let mut workers = Vec::with_capacity(THREADS);
    for _ in 0..THREADS {
        let registry = registry.clone();
        workers.push(thread::spawn(move || {
            let mut sum = 0_u64;
            for _ in 0..LOOKUPS_PER_THREAD {
                sum += *registry.get(handle).expect("stable lookup");
            }
            sum
        }));
    }

    for worker in workers {
        assert_eq!(
            worker.join().expect("parallel lookup worker"),
            41 * LOOKUPS_PER_THREAD as u64
        );
    }
    assert_eq!(registry.writer_acquire_count(), writer_acquires_before);
}

#[test]
fn paged_directory_append_avoids_full_slot_arc_snapshot_copies() {
    for allocations in [1_usize, 100, 10_000] {
        let registry = HostContextRegistry::default();
        for value in 0..allocations {
            registry.insert(Arc::new(value));
        }

        let metrics = registry.directory_metrics();
        let expected_pages = allocations.div_ceil(HOST_CONTEXT_PAGE_SLOTS);
        let max_directory_page_references = expected_pages * expected_pages.saturating_add(1) / 2;

        assert_eq!(metrics.page_count, expected_pages);
        assert_eq!(
            metrics.slot_arc_copies, 0,
            "appending a host context must never clone the complete slot Arc table"
        );
        assert!(
            metrics.directory_page_reference_copies <= max_directory_page_references,
            "directory publication may copy page Arc references only when a page is added"
        );
    }
}

const CONTEXT_BENCHMARK_LOOKUPS: usize = 1_000_000;
const CONTEXT_BENCHMARK_WARMUP_LOOKUPS: usize = 10_000;
const CONTEXT_BENCHMARK_MAX_LATENCY_SAMPLES: usize = 8_192;

#[test]
#[ignore = "manual 1-thread native host context registry throughput evidence"]
fn native_host_context_lookup_1_thread_benchmark() {
    run_stable_lookup_benchmark(1);
}

#[test]
#[ignore = "manual 16-thread native host context registry throughput evidence"]
fn native_host_context_lookup_16_thread_benchmark() {
    run_stable_lookup_benchmark(16);
}

fn run_stable_lookup_benchmark(threads: usize) {
    let metadata = BenchmarkRunMetadata::from_environment(
        "native_host_context_lookup",
        format!("threads={threads},lookups={CONTEXT_BENCHMARK_LOOKUPS}"),
    )
    .expect("benchmark metadata must be bound to a managed optimized-profile run");
    let registry = Arc::new(HostContextRegistry::default());
    let handle = registry.insert(Arc::new(41_u64));
    let writer_acquires_before = registry.writer_acquire_count();

    run_stable_lookup_batch(&registry, handle, threads, CONTEXT_BENCHMARK_WARMUP_LOOKUPS);
    let workers = stable_lookup_workers(
        Arc::clone(&registry),
        handle,
        threads,
        CONTEXT_BENCHMARK_LOOKUPS,
    );
    workers.start.wait_until_ready();
    let started = Instant::now();
    workers.start.start();
    workers.wait_for_completion();
    let elapsed = started.elapsed();
    for worker in workers.threads {
        worker.join().expect("stable lookup benchmark worker");
    }

    let writer_acquires_after = registry.writer_acquire_count();
    assert_eq!(writer_acquires_after, writer_acquires_before);
    let mut latency_sample = run_stable_lookup_latency_sample(&registry, handle, threads);
    assert_eq!(registry.writer_acquire_count(), writer_acquires_before);
    let latency_sample_count = latency_sample.samples_ns.len() as u64;
    let observer_elapsed = latency_sample.observer_elapsed;
    metadata.emit(BenchmarkMeasurement {
        warmup_operations: CONTEXT_BENCHMARK_WARMUP_LOOKUPS as u64,
        measured_operations: CONTEXT_BENCHMARK_LOOKUPS as u64,
        elapsed,
        counters: &[(
            "writer_acquires",
            writer_acquires_after.saturating_sub(writer_acquires_before),
        )],
        latency_sample: Some(BenchmarkLatencySample {
            samples_ns: &mut latency_sample.samples_ns,
            sampling_ratio_numerator: latency_sample_count,
            sampling_ratio_denominator: CONTEXT_BENCHMARK_LOOKUPS as u64,
            observer_elapsed,
        }),
    });
}

struct StableLookupWorkers {
    start: BenchmarkWorkerStartGate,
    completion: BenchmarkWorkerCompletionGate,
    threads: Vec<thread::JoinHandle<()>>,
}

impl StableLookupWorkers {
    fn wait_for_completion(&self) {
        self.completion.wait();
    }
}

struct StableLookupLatencySample {
    samples_ns: Vec<u64>,
    observer_elapsed: std::time::Duration,
}

struct StableLookupLatencySampleWorkers {
    start: Arc<Barrier>,
    threads: Vec<thread::JoinHandle<Vec<u64>>>,
}

fn run_stable_lookup_latency_sample(
    registry: &Arc<HostContextRegistry<u64>>,
    handle: u64,
    threads: usize,
) -> StableLookupLatencySample {
    run_stable_lookup_batch(registry, handle, threads, CONTEXT_BENCHMARK_WARMUP_LOOKUPS);
    let workers = stable_lookup_latency_sample_workers(
        Arc::clone(registry),
        handle,
        threads,
        CONTEXT_BENCHMARK_MAX_LATENCY_SAMPLES,
    );
    let observer_started = Instant::now();
    workers.start.wait();
    let mut samples_ns = Vec::with_capacity(CONTEXT_BENCHMARK_MAX_LATENCY_SAMPLES);
    for worker in workers.threads {
        samples_ns.extend(worker.join().expect("stable lookup latency sample worker"));
    }
    assert_eq!(samples_ns.len(), CONTEXT_BENCHMARK_MAX_LATENCY_SAMPLES);
    StableLookupLatencySample {
        samples_ns,
        observer_elapsed: observer_started.elapsed(),
    }
}

fn run_stable_lookup_batch(
    registry: &Arc<HostContextRegistry<u64>>,
    handle: u64,
    threads: usize,
    lookups: usize,
) {
    let workers = stable_lookup_workers(Arc::clone(registry), handle, threads, lookups);
    workers.start.wait_until_ready();
    workers.start.start();
    workers.wait_for_completion();
    for worker in workers.threads {
        worker.join().expect("stable lookup warm-up worker");
    }
}

fn stable_lookup_workers(
    registry: Arc<HostContextRegistry<u64>>,
    handle: u64,
    threads: usize,
    lookups: usize,
) -> StableLookupWorkers {
    let start = BenchmarkWorkerStartGate::new(threads);
    let completion = BenchmarkWorkerCompletionGate::new(threads);
    let base = lookups / threads;
    let remainder = lookups % threads;
    let threads = (0..threads)
        .map(|worker_index| {
            let registry = Arc::clone(&registry);
            let worker_start = start.worker_start();
            let worker_completion = completion.worker_completion();
            let operations = base + usize::from(worker_index < remainder);
            thread::spawn(move || {
                let _worker_completion = worker_completion;
                worker_start.await_start();
                for _ in 0..operations {
                    std::hint::black_box(registry.get(handle).expect("stable lookup must resolve"));
                }
            })
        })
        .collect();
    StableLookupWorkers {
        start,
        completion,
        threads,
    }
}

fn stable_lookup_latency_sample_workers(
    registry: Arc<HostContextRegistry<u64>>,
    handle: u64,
    threads: usize,
    samples: usize,
) -> StableLookupLatencySampleWorkers {
    let start = Arc::new(Barrier::new(threads + 1));
    let base = samples / threads;
    let remainder = samples % threads;
    let threads = (0..threads)
        .map(|worker_index| {
            let registry = Arc::clone(&registry);
            let start = Arc::clone(&start);
            let samples = base + usize::from(worker_index < remainder);
            thread::spawn(move || {
                let mut samples_ns = Vec::with_capacity(samples);
                start.wait();
                for _ in 0..samples {
                    let lookup_started = Instant::now();
                    std::hint::black_box(registry.get(handle).expect("stable lookup must resolve"));
                    samples_ns.push(
                        lookup_started
                            .elapsed()
                            .as_nanos()
                            .try_into()
                            .unwrap_or(u64::MAX),
                    );
                }
                samples_ns
            })
        })
        .collect();
    StableLookupLatencySampleWorkers { start, threads }
}
