//! Requested Rust heap bytes; excludes driver and foreign allocations.
//! The host installs this wrapper once. Allocator callbacks only use atomics.

use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
struct Counters {
    allocations: AtomicUsize,
    live: AtomicUsize,
    peak: AtomicUsize,
}

impl Counters {
    fn allocated(&self, bytes: usize) {
        self.allocations.fetch_add(1, Ordering::Relaxed);
        let live = self.live.fetch_add(bytes, Ordering::Relaxed) + bytes;
        self.peak.fetch_max(live, Ordering::Relaxed);
    }

    fn deallocated(&self, bytes: usize) {
        self.live.fetch_sub(bytes, Ordering::Relaxed);
    }
}

static COUNTERS: Counters = Counters {
    allocations: AtomicUsize::new(0),
    live: AtomicUsize::new(0),
    peak: AtomicUsize::new(0),
};

/// Install at the process boundary to include allocations from all Rust crates.
pub struct CountingAllocator<A>(pub A);

// SAFETY: pointers and layouts are forwarded unchanged to the wrapped allocator.
// Failed allocations/reallocations leave accounting unchanged; callbacks never allocate.
unsafe impl<A: GlobalAlloc> GlobalAlloc for CountingAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { self.0.alloc(layout) };
        if !pointer.is_null() {
            COUNTERS.allocated(layout.size());
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { self.0.alloc_zeroed(layout) };
        if !pointer.is_null() {
            COUNTERS.allocated(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        COUNTERS.deallocated(layout.size());
        unsafe { self.0.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let replacement = unsafe { self.0.realloc(pointer, layout, new_size) };
        if !replacement.is_null() {
            COUNTERS.deallocated(layout.size());
            COUNTERS.allocated(new_size);
        }
        replacement
    }
}

/// Values are process-wide lifetime totals, sampled before recorder allocations.
pub(super) fn record_frame(stream: &'static str) {
    let live = COUNTERS.live.load(Ordering::Relaxed);
    let peak = COUNTERS.peak.load(Ordering::Relaxed);
    let allocations = COUNTERS.allocations.load(Ordering::Relaxed);
    super::record_counter_batch(
        stream,
        &[
            ("memory.live_bytes", live as f64),
            ("memory.peak_bytes", peak as f64),
            ("memory.allocations", allocations as f64),
        ],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_profile_accounting_retains_peak_after_free_and_resize() {
        let counters = Counters::default();
        counters.allocated(1024);
        counters.allocated(512);
        counters.deallocated(1024);
        counters.allocated(2048);
        counters.deallocated(512);
        counters.deallocated(2048);
        assert_eq!(counters.live.load(Ordering::Relaxed), 0);
        assert_eq!(counters.peak.load(Ordering::Relaxed), 2560);
        assert_eq!(counters.allocations.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn memory_profile_frame_exports_heap_counters() {
        let _guard = super::super::test_capture_lock();
        super::super::start_capture(super::super::ProfileCaptureConfig::default());
        {
            let _frame = super::super::ProfileFrameScope::enter("test", "heap");
        }
        super::super::stop_capture();
        let snapshot = super::super::snapshot();
        for name in [
            "memory.live_bytes",
            "memory.peak_bytes",
            "memory.allocations",
        ] {
            assert!(snapshot
                .counters
                .iter()
                .any(|counter| counter.name == name && counter.frame_index == Some(0)));
        }
        super::super::reset_capture();
    }
}
