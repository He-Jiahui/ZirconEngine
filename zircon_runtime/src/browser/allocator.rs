use serde::Serialize;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

struct BrowserAllocator;
#[global_allocator]
static ALLOCATOR: BrowserAllocator = BrowserAllocator;
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static LIVE: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);
fn added(bytes: usize) {
    ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    let live = LIVE.fetch_add(bytes as u64, Ordering::Relaxed) + (bytes as u64);
    PEAK.fetch_max(live, Ordering::Relaxed);
}
// SAFETY: System receives the caller's unchanged pointer, layout and size.
// The counters never allocate and do not change allocation ownership or error handling.
unsafe impl GlobalAlloc for BrowserAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: GlobalAlloc supplies a valid layout; System owns the returned allocation.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            added(layout.size());
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Preserve the layout and zero-initialization contract of System.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            added(layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: Forward the live System allocation with its original layout.
        unsafe { System.dealloc(pointer, layout) };
        LIVE.fetch_sub(layout.size() as u64, Ordering::Relaxed);
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: Forward the live allocation, original layout and requested new size unchanged.
        let replacement = unsafe { System.realloc(pointer, layout, size) };
        if !replacement.is_null() {
            LIVE.fetch_sub(layout.size() as u64, Ordering::Relaxed);
            added(size);
        }
        replacement
    }
}
#[derive(Debug, Serialize)]
pub struct BrowserHeapSnapshot {
    pub allocation_calls: u64,
    pub live_requested_bytes: u64,
    pub peak_requested_bytes: u64,
}
/// Requested Rust heap bytes; excludes JS, allocator overhead and GPU residency.
pub fn browser_heap_snapshot() -> BrowserHeapSnapshot {
    BrowserHeapSnapshot {
        allocation_calls: ALLOCATIONS.load(Ordering::Relaxed),
        live_requested_bytes: LIVE.load(Ordering::Relaxed),
        peak_requested_bytes: PEAK.load(Ordering::Relaxed),
    }
}
