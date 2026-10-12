#![cfg(feature = "profiling")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use zircon_runtime::core::diagnostics::profiling::{
    reset_capture, snapshot, start_capture, stop_capture, ProfileCaptureConfig, ProfileFrameScope,
    ProfileScope,
};

const SAMPLE_COUNT: usize = 101;
const LEAVES_PER_SAMPLE: usize = 128;
const WARMUP_BATCHES: usize = 2;
const DEPTHS: [usize; 4] = [0, 1, 8, 64];

struct CountingAllocator;

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
thread_local! {
    static MEASURING: Cell<bool> = const { Cell::new(false) };
}
static REQUESTS: AtomicU64 = AtomicU64::new(0);
static GROSS_BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Forward the caller's allocation contract to System.
        let pointer = unsafe { System.alloc(layout) };
        record_request(pointer, layout.size());
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Forward the caller's zeroed allocation contract to System.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        record_request(pointer, layout.size());
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: Forward the caller's matching live allocation to System.
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: Forward the caller's live allocation and new size to System.
        let replacement = unsafe { System.realloc(pointer, layout, size) };
        record_request(replacement, size);
        replacement
    }
}

#[derive(Clone, Copy, Debug)]
enum Shape {
    Current,
    RetiredEpoch,
    NonLifo,
}

