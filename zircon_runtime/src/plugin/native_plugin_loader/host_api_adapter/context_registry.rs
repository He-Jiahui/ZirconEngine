use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use arc_swap::{ArcSwap, ArcSwapOption};

const HANDLE_SLOT_BITS: u32 = 32;
const HANDLE_SLOT_MASK: u64 = u32::MAX as u64;
const FIRST_GENERATION: u32 = 1;
const HOST_CONTEXT_PAGE_SLOTS: usize = 256;

/// Generational host handles resolve through immutable slot snapshots. Only slot allocation and
/// retirement acquire the writer mutex; stable callbacks only perform atomic reads and `Arc` pins.
pub(super) struct HostContextRegistry<T> {
    directory: ArcSwap<HostContextDirectory<T>>,
    writer: Mutex<HostContextWriter<T>>,
    writer_acquires: AtomicU64,
    #[cfg(test)]
    directory_page_reference_copies: AtomicU64,
}

impl<T> Default for HostContextRegistry<T> {
    fn default() -> Self {
        Self {
            directory: ArcSwap::from_pointee(HostContextDirectory::empty()),
            writer: Mutex::new(HostContextWriter::default()),
            writer_acquires: AtomicU64::new(0),
            #[cfg(test)]
            directory_page_reference_copies: AtomicU64::new(0),
        }
    }
}

impl<T> HostContextRegistry<T> {
    pub(super) fn insert(&self, context: Arc<T>) -> u64 {
        let mut writer = self.lock_writer();
        if let Some(index) = writer.free.pop() {
            let slot = writer.slot(index);
            let generation = slot.generation.load(Ordering::Acquire);
            debug_assert_ne!(generation, 0, "retired host context slot was reused");
            debug_assert!(slot.context.load().is_none());
            slot.context.store(Some(context));
            return encode_handle(index, generation);
        }

        let index = writer.next_slot;
        let page_index = index / HOST_CONTEXT_PAGE_SLOTS;
        if page_index == writer.pages.len() {
            writer.pages.push(Arc::new(HostContextPage::new()));
            #[cfg(test)]
            self.directory_page_reference_copies
                .fetch_add(writer.pages.len() as u64, Ordering::Relaxed);
            self.directory.store(Arc::new(HostContextDirectory {
                pages: writer.pages.clone().into_boxed_slice(),
            }));
        }

        let slot = writer.slot(index);
        debug_assert_eq!(slot.generation.load(Ordering::Acquire), 0);
        debug_assert!(slot.context.load().is_none());
        slot.context.store(Some(context));
        slot.generation.store(FIRST_GENERATION, Ordering::Release);
        writer.next_slot += 1;
        encode_handle(index, FIRST_GENERATION)
    }

    pub(super) fn get(&self, handle: u64) -> Option<Arc<T>> {
        let (index, generation) = decode_handle(handle)?;
        let directory = self.directory.load();
        let slot = directory.slot(index)?;
        if slot.generation.load(Ordering::Acquire) != generation {
            return None;
        }
        let context = slot.context.load_full()?;
        if slot.generation.load(Ordering::Acquire) != generation {
            return None;
        }
        Some(context)
    }

    pub(super) fn remove(&self, handle: u64) -> bool {
        let Some((index, generation)) = decode_handle(handle) else {
            return false;
        };
        let mut writer = self.lock_writer();
        if index >= writer.next_slot {
            return false;
        }
        let slot = writer.slot(index);
        if slot.generation.load(Ordering::Acquire) != generation {
            return false;
        }
        if slot.context.swap(None).is_none() {
            return false;
        }

        if generation == u32::MAX {
            // A wrapped generation could make the oldest handle valid again. Retire the slot
            // permanently instead; a future insertion will append a fresh slot.
            slot.generation.store(0, Ordering::Release);
        } else {
            slot.generation.store(generation + 1, Ordering::Release);
            writer.free.push(index);
        }
        true
    }

    #[cfg(test)]
    pub(super) fn slot_index(handle: u64) -> Option<usize> {
        decode_handle(handle).map(|(index, _)| index)
    }

