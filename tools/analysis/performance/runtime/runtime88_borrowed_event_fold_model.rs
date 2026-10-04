use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

const EVENT_COUNT: usize = 16_384;
const SAMPLE_COUNT: usize = 31;
const WARMUP_COUNT: usize = 5;

struct CountingAllocator;

static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record_allocation();
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}

fn record_allocation() {
    if COUNTING.load(Ordering::Relaxed) {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AssetUriModel {
    path: String,
    label: Option<String>,
}

fn main() {
    let uri = AssetUriModel {
        path: "res://materials/long_shared_prefix/grid.zmaterial".to_string(),
        label: Some("primary-surface".to_string()),
    };
    let events = vec![uri; EVENT_COUNT];

    assert_eq!(legacy_fold(&events), optimized_fold(&events));
    let legacy_allocations = allocation_count(|| legacy_fold(&events));
    let optimized_allocations = allocation_count(|| optimized_fold(&events));
    assert_eq!(legacy_allocations, EVENT_COUNT * 2 + 1);
    assert_eq!(optimized_allocations, 3);
    assert!(optimized_allocations * 100 <= legacy_allocations);

    for _ in 0..WARMUP_COUNT {
        black_box(legacy_fold(black_box(&events)));
        black_box(optimized_fold(black_box(&events)));
    }

    let (legacy_ns, optimized_ns) = paired_samples(
        || checksum(&legacy_fold(black_box(&events))),
        || checksum(&optimized_fold(black_box(&events))),
    );
    let expected_checksum = checksum(&optimized_fold(&events));
    let legacy_p50 = percentile(&legacy_ns, 50);
    let legacy_p95 = percentile(&legacy_ns, 95);
    let optimized_p50 = percentile(&optimized_ns, 50);
    let optimized_p95 = percentile(&optimized_ns, 95);
    assert!(optimized_p50.saturating_mul(2) <= legacy_p50);
    assert!(optimized_p95.saturating_mul(2) <= legacy_p95);

    println!(
        "RUNTIME88_BORROWED_EVENT_FOLD_MODEL_V1 events={EVENT_COUNT} samples={SAMPLE_COUNT} legacy_allocations={legacy_allocations} optimized_allocations={optimized_allocations} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} checksum={expected_checksum} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns)
    );
}

fn legacy_fold(events: &[AssetUriModel]) -> Vec<AssetUriModel> {
    let mut result = Vec::with_capacity(1);
    for event in events.iter().cloned() {
        if result.is_empty() {
            result.push(event);
        } else {
            result[0] = event;
        }
    }
    result
}

fn optimized_fold(events: &[AssetUriModel]) -> Vec<AssetUriModel> {
    let mut result = Vec::with_capacity(1);
    for event in events {
        if result.is_empty() {
            result.push(event.clone());
        }
    }
    result
}

fn allocation_count(operation: impl FnOnce() -> Vec<AssetUriModel>) -> usize {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::Relaxed);
    let result = black_box(operation());
    COUNTING.store(false, Ordering::Relaxed);
    let allocations = ALLOCATIONS.load(Ordering::Relaxed);
    black_box(result);
    allocations
}

fn paired_samples(
    mut legacy: impl FnMut() -> usize,
    mut optimized: impl FnMut() -> usize,
) -> (Vec<u128>, Vec<u128>) {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_COUNT);
    for index in 0..SAMPLE_COUNT {
        if index % 2 == 0 {
            legacy_ns.push(measure(&mut legacy));
            optimized_ns.push(measure(&mut optimized));
        } else {
            optimized_ns.push(measure(&mut optimized));
            legacy_ns.push(measure(&mut legacy));
        }
    }
    (legacy_ns, optimized_ns)
}

fn measure(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn checksum(values: &[AssetUriModel]) -> usize {
    values
        .iter()
        .map(|uri| uri.path.len() + uri.label.as_deref().map(str::len).unwrap_or(0))
        .sum()
}

fn percentile(values: &[u128], percentile: usize) -> u128 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100) - 1]
}

fn samples(values: &[u128]) -> String {
    values
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
