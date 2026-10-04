//! Isolated integration-binary allocator, using the existing zr_resource profiling pattern.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};

struct CountingAllocator;
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
static ACTIVE: AtomicBool = AtomicBool::new(false);
static COUNT: AtomicU64 = AtomicU64::new(0);
static REQUESTED: AtomicU64 = AtomicU64::new(0);
static NET_LIVE: AtomicI64 = AtomicI64::new(0);
static PEAK: AtomicI64 = AtomicI64::new(0);

// SAFETY: Every operation delegates the unchanged pointer/Layout contract to System.
// Counter updates use atomics without allocation. This executable has no second allocator.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The caller supplies the valid allocation Layout required by GlobalAlloc.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() && ACTIVE.load(Ordering::Relaxed) {
            record(layout.size() as u64);
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: System receives the original valid Layout and owns zero initialization.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() && ACTIVE.load(Ordering::Relaxed) {
            record(layout.size() as u64);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        if ACTIVE.load(Ordering::Relaxed) {
            decrease(layout.size() as u64);
        }
        // SAFETY: System owns the pointer with this Layout, as required by GlobalAlloc.
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: GlobalAlloc guarantees the System-owned pointer, old Layout and new size.
        let replacement = unsafe { System.realloc(pointer, layout, new_size) };
        if !replacement.is_null() && ACTIVE.load(Ordering::Relaxed) {
            COUNT.fetch_add(1, Ordering::Relaxed);
            REQUESTED.fetch_add(new_size as u64, Ordering::Relaxed);
            // A successful realloc changes live requested bytes by new minus old,
            // including pointers allocated before the measurement window.
            adjust_live(new_size as i64 - layout.size() as i64);
        }
        replacement
    }
}

fn record(size: u64) {
    COUNT.fetch_add(1, Ordering::Relaxed);
    REQUESTED.fetch_add(size, Ordering::Relaxed);
    adjust_live(size as i64);
}

fn decrease(size: u64) {
    adjust_live(-(size as i64));
}

fn adjust_live(delta: i64) {
    // Layout sizes fit isize. Never discard a deficit from a pre-window free:
    // this is net process requested-live change, not query-owned retained bytes.
    let live = NET_LIVE.fetch_add(delta, Ordering::Relaxed) + delta;
    PEAK.fetch_max(live, Ordering::Relaxed);
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Snapshot {
    pub(super) allocations: u64,
    pub(super) requested_bytes: u64,
    pub(super) net_live_byte_delta: i64,
    pub(super) peak_net_live_bytes_above_window_start: i64,
}

struct Window;
impl Drop for Window {
    fn drop(&mut self) {
        ACTIVE.store(false, Ordering::SeqCst);
    }
}

pub(super) fn measure<T>(operation: impl FnOnce() -> T) -> (T, Snapshot) {
    assert!(
        !ACTIVE.load(Ordering::SeqCst),
        "allocation windows must not overlap"
    );
    COUNT.store(0, Ordering::Relaxed);
    REQUESTED.store(0, Ordering::Relaxed);
    NET_LIVE.store(0, Ordering::Relaxed);
    PEAK.store(0, Ordering::Relaxed);
    assert!(!ACTIVE.swap(true, Ordering::SeqCst));
    let window = Window;
    let output = operation();
    drop(window);
    (
        output,
        Snapshot {
            allocations: COUNT.load(Ordering::Relaxed),
            requested_bytes: REQUESTED.load(Ordering::Relaxed),
            net_live_byte_delta: NET_LIVE.load(Ordering::Relaxed),
            peak_net_live_bytes_above_window_start: PEAK.load(Ordering::Relaxed),
        },
    )
}

// Actual allocator regression probe, executed by the isolated ignored profile.
// A pre-window 8 KiB free must remain a deficit when a new 4 KiB allocation follows.
// Single selected profile and --test-threads=1 are required because scope is process-wide.
pub(super) fn verify_preexisting_free_window() -> Snapshot {
    let old_layout = Layout::from_size_align(8_192, 8).unwrap();
    let new_layout = Layout::from_size_align(4_096, 8).unwrap();
    // SAFETY: Both layouts are valid; each non-null allocation is deallocated once.
    let old_pointer = unsafe { std::alloc::alloc(old_layout) };
    assert!(!old_pointer.is_null());
    let old_pointer = std::hint::black_box(old_pointer);
    let (new_pointer, snapshot) = measure(|| {
        // SAFETY: The pointer was allocated above with old_layout.
        unsafe { std::alloc::dealloc(old_pointer, old_layout) };
        // SAFETY: new_layout is valid; the returned pointer is retained past the window.
        std::hint::black_box(unsafe { std::alloc::alloc(new_layout) })
    });
    assert!(!new_pointer.is_null());
    // SAFETY: The successful allocation uses new_layout and is no longer sampled.
    unsafe { std::alloc::dealloc(new_pointer, new_layout) };
    assert_eq!(snapshot.allocations, 1);
    assert_eq!(snapshot.requested_bytes, 4_096);
    assert_eq!(snapshot.net_live_byte_delta, -4_096);
    assert_eq!(snapshot.peak_net_live_bytes_above_window_start, 0);
    snapshot
}
