#![cfg(feature = "profiling")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

use zircon_runtime::core::diagnostics::profiling::{
    capture_active, reset_capture, snapshot, start_capture, stop_capture, ProfileCaptureConfig,
    ProfileScope,
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
    requested_span_capacity: usize,
    effective_span_capacity: u64,
    written_spans: u64,
    overwritten_spans: u64,
    sample_index: usize,
    attempted_scope_calls: usize,
    capture_active_before: bool,
    capture_active_after: bool,
    feature_enabled: bool,
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
                    true,
                ));
            }
            let mut allocation_counts: [u64; SAMPLE_COUNT] =
                std::array::from_fn(|index| samples[index].allocation_count);
            allocation_counts.sort_unstable();
            let mut allocated_bytes: [u64; SAMPLE_COUNT] =
                std::array::from_fn(|index| samples[index].allocated_bytes);
            allocated_bytes.sort_unstable();
            samples.sort_unstable_by_key(|sample| sample.elapsed_ns);
            let p50 = samples[SAMPLE_COUNT / 2];
            let p95 = samples[SAMPLE_COUNT - 1];
            let event_count = thread_count.saturating_mul(events_per_thread);
            println!(
                "RUNTIME07_PROFILE_RECORDER_BASELINE_V1 threads={thread_count} events_per_thread={events_per_thread} total_events={event_count} elapsed_p50_ns={} elapsed_p95_ns={} allocations_p50={} allocated_bytes_p50={} retained_spans={} requested_span_capacity={} effective_span_capacity={} written_spans={} overwritten_spans={}",
                p50.elapsed_ns,
                p95.elapsed_ns,
                allocation_counts[SAMPLE_COUNT / 2],
                allocated_bytes[SAMPLE_COUNT / 2],
                p50.retained_spans,
                p50.requested_span_capacity,
                p50.effective_span_capacity,
                p50.written_spans,
                p50.overwritten_spans,
            );
            assert_eq!(
                p50.retained_spans as u64,
                (event_count as u64).min(p50.effective_span_capacity)
            );
            assert_eq!(
                p95.retained_spans as u64,
                (event_count as u64).min(p95.effective_span_capacity)
            );

            let mut inactive_samples = Vec::with_capacity(SAMPLE_COUNT);
            for sample_index in 0..SAMPLE_COUNT {
                inactive_samples.push(profile_scenario(
                    thread_count,
                    events_per_thread,
                    sample_index,
                    false,
                ));
            }
            let mut inactive_allocation_counts: [u64; SAMPLE_COUNT] =
                std::array::from_fn(|index| inactive_samples[index].allocation_count);
            inactive_allocation_counts.sort_unstable();
            let mut inactive_allocated_bytes: [u64; SAMPLE_COUNT] =
                std::array::from_fn(|index| inactive_samples[index].allocated_bytes);
            inactive_allocated_bytes.sort_unstable();
            inactive_samples.sort_unstable_by_key(|sample| sample.elapsed_ns);
            let inactive_p50 = inactive_samples[SAMPLE_COUNT / 2];
            let inactive_p95 = inactive_samples[SAMPLE_COUNT - 1];
            let active_ns_per_attempt_p50 =
                (event_count > 0).then(|| p50.elapsed_ns as f64 / event_count as f64);
            let inactive_ns_per_attempt_p50 =
                (event_count > 0).then(|| inactive_p50.elapsed_ns as f64 / event_count as f64);
            let scope_api_wall_ratio_p50 = (inactive_p50.elapsed_ns > 0)
                .then(|| p50.elapsed_ns as f64 / inactive_p50.elapsed_ns as f64);
            println!(
                "RUNTIME03_PROFILE_RECORDER_INACTIVE_CONTROL_V1 threads={thread_count} events_per_thread={events_per_thread} total_events={event_count} samples={SAMPLE_COUNT} attempted_scope_calls={} capture_active_before={} capture_active_after={} feature_enabled={} elapsed_p50_ns={} elapsed_p95_ns={} allocations_p50={} allocated_bytes_p50={} retained_spans={} requested_span_capacity={} effective_span_capacity={} written_spans={} overwritten_spans={} active_ns_per_attempt_p50={active_ns_per_attempt_p50:?} inactive_ns_per_attempt_p50={inactive_ns_per_attempt_p50:?} scope_api_wall_ratio_p50={scope_api_wall_ratio_p50:?} allocator_scope=global_barrier_scope_calls_and_join allocated_bytes=gross_requests_including_realloc p95=nearest_rank_n5_max comparison=scope_api_workload_only numeric_budget=unassigned",
                inactive_p50.attempted_scope_calls,
                inactive_p50.capture_active_before,
                inactive_p50.capture_active_after,
                inactive_p50.feature_enabled,
                inactive_p50.elapsed_ns,
                inactive_p95.elapsed_ns,
                inactive_allocation_counts[SAMPLE_COUNT / 2],
                inactive_allocated_bytes[SAMPLE_COUNT / 2],
                inactive_p50.retained_spans,
                inactive_p50.requested_span_capacity,
                inactive_p50.effective_span_capacity,
                inactive_p50.written_spans,
                inactive_p50.overwritten_spans,
            );
            for sample in samples.iter().chain(&inactive_samples) {
                println!(
                    "RUNTIME03_PROFILE_RECORDER_SCOPE_CONTROL_RAW_V1 threads={thread_count} events_per_thread={events_per_thread} sample_index={} attempted_scope_calls={} capture_active_before={} capture_active_after={} feature_enabled={} elapsed_ns={} allocation_count={} allocated_bytes={} retained_spans={} requested_span_capacity={} effective_span_capacity={} written_spans={} overwritten_spans={} sample_order=active_then_inactive elapsed_sort=independent_per_mode setup_control_assertions_and_output=outside_measurement",
                    sample.sample_index,
                    sample.attempted_scope_calls,
                    sample.capture_active_before,
                    sample.capture_active_after,
                    sample.feature_enabled,
                    sample.elapsed_ns,
                    sample.allocation_count,
                    sample.allocated_bytes,
                    sample.retained_spans,
                    sample.requested_span_capacity,
                    sample.effective_span_capacity,
                    sample.written_spans,
                    sample.overwritten_spans,
                );
            }
        }
    }
}

