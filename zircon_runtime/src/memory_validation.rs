//! Rust allocator requested-byte probes, run alone with `--test-threads=1`.
//! System and driver allocations outside Rust's allocator are excluded.
use std::sync::atomic::{AtomicUsize, Ordering};
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
pub(crate) fn allocated(bytes: usize) {
    ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    let live = LIVE.fetch_add(bytes, Ordering::Relaxed) + bytes;
    PEAK.fetch_max(live, Ordering::Relaxed);
}
pub(crate) fn deallocated(bytes: usize) {
    LIVE.fetch_sub(bytes, Ordering::Relaxed);
}
pub(crate) fn reallocated(old: usize, new: usize) {
    deallocated(old);
    allocated(new);
}
#[derive(Debug)]
pub(crate) struct Measurement {
    pub allocations: usize,
    pub live_bytes: isize,
    pub peak_bytes: usize,
}
pub(crate) fn measure<T>(operation: impl FnOnce() -> T) -> (T, Measurement) {
    let live = LIVE.load(Ordering::Relaxed);
    let allocations = ALLOCATIONS.load(Ordering::Relaxed);
    PEAK.store(live, Ordering::Relaxed);
    let value = operation();
    let result = Measurement {
        allocations: ALLOCATIONS
            .load(Ordering::Relaxed)
            .saturating_sub(allocations),
        live_bytes: LIVE.load(Ordering::Relaxed) as isize - live as isize,
        peak_bytes: PEAK.load(Ordering::Relaxed).saturating_sub(live),
    };
    (value, result)
}
pub(crate) fn live_bytes() -> usize {
    let _ = std::io::stdout();
    LIVE.load(Ordering::Relaxed)
}
impl Measurement {
    pub(crate) fn report(&self, scenario: &str, implementation: &str) {
        println!("MEMORY_PROFILE {{\"scenario\":\"{scenario}\",\"implementation\":\"{implementation}\",\"scope\":\"Rust allocator requested bytes\",\"allocations\":{},\"live_bytes\":{},\"peak_bytes\":{}}}", self.allocations, self.live_bytes, self.peak_bytes);
    }
}