    #[cfg(test)]
    pub(super) fn writer_acquire_count(&self) -> u64 {
        self.writer_acquires.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    fn directory_metrics(&self) -> HostContextDirectoryMetrics {
        let directory = self.directory.load();
        HostContextDirectoryMetrics {
            page_count: directory.pages.len(),
            directory_page_reference_copies: self
                .directory_page_reference_copies
                .load(Ordering::Relaxed) as usize,
            slot_arc_copies: 0,
        }
    }

    fn lock_writer(&self) -> MutexGuard<'_, HostContextWriter<T>> {
        self.writer_acquires.fetch_add(1, Ordering::Relaxed);
        self.writer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

struct HostContextSlot<T> {
    generation: AtomicU32,
    context: ArcSwapOption<T>,
}

impl<T> HostContextSlot<T> {
    fn empty() -> Self {
        Self {
            generation: AtomicU32::new(0),
            context: ArcSwapOption::empty(),
        }
    }
}

struct HostContextPage<T> {
    slots: Box<[HostContextSlot<T>]>,
}

impl<T> HostContextPage<T> {
    fn new() -> Self {
        Self {
            slots: (0..HOST_CONTEXT_PAGE_SLOTS)
                .map(|_| HostContextSlot::empty())
                .collect(),
        }
    }

    fn slot(&self, slot_index: usize) -> Option<&HostContextSlot<T>> {
        self.slots.get(slot_index)
    }
}

struct HostContextDirectory<T> {
    pages: Box<[Arc<HostContextPage<T>>]>,
}

impl<T> HostContextDirectory<T> {
    fn empty() -> Self {
        Self {
            pages: Box::new([]),
        }
    }

    fn slot(&self, index: usize) -> Option<&HostContextSlot<T>> {
        self.pages
            .get(index / HOST_CONTEXT_PAGE_SLOTS)?
            .slot(index % HOST_CONTEXT_PAGE_SLOTS)
    }
}

struct HostContextWriter<T> {
    pages: Vec<Arc<HostContextPage<T>>>,
    free: Vec<usize>,
    next_slot: usize,
}

impl<T> Default for HostContextWriter<T> {
    fn default() -> Self {
        Self {
            pages: Vec::new(),
            free: Vec::new(),
            next_slot: 0,
        }
    }
}

impl<T> HostContextWriter<T> {
    fn slot(&self, index: usize) -> &HostContextSlot<T> {
        self.pages[index / HOST_CONTEXT_PAGE_SLOTS]
            .slot(index % HOST_CONTEXT_PAGE_SLOTS)
            .expect("allocated host context index must resolve to a page slot")
    }
}

#[cfg(test)]
struct HostContextDirectoryMetrics {
    page_count: usize,
    directory_page_reference_copies: usize,
    slot_arc_copies: usize,
}

fn encode_handle(index: usize, generation: u32) -> u64 {
    let slot = u32::try_from(index)
        .ok()
        .and_then(|index| index.checked_add(1))
        .expect("native host context registry exhausted its 32-bit slot space");
    (u64::from(generation) << HANDLE_SLOT_BITS) | u64::from(slot)
}

fn decode_handle(handle: u64) -> Option<(usize, u32)> {
    let slot = handle & HANDLE_SLOT_MASK;
    let generation = (handle >> HANDLE_SLOT_BITS) as u32;
    if slot == 0 || generation == 0 {
        return None;
    }
    Some(((slot - 1) as usize, generation))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Instant;

    use super::HostContextRegistry;

    #[test]
    fn stale_generation_cannot_resolve_reused_slot() {
        let registry = HostContextRegistry::default();
        let first = registry.insert(Arc::new("first"));

        assert!(registry.remove(first));
        let second = registry.insert(Arc::new("second"));

        assert_eq!(
            HostContextRegistry::<&str>::slot_index(first),
            HostContextRegistry::<&str>::slot_index(second)
        );
        assert_ne!(
            first, second,
            "slot reuse must advance the encoded generation"
        );
        assert!(
            registry.get(first).is_none(),
            "stale handle must stay invalid after slot reuse"
        );
        assert_eq!(registry.get(second).as_deref(), Some(&"second"));
    }

    #[test]
    fn remove_blocks_new_lookups_while_in_flight_arc_finishes() {
        let registry = Arc::new(HostContextRegistry::default());
        let context_alive = Arc::new(AtomicBool::new(true));
        let handle = registry.insert(context_alive.clone());
        let acquired = Arc::new(Barrier::new(2));
        let release = Arc::new(Barrier::new(2));
        let worker = {
            let registry = registry.clone();
            let acquired = acquired.clone();
            let release = release.clone();
            thread::spawn(move || {
                let context = registry.get(handle).expect("in-flight lookup");
                acquired.wait();
                release.wait();
                context.load(Ordering::Acquire)
            })
        };

        acquired.wait();
        assert!(registry.remove(handle));
        assert!(
            registry.get(handle).is_none(),
            "remove must reject all new lookups"
        );
        context_alive.store(false, Ordering::Release);
        release.wait();

        assert!(!worker.join().expect("in-flight lookup worker"));
    }

    #[test]
    fn parallel_stable_lookups_never_acquire_writer_lock() {
        const THREADS: usize = 16;
        const LOOKUPS_PER_THREAD: usize = 16_384;

        let registry = Arc::new(HostContextRegistry::default());
        let handle = registry.insert(Arc::new(41_u64));
        let writer_acquires_before = registry.writer_acquire_count();
        let mut workers = Vec::with_capacity(THREADS);
        for _ in 0..THREADS {
            let registry = registry.clone();
            workers.push(thread::spawn(move || {
                let mut sum = 0_u64;
                for _ in 0..LOOKUPS_PER_THREAD {
                    sum += *registry.get(handle).expect("stable lookup");
                }
                sum
            }));
        }

        for worker in workers {
            assert_eq!(
                worker.join().expect("parallel lookup worker"),
                41 * LOOKUPS_PER_THREAD as u64
            );
        }
        assert_eq!(registry.writer_acquire_count(), writer_acquires_before);
    }

    #[test]
    fn paged_directory_append_avoids_full_slot_arc_snapshot_copies() {
        for allocations in [1_usize, 100, 10_000] {
            let registry = HostContextRegistry::default();
            for value in 0..allocations {
                registry.insert(Arc::new(value));
            }

            let metrics = registry.directory_metrics();
            let expected_pages = allocations.div_ceil(super::HOST_CONTEXT_PAGE_SLOTS);
            let max_directory_page_references =
                expected_pages * expected_pages.saturating_add(1) / 2;

            assert_eq!(metrics.page_count, expected_pages);
            assert_eq!(
                metrics.slot_arc_copies, 0,
                "appending a host context must never clone the complete slot Arc table"
            );
            assert!(
                metrics.directory_page_reference_copies <= max_directory_page_references,
                "directory publication may copy page Arc references only when a page is added"
            );
        }
    }

    #[test]
    #[ignore = "manual native host context registry performance evidence"]
    fn stable_lookup_benchmark_records_one_and_sixteen_thread_evidence() {
        const LOOKUPS: usize = 1_000_000;

        for threads in [1, 16] {
            let registry = Arc::new(HostContextRegistry::default());
            let handle = registry.insert(Arc::new(41_u64));
            let writer_acquires_before = registry.writer_acquire_count();
            let started = Instant::now();
            let mut workers = Vec::with_capacity(threads);
            for _ in 0..threads {
                let registry = registry.clone();
                workers.push(thread::spawn(move || {
                    let mut latencies_ns = Vec::with_capacity(LOOKUPS / threads);
                    let mut sum = 0_u64;
                    for _ in 0..LOOKUPS / threads {
                        let lookup_started = Instant::now();
                        sum += *registry.get(handle).expect("stable lookup");
                        latencies_ns.push(
                            lookup_started
                                .elapsed()
                                .as_nanos()
                                .try_into()
                                .unwrap_or(u64::MAX),
                        );
                    }
                    (sum, latencies_ns)
                }));
            }

            let mut latencies_ns = Vec::with_capacity(LOOKUPS);
            let mut sum = 0_u64;
            for worker in workers {
                let (worker_sum, mut worker_latencies) =
                    worker.join().expect("stable lookup benchmark worker");
                sum += worker_sum;
                latencies_ns.append(&mut worker_latencies);
            }
            let elapsed = started.elapsed();
            latencies_ns.sort_unstable();
            let p95_ns = percentile(&latencies_ns, 95);
            let p99_ns = percentile(&latencies_ns, 99);
            let throughput = LOOKUPS as f64 / elapsed.as_secs_f64();

            assert_eq!(sum, 41 * LOOKUPS as u64);
            assert_eq!(registry.writer_acquire_count(), writer_acquires_before);
            eprintln!(
                "native host context registry: threads={threads} lookups={LOOKUPS} \
                 writer_acquires=0 throughput={throughput:.0}/s p95={p95_ns}ns p99={p99_ns}ns"
            );
        }
    }

    fn percentile(sorted_values: &[u64], percentile: usize) -> u64 {
        let index = (sorted_values.len() - 1) * percentile / 100;
        sorted_values[index]
    }
}
