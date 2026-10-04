#![cfg(feature = "profiling")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

use zircon_runtime::core::diagnostics::profiling::{
    reset_capture, snapshot, start_capture, stop_capture, ProfileCaptureConfig, ProfileScope,
};

const THREAD_COUNTS: [usize; 3] = [1, 8, 64];
const EVENTS_PER_THREAD: [usize; 3] = [0, 100, 10_000];
const SAMPLE_COUNT: usize = 5;

struct CountingAllocator;

#[global_allocator]
static COUNTING_ALLOCATOR: CountingAllocator = CountingAllocator;
static PROFILE_ACTIVE: AtomicBool = AtomicBool::new(false);
static ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        record_allocation(pointer, layout.size());
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        record_allocation(pointer, layout.size());
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let replacement = unsafe { System.realloc(pointer, layout, new_size) };
        record_allocation(replacement, new_size);
        replacement
    }
}

#[derive(Clone, Copy, Debug)]
struct Sample {
    elapsed_ns: u64,
    allocation_count: u64,
    allocated_bytes: u64,
    retained_spans: usize,
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn active_profile_recorder_reports_multithread_scaling_and_allocation_cost() {
    for thread_count in THREAD_COUNTS {
        for events_per_thread in EVENTS_PER_THREAD {
            let mut samples = Vec::with_capacity(SAMPLE_COUNT);
            for sample_index in 0..SAMPLE_COUNT {
                samples.push(profile_scenario(
                    thread_count,
                    events_per_thread,
                    sample_index,
                ));
            }
            samples.sort_unstable_by_key(|sample| sample.elapsed_ns);
            let p50 = samples[SAMPLE_COUNT / 2];
            let p95 = samples[SAMPLE_COUNT - 1];
            let event_count = thread_count.saturating_mul(events_per_thread);
            println!(
                "RUNTIME07_PROFILE_RECORDER_BASELINE_V1 threads={thread_count} events_per_thread={events_per_thread} total_events={event_count} elapsed_p50_ns={} elapsed_p95_ns={} allocations_p50={} allocated_bytes_p50={} retained_spans={}",
                p50.elapsed_ns,
                p95.elapsed_ns,
                p50.allocation_count,
                p50.allocated_bytes,
                p50.retained_spans,
            );
            assert_eq!(p50.retained_spans, event_count);
            assert_eq!(p95.retained_spans, event_count);
        }
    }
}

fn profile_scenario(thread_count: usize, events_per_thread: usize, sample_index: usize) -> Sample {
    reset_capture();
    let event_count = thread_count.saturating_mul(events_per_thread);
    start_capture(ProfileCaptureConfig {
        session_id: format!("runtime07-recorder-{thread_count}-{events_per_thread}-{sample_index}"),
        max_frames: 1,
        max_spans: event_count.max(1),
        max_counters: 1,
        include_perfetto: false,
        ..ProfileCaptureConfig::default()
    });

    let ready = Arc::new(Barrier::new(thread_count + 1));
    let start = Arc::new(Barrier::new(thread_count + 1));
    let workers = (0..thread_count)
        .map(|_| {
            let ready = Arc::clone(&ready);
            let start = Arc::clone(&start);
            thread::spawn(move || {
                ready.wait();
                start.wait();
                for _ in 0..events_per_thread {
                    black_box(ProfileScope::enter(
                        "runtime",
                        "profile.recorder",
                        "static_scope",
                    ));
                }
            })
        })
        .collect::<Vec<_>>();
    ready.wait();

    begin_allocation_profile();
    let started = Instant::now();
    start.wait();
    for worker in workers {
        worker.join().expect("profile worker should finish");
    }
    let elapsed_ns = started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
    let (allocation_count, allocated_bytes) = finish_allocation_profile();

    stop_capture();
    let retained_spans = snapshot().spans.len();
    reset_capture();
    Sample {
        elapsed_ns,
        allocation_count,
        allocated_bytes,
        retained_spans,
    }
}

fn record_allocation(pointer: *mut u8, size: usize) {
    if !pointer.is_null() && PROFILE_ACTIVE.load(Ordering::Relaxed) {
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(size as u64, Ordering::Relaxed);
    }
}

fn begin_allocation_profile() {
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    PROFILE_ACTIVE.store(true, Ordering::SeqCst);
}

fn finish_allocation_profile() -> (u64, u64) {
    PROFILE_ACTIVE.store(false, Ordering::SeqCst);
    (
        ALLOCATION_COUNT.load(Ordering::Relaxed),
        ALLOCATED_BYTES.load(Ordering::Relaxed),
    )
}