impl Shape {
    fn label(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::RetiredEpoch => "retired_epoch",
            Self::NonLifo => "non_lifo",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Sample {
    wall_ns: u64,
    requests: u64,
    gross_bytes: u64,
    leaf_path_bytes: usize,
    live_depth: u16,
}

#[test]
#[ignore = "native Release current-source nested scope allocator evidence"]
fn runtime03_nested_scope_allocation_profile() {
    assert!(!cfg!(debug_assertions), "this evidence requires Release");
    println!(
        "RUNTIME03_NESTED_SCOPE_ENV_V1 os={} arch={} samples={SAMPLE_COUNT} leaves_per_sample={LEAVES_PER_SAMPLE} warmup_batches={WARMUP_BATCHES} scope_source_blake3={} probe_source_blake3={} measured_scope=public_leaf_enter_and_drop_with_live_ancestors setup_and_snapshot=outside_window allocator_scope=workload_thread requested_bytes=gross_including_realloc instrumented_wall=allocator_counters_enabled product_cpu_rss_power_contention=unmeasured gain_acceptance=requires_same_probe_on_hash_bound_before_and_after_source numeric_budget=unchanged",
        std::env::consts::OS,
        std::env::consts::ARCH,
        blake3::hash(include_str!("../src/core/runtime/diagnostics/profiling/scope.rs").as_bytes()),
        blake3::hash(include_str!("runtime03_nested_scope_allocation_profile.rs").as_bytes()),
    );
    for shape in [Shape::Current, Shape::RetiredEpoch, Shape::NonLifo] {
        for depth in DEPTHS {
            if matches!(shape, Shape::NonLifo) && depth == 0 {
                continue;
            }
            let samples: Vec<_> = (0..SAMPLE_COUNT)
                .map(|index| profile_sample(shape, depth, index))
                .collect();
            let wall_ns: Vec<_> = samples.iter().map(|sample| sample.wall_ns).collect();
            let requests: Vec<_> = samples.iter().map(|sample| sample.requests).collect();
            let gross_bytes: Vec<_> = samples.iter().map(|sample| sample.gross_bytes).collect();
            println!(
                "RUNTIME03_NESTED_SCOPE_PROFILE_V1 shape={} authored_depth={depth} live_depth={} leaf_path_bytes={} wall_ns_p50={} wall_ns_p95={} wall_ns_p99={} requests_p50={} requests_p95={} requests_p99={} gross_bytes_p50={} gross_bytes_p95={} gross_bytes_p99={} wall_ns_raw={wall_ns:?} requests_raw={requests:?} gross_bytes_raw={gross_bytes:?}",
                shape.label(),
                samples[0].live_depth,
                samples[0].leaf_path_bytes,
                percentile(&wall_ns, 50),
                percentile(&wall_ns, 95),
                percentile(&wall_ns, 99),
                percentile(&requests, 50),
                percentile(&requests, 95),
                percentile(&requests, 99),
                percentile(&gross_bytes, 50),
                percentile(&gross_bytes, 95),
                percentile(&gross_bytes, 99),
            );
        }
    }
}

fn profile_sample(shape: Shape, depth: usize, index: usize) -> Sample {
    reset_capture();
    let config = ProfileCaptureConfig {
        session_id: format!("runtime03-nested-{}-{depth}-{index}", shape.label()),
        max_frames: 1,
        max_spans: LEAVES_PER_SAMPLE,
        max_counters: 1,
        include_perfetto: false,
        ..ProfileCaptureConfig::default()
    };
    let retired = if matches!(shape, Shape::RetiredEpoch) {
        assert!(start_capture(config.clone()).active);
        let retired = ProfileScope::enter("retired", "old", "retired");
        reset_capture();
        Some(retired)
    } else {
        None
    };
    assert!(start_capture(config).active);
    let frame = ProfileFrameScope::enter("runtime", "nested_fixture");
    let mut ancestors: Vec<_> = (0..depth)
        .map(|_| ProfileScope::enter("runtime", "profile.nested", "祖/先:scope"))
        .collect();
    if matches!(shape, Shape::NonLifo) {
        drop(ancestors.remove(0));
    }
    let live_depth = ancestors.len();
    let mut parent_path = "runtime".to_string();
    if live_depth > 0 {
        for _ in 0..depth {
            parent_path.push_str("/profile.nested:祖/先:scope");
        }
    }
    let expected_leaf_path = format!("{parent_path}/profile.nested:leaf");
    // Fill the real recorder ring and TLS capacity before the measured leaf batch.
    for _ in 0..WARMUP_BATCHES {
        leaves();
    }

    begin_profile();
    let started = Instant::now();
    leaves();
    let wall_ns = started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
    let (requests, gross_bytes) = end_profile();

    stop_capture();
    let leaf_snapshot = snapshot();
    assert_eq!(leaf_snapshot.spans.len(), LEAVES_PER_SAMPLE);
    for span in &leaf_snapshot.spans {
        assert_eq!(span.name, "leaf");
        assert_eq!(span.path, expected_leaf_path);
        assert_eq!(usize::from(span.depth), live_depth);
    }
    drop(ancestors);
    drop(retired);
    drop(frame);
    let completed = snapshot();
    assert_eq!(completed.frames.len(), 1);
    assert!(completed.spans.iter().all(|span| span.name != "retired"));
    let parent_id = if live_depth > 0 {
        Some(
            completed
                .spans
                .iter()
                .find(|span| span.path == parent_path)
                .expect("the live deepest parent must complete in this capture")
                .id,
        )
    } else {
        None
    };
    for span in &leaf_snapshot.spans {
        assert_eq!(span.parent_id, parent_id);
        assert_eq!(span.frame_index, Some(completed.frames[0].frame_index));
    }
    let retention = &completed.recorder_retention[0].spans;
    assert_eq!(
        retention.written,
        ((WARMUP_BATCHES + 1) * LEAVES_PER_SAMPLE + depth) as u64
    );
    reset_capture();
    Sample {
        wall_ns,
        requests,
        gross_bytes,
        leaf_path_bytes: leaf_snapshot.spans[0].path.len(),
        live_depth: leaf_snapshot.spans[0].depth,
    }
}

fn leaves() {
    for _ in 0..LEAVES_PER_SAMPLE {
        drop(black_box(ProfileScope::enter(
            "runtime",
            "profile.nested",
            "leaf",
        )));
    }
}

fn record_request(pointer: *mut u8, bytes: usize) {
    if pointer.is_null() {
        return;
    }
    let _ = MEASURING.try_with(|measuring| {
        if measuring.get() {
            REQUESTS.fetch_add(1, Ordering::Relaxed);
            GROSS_BYTES.fetch_add(bytes as u64, Ordering::Relaxed);
        }
    });
}

fn begin_profile() {
    MEASURING.with(|measuring| {
        assert!(!measuring.get(), "allocator windows must not overlap");
        REQUESTS.store(0, Ordering::Relaxed);
        GROSS_BYTES.store(0, Ordering::Relaxed);
        measuring.set(true);
    });
}

fn end_profile() -> (u64, u64) {
    MEASURING.with(|measuring| measuring.set(false));
    (
        REQUESTS.load(Ordering::Relaxed),
        GROSS_BYTES.load(Ordering::Relaxed),
    )
}

fn percentile(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}