fn profile_scenario(
    thread_count: usize,
    events_per_thread: usize,
    sample_index: usize,
    capture_enabled: bool,
) -> Sample {
    reset_capture();
    let event_count = thread_count.saturating_mul(events_per_thread);
    let requested_span_capacity = event_count.max(1);
    let config = ProfileCaptureConfig {
        session_id: format!("runtime07-recorder-{thread_count}-{events_per_thread}-{sample_index}"),
        max_frames: 1,
        max_spans: requested_span_capacity,
        max_counters: 1,
        include_perfetto: false,
        ..ProfileCaptureConfig::default()
    };
    let effective_span_capacity = config.clone().normalized().max_spans;
    let started_capture = start_capture(config);
    assert!(started_capture.feature_enabled);
    assert!(started_capture.active);
    if !capture_enabled {
        let stopped_capture = stop_capture();
        assert!(stopped_capture.feature_enabled);
        assert!(!stopped_capture.active);
    }
    let capture_active_before = capture_active();
    assert_eq!(capture_active_before, capture_enabled);

    let ready = Arc::new(Barrier::new(thread_count + 1));
    let start = Arc::new(Barrier::new(thread_count + 1));
    let workers = (0..thread_count)
        .map(|_| {
            let ready = Arc::clone(&ready);
            let start = Arc::clone(&start);
            thread::spawn(move || {
                ready.wait();
                start.wait();
                let mut attempted_scope_calls = 0;
                for _ in 0..events_per_thread {
                    black_box(ProfileScope::enter(
                        "runtime",
                        "profile.recorder",
                        "static_scope",
                    ));
                    attempted_scope_calls += 1;
                }
                attempted_scope_calls
            })
        })
        .collect::<Vec<_>>();
    ready.wait();

    begin_allocation_profile();
    let started = Instant::now();
    start.wait();
    let mut attempted_scope_calls = 0;
    for worker in workers {
        attempted_scope_calls += worker.join().expect("profile worker should finish");
    }
    let elapsed_ns = started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
    let (allocation_count, allocated_bytes) = finish_allocation_profile();

    let capture_active_after = capture_active();
    assert_eq!(capture_active_after, capture_enabled);
    assert_eq!(attempted_scope_calls, event_count);
    let stopped_capture = stop_capture();
    assert!(stopped_capture.feature_enabled);
    assert!(!stopped_capture.active);
    let snapshot = snapshot();
    assert!(snapshot.feature_enabled);
    assert!(!snapshot.active);
    assert_eq!(snapshot.recorder_retention.len(), 1);
    let span_retention = &snapshot.recorder_retention[0].spans;
    let retained_spans = snapshot.spans.len();
    let expected_retained = event_count.min(effective_span_capacity);
    assert_eq!(span_retention.capacity, effective_span_capacity as u64);
    if capture_enabled {
        assert_eq!(span_retention.written, event_count as u64);
        assert_eq!(span_retention.retained, expected_retained as u64);
        assert_eq!(retained_spans, expected_retained);
        assert_eq!(
            span_retention.overwritten,
            (event_count - expected_retained) as u64
        );
    } else {
        assert_eq!(span_retention.written, 0);
        assert_eq!(span_retention.overwritten, 0);
        assert_eq!(span_retention.retained, 0);
        assert_eq!(retained_spans, 0);
    }
    assert_eq!(
        span_retention.written,
        span_retention.retained + span_retention.overwritten
    );
    reset_capture();
    Sample {
        elapsed_ns,
        allocation_count,
        allocated_bytes,
        retained_spans,
        requested_span_capacity,
        effective_span_capacity: span_retention.capacity,
        written_spans: span_retention.written,
        overwritten_spans: span_retention.overwritten,
        sample_index,
        attempted_scope_calls,
        capture_active_before,
        capture_active_after,
        feature_enabled: snapshot.feature_enabled,
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
