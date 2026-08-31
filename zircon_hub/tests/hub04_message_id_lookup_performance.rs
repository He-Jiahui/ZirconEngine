use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Instant,
};

use zircon_hub::state::HubMessageId;

const LOOKUPS: usize = 8_192;
const SAMPLE_PAIRS: usize = 21;
const THRESHOLD_PERCENT: u128 = 60;

struct CountingAllocator;

static TRACK_ALLOCATIONS: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static COUNTING_ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record_allocation(size);
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[derive(Clone, Copy)]
struct Measurement {
    elapsed_ns: u128,
    allocations: usize,
    allocated_bytes: usize,
    checksum: usize,
}

#[test]
#[ignore = "managed release performance contract"]
fn hub04_message_id_lookup_release_benchmark_evidence() {
    let ids = HubMessageId::all()
        .into_iter()
        .map(HubMessageId::as_str)
        .collect::<Vec<_>>();
    for id in &ids {
        assert_eq!(legacy_from_str_id(id), HubMessageId::from_str_id(id));
    }
    for id in ["unknown.id", "shell", ".missing"] {
        assert_eq!(legacy_from_str_id(id), HubMessageId::from_str_id(id));
    }

    for _ in 0..4 {
        black_box(measure(&ids, legacy_from_str_id));
        black_box(measure(&ids, HubMessageId::from_str_id));
    }

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&ids, legacy_from_str_id));
            optimized.push(measure(&ids, HubMessageId::from_str_id));
        } else {
            optimized.push(measure(&ids, HubMessageId::from_str_id));
            legacy.push(measure(&ids, legacy_from_str_id));
        }
    }

    let checksum = legacy[0].checksum;
    assert!(legacy.iter().all(|sample| sample.checksum == checksum));
    assert!(optimized.iter().all(|sample| sample.checksum == checksum));

    let legacy_allocations = legacy[0].allocations;
    let optimized_allocations = optimized[0].allocations;
    assert!(legacy_allocations >= LOOKUPS);
    assert!(legacy
        .iter()
        .all(|sample| sample.allocations == legacy_allocations));
    assert_eq!(optimized_allocations, 0);
    assert!(optimized.iter().all(|sample| sample.allocations == 0));
    assert!(optimized.iter().all(|sample| sample.allocated_bytes == 0));

    let legacy_ns = elapsed_samples(&legacy);
    let optimized_ns = elapsed_samples(&optimized);
    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    let p50_reduction_percent = reduction_percent(legacy_p50_ns, optimized_p50_ns);
    let p95_reduction_percent = reduction_percent(legacy_p95_ns, optimized_p95_ns);

    println!(
        "PERF_RESULT hub04_message_id_lookup lookups=8192 sample_pairs=21 \
         threshold_percent=60 checksum={checksum} \
         legacy_allocations={legacy_allocations} optimized_allocations={optimized_allocations} \
         legacy_allocated_bytes={} optimized_allocated_bytes={} allocation_reduction_percent=100 \
         legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
         p50_reduction_percent={p50_reduction_percent} \
         legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
         p95_reduction_percent={p95_reduction_percent} \
         legacy_raw_ns={} optimized_raw_ns={}",
        legacy[0].allocated_bytes,
        optimized[0].allocated_bytes,
        raw_samples(&legacy_ns),
        raw_samples(&optimized_ns),
    );

    assert!(
        p50_reduction_percent >= THRESHOLD_PERCENT,
        "optimized lookup must improve P50 by at least {THRESHOLD_PERCENT}%: \
         legacy={legacy_p50_ns}ns optimized={optimized_p50_ns}ns"
    );
    assert!(
        p95_reduction_percent >= THRESHOLD_PERCENT,
        "optimized lookup must improve P95 by at least {THRESHOLD_PERCENT}%: \
         legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_from_str_id(id: &str) -> Option<HubMessageId> {
    HubMessageId::all()
        .into_iter()
        .find(|candidate| candidate.as_str() == id)
}

fn measure(ids: &[&str], lookup: fn(&str) -> Option<HubMessageId>) -> Measurement {
    reset_allocation_counters();
    TRACK_ALLOCATIONS.store(true, Ordering::Relaxed);
    let started = Instant::now();
    let mut checksum = 0usize;
    for index in 0..LOOKUPS {
        let id = ids[index % ids.len()];
        let result = lookup(black_box(id));
        checksum = checksum.wrapping_add(match black_box(result) {
            Some(message_id) => message_id.as_str().len().wrapping_add(index),
            None => index,
        });
    }
    let elapsed_ns = started.elapsed().as_nanos().max(1);
    TRACK_ALLOCATIONS.store(false, Ordering::Relaxed);

    Measurement {
        elapsed_ns,
        allocations: ALLOCATIONS.load(Ordering::Relaxed),
        allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
        checksum,
    }
}

fn record_allocation(size: usize) {
    if TRACK_ALLOCATIONS.load(Ordering::Relaxed) {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(size, Ordering::Relaxed);
    }
}

fn reset_allocation_counters() {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
}

fn elapsed_samples(measurements: &[Measurement]) -> Vec<u128> {
    measurements
        .iter()
        .map(|measurement| measurement.elapsed_ns)
        .collect()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn reduction_percent(legacy: u128, optimized: u128) -> u128 {
    legacy.saturating_sub(optimized).saturating_mul(100) / legacy.max(1)
}

fn raw_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
