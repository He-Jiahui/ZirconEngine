use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use zircon_runtime::core::runtime::tasks::{
    EngineTaskGraph, EngineTaskGraphOptions, TaskCancellationPolicy, TaskDescriptor,
    TaskGraphScopeDescriptor, TaskId, TaskPoolKind,
};

const TASK_COUNTS: [usize; 3] = [1, 1_000, 100_000];
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

#[derive(Clone, Copy)]
struct ProfileSample {
    elapsed_ns: u64,
    allocation_count: u64,
    allocated_bytes: u64,
}

#[test]
#[ignore = "managed Windows release allocation and elapsed-time evidence"]
fn runtime11_canonical_task_node_allocation_profile() {
    for task_count in TASK_COUNTS {
        let mut samples = (0..SAMPLE_COUNT)
            .map(|_| profile_task_graph_lifecycle(task_count))
            .collect::<Vec<_>>();
        samples.sort_unstable_by_key(|sample| sample.elapsed_ns);
        let elapsed_p50_ns = samples[SAMPLE_COUNT / 2].elapsed_ns;
        let elapsed_p95_ns = samples[SAMPLE_COUNT - 1].elapsed_ns;
        samples.sort_unstable_by_key(|sample| sample.allocation_count);
        let allocations_p50 = samples[SAMPLE_COUNT / 2].allocation_count;
        samples.sort_unstable_by_key(|sample| sample.allocated_bytes);
        let allocated_bytes_p50 = samples[SAMPLE_COUNT / 2].allocated_bytes;

        println!(
            "RUNTIME11_CANONICAL_TASK_NODE_PROFILE_V1 tasks={task_count} samples={SAMPLE_COUNT} elapsed_p50_ns={elapsed_p50_ns} elapsed_p95_ns={elapsed_p95_ns} allocations_p50={allocations_p50} allocated_bytes_p50={allocated_bytes_p50} allocations_per_task_milli={} allocated_bytes_per_task={}",
            allocations_p50.saturating_mul(1_000) / task_count as u64,
            allocated_bytes_p50 / task_count as u64,
        );
    }
}

fn profile_task_graph_lifecycle(task_count: usize) -> ProfileSample {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("performance profile should create the minimum runtime worker domains");
    let scope = runtime
        .create_scope(
            TaskGraphScopeDescriptor::new("runtime11-allocation-profile")
                .with_task_capacity(task_count),
        )
        .expect("running task graph should create a profiling scope");
    let mut handles = Vec::with_capacity(task_count);

    begin_profile();
    let started_at = Instant::now();
    for index in 0..task_count {
        handles.push(
            scope
                .submit(
                    TaskDescriptor::new(
                        TaskId::new(index as u64 + 1),
                        TaskPoolKind::Compute,
                        "runtime11-profile-task",
                    )
                    .with_cancellation_policy(TaskCancellationPolicy::FinishOnShutdown),
                    |_| {},
                )
                .expect("profiling scope should admit its declared capacity"),
        );
    }
    for handle in &handles {
        handle.wait();
    }
    scope.close_admission();
    let shutdown = runtime
        .shutdown(Duration::from_secs(30))
        .expect("completed profiling tasks should drain and join");
    let elapsed_ns = started_at.elapsed().as_nanos() as u64;
    let (allocation_count, allocated_bytes) = finish_profile();

    assert_eq!(shutdown.scopes.len(), 1);
    let census = scope.census();
    assert_eq!(census.submitted, task_count as u64);
    assert_eq!(census.completed, task_count as u64);
    assert!(census.is_quiescent());
    black_box(handles);

    ProfileSample {
        elapsed_ns,
        allocation_count,
        allocated_bytes,
    }
}

fn begin_profile() {
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    assert!(!PROFILE_ACTIVE.swap(true, Ordering::SeqCst));
}

fn finish_profile() -> (u64, u64) {
    assert!(PROFILE_ACTIVE.swap(false, Ordering::SeqCst));
    (
        ALLOCATION_COUNT.load(Ordering::Relaxed),
        ALLOCATED_BYTES.load(Ordering::Relaxed),
    )
}

fn record_allocation(pointer: *mut u8, size: usize) {
    if !pointer.is_null() && PROFILE_ACTIVE.load(Ordering::Relaxed) {
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(size as u64, Ordering::Relaxed);
    }
}
