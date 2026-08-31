use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

struct CountingAllocator;

#[global_allocator]
static COUNTING_ALLOCATOR: CountingAllocator = CountingAllocator;
static PROFILE_ACTIVE: AtomicBool = AtomicBool::new(false);
static ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static REQUESTED_BYTES: AtomicU64 = AtomicU64::new(0);
static LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
static PEAK_LIVE_BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() && PROFILE_ACTIVE.load(Ordering::Relaxed) {
            record_allocation(layout.size() as u64);
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() && PROFILE_ACTIVE.load(Ordering::Relaxed) {
            record_allocation(layout.size() as u64);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        if PROFILE_ACTIVE.load(Ordering::Relaxed) {
            decrease_live_bytes(layout.size() as u64);
        }
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let replacement = unsafe { System.realloc(pointer, layout, new_size) };
        if !replacement.is_null() && PROFILE_ACTIVE.load(Ordering::Relaxed) {
            ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
            REQUESTED_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
            let old_size = layout.size() as u64;
            let new_size = new_size as u64;
            let live_bytes = if new_size >= old_size {
                LIVE_BYTES
                    .fetch_add(new_size - old_size, Ordering::Relaxed)
                    .saturating_add(new_size - old_size)
            } else {
                decrease_live_bytes(old_size - new_size)
            };
            update_peak_live_bytes(live_bytes);
        }
        replacement
    }
}

fn record_allocation(size: u64) {
    ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
    REQUESTED_BYTES.fetch_add(size, Ordering::Relaxed);
    let live_bytes = LIVE_BYTES
        .fetch_add(size, Ordering::Relaxed)
        .saturating_add(size);
    update_peak_live_bytes(live_bytes);
}

fn decrease_live_bytes(size: u64) -> u64 {
    LIVE_BYTES
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            Some(current.saturating_sub(size))
        })
        .unwrap_or_default()
        .saturating_sub(size)
}

fn update_peak_live_bytes(live_bytes: u64) {
    let mut peak = PEAK_LIVE_BYTES.load(Ordering::Relaxed);
    while live_bytes > peak {
        match PEAK_LIVE_BYTES.compare_exchange_weak(
            peak,
            live_bytes,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(observed) => peak = observed,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AllocationSnapshot {
    pub(crate) allocation_count: u64,
    pub(crate) requested_bytes: u64,
    pub(crate) peak_live_bytes: u64,
}

pub(crate) fn begin_allocation_profile() {
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    LIVE_BYTES.store(0, Ordering::Relaxed);
    PEAK_LIVE_BYTES.store(0, Ordering::Relaxed);
    assert!(
        !PROFILE_ACTIVE.swap(true, Ordering::SeqCst),
        "allocation profiling must be single-threaded and non-overlapping"
    );
}

pub(crate) fn finish_allocation_profile() -> AllocationSnapshot {
    assert!(PROFILE_ACTIVE.swap(false, Ordering::SeqCst));
    AllocationSnapshot {
        allocation_count: ALLOCATION_COUNT.load(Ordering::Relaxed),
        requested_bytes: REQUESTED_BYTES.load(Ordering::Relaxed),
        peak_live_bytes: PEAK_LIVE_BYTES.load(Ordering::Relaxed),
    }
}
