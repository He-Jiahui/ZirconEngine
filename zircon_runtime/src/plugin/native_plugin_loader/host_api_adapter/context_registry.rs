use std::ops::Deref;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use arc_swap::{ArcSwap, ArcSwapOption};

use crate::plugin::FrozenBridgeTable;

use super::{
    NativeBridgeMethodFn, NativeHostApiV3RegistrationContext, NativeHostApiV4RegistrationContext,
    NativePluginLibraryGenerationOwner,
};

const HANDLE_SLOT_BITS: u32 = 32;
const HANDLE_SLOT_MASK: u64 = u32::MAX as u64;
const FIRST_GENERATION: u32 = 1;
const HOST_CONTEXT_PAGE_SLOTS: usize = 256;

pub(super) struct NativeHostBridgeCallContext {
    pub(super) table: FrozenBridgeTable,
    pub(super) methods: DenseBridgeMethodTable,
    pub(super) library_owner: Option<NativePluginLibraryGenerationOwner>,
}

/// Pins the single registry-owned context allocation for the complete callback dispatch.
pub(super) struct NativeHostBridgeCallContextPin {
    context: Arc<NativeHostApiV3Context>,
}

impl NativeHostBridgeCallContextPin {
    pub(super) fn new(context: Arc<NativeHostApiV3Context>) -> Self {
        debug_assert!(matches!(
            context.as_ref(),
            NativeHostApiV3Context::BridgeCall(_)
        ));
        Self { context }
    }
}

impl Deref for NativeHostBridgeCallContextPin {
    type Target = NativeHostBridgeCallContext;

    fn deref(&self) -> &Self::Target {
        match self.context.as_ref() {
            NativeHostApiV3Context::BridgeCall(context) => context,
            NativeHostApiV3Context::Registration(_) | NativeHostApiV3Context::RegistrationV4(_) => {
                unreachable!("bridge call context pin must retain a bridge context")
            }
        }
    }
}

/// Immutable callback dispatch storage indexed by the already-resolved interface and method
/// slots. The manifest parser may use ordered maps while building descriptors, but a stable ABI
/// call must not pay a tree lookup after its scope has been frozen.
pub(super) struct DenseBridgeMethodTable {
    interfaces: DenseBridgeSlotDirectory<DenseBridgeMethodRow>,
    method_count: usize,
}

impl DenseBridgeMethodTable {
    pub(super) fn from_entries(
        entries: impl IntoIterator<Item = (u32, u32, NativeBridgeMethodFn)>,
    ) -> Self {
        let mut interfaces = DenseBridgeSlotDirectory::default();
        let mut method_count = 0;

        for (interface_slot, method_slot, method) in entries {
            if interfaces.get(interface_slot).is_none() {
                interfaces.insert(interface_slot, DenseBridgeMethodRow::default());
            }
            let row = interfaces
                .get_mut(interface_slot)
                .expect("inserted bridge interface row must remain addressable");
            method_count += usize::from(row.insert(method_slot, method));
        }

        Self {
            interfaces,
            method_count,
        }
    }

    pub(super) fn get(
        &self,
        interface_slot: u32,
        method_slot: u32,
    ) -> Option<NativeBridgeMethodFn> {
        self.interfaces.get(interface_slot)?.get(method_slot)
    }

    pub(super) const fn len(&self) -> usize {
        self.method_count
    }

    #[cfg(test)]
    pub(super) fn metrics(&self) -> DenseBridgeMethodTableMetrics {
        DenseBridgeMethodTableMetrics {
            interface_rows: self.interfaces.len(),
            method_rows: self.interfaces.len(),
            occupied_methods: self.method_count,
            tree_probes: 0,
        }
    }
}

struct DenseBridgeMethodRow {
    methods: DenseBridgeSlotDirectory<NativeBridgeMethodFn>,
}

impl Default for DenseBridgeMethodRow {
    fn default() -> Self {
        Self {
            methods: DenseBridgeSlotDirectory::default(),
        }
    }
}

impl DenseBridgeMethodRow {
    fn insert(&mut self, method_slot: u32, method: NativeBridgeMethodFn) -> bool {
        self.methods.insert(method_slot, method).is_none()
    }

    fn get(&self, method_slot: u32) -> Option<NativeBridgeMethodFn> {
        self.methods.get(method_slot).copied()
    }
}

// ABI slots are arbitrary u32 values. A fixed four-level directory shares the common prefixes
// of dense registrations while keeping a sparse slot from turning into a slot-sized allocation.
const DENSE_BRIDGE_SLOT_DIRECTORY_BITS: u32 = 8;
const DENSE_BRIDGE_SLOT_DIRECTORY_LEVELS: u32 = u32::BITS / DENSE_BRIDGE_SLOT_DIRECTORY_BITS;
const DENSE_BRIDGE_SLOT_DIRECTORY_FANOUT: usize = 1_usize << DENSE_BRIDGE_SLOT_DIRECTORY_BITS;

struct DenseBridgeSlotDirectory<T> {
    root: DenseBridgeSlotDirectoryNode<T>,
    len: usize,
}

impl<T> Default for DenseBridgeSlotDirectory<T> {
    fn default() -> Self {
        Self {
            root: DenseBridgeSlotDirectoryNode::empty_at_depth(0),
            len: 0,
        }
    }
}

impl<T> DenseBridgeSlotDirectory<T> {
    fn get(&self, slot: u32) -> Option<&T> {
        self.root.get(slot, 0)
    }

    fn get_mut(&mut self, slot: u32) -> Option<&mut T> {
        self.root.get_mut(slot, 0)
    }

    fn insert(&mut self, slot: u32, value: T) -> Option<T> {
        let previous = self.root.insert(slot, 0, value);
        if previous.is_none() {
            self.len += 1;
        }
        previous
    }

    const fn len(&self) -> usize {
        self.len
    }
}

enum DenseBridgeSlotDirectoryNode<T> {
    Branch(Box<[Option<Box<DenseBridgeSlotDirectoryNode<T>>>; DENSE_BRIDGE_SLOT_DIRECTORY_FANOUT]>),
    Page(Box<[Option<T>; DENSE_BRIDGE_SLOT_DIRECTORY_FANOUT]>),
}

impl<T> DenseBridgeSlotDirectoryNode<T> {
    fn empty_at_depth(depth: u32) -> Self {
        if depth + 1 == DENSE_BRIDGE_SLOT_DIRECTORY_LEVELS {
            Self::Page(Box::new(std::array::from_fn(|_| None)))
        } else {
            Self::Branch(Box::new(std::array::from_fn(|_| None)))
        }
    }

    fn get(&self, slot: u32, depth: u32) -> Option<&T> {
        let index = dense_bridge_slot_directory_index(slot, depth);
        match self {
            Self::Branch(branches) => branches.get(index)?.as_deref()?.get(slot, depth + 1),
            Self::Page(values) => values.get(index)?.as_ref(),
        }
    }

    fn get_mut(&mut self, slot: u32, depth: u32) -> Option<&mut T> {
        let index = dense_bridge_slot_directory_index(slot, depth);
        match self {
            Self::Branch(branches) => branches
                .get_mut(index)?
                .as_deref_mut()?
                .get_mut(slot, depth + 1),
            Self::Page(values) => values.get_mut(index)?.as_mut(),
        }
    }

    fn insert(&mut self, slot: u32, depth: u32, value: T) -> Option<T> {
        let index = dense_bridge_slot_directory_index(slot, depth);
        match self {
            Self::Branch(branches) => branches[index]
                .get_or_insert_with(|| Box::new(Self::empty_at_depth(depth + 1)))
                .insert(slot, depth + 1, value),
            Self::Page(values) => values[index].replace(value),
        }
    }
}

fn dense_bridge_slot_directory_index(slot: u32, depth: u32) -> usize {
    debug_assert!(depth < DENSE_BRIDGE_SLOT_DIRECTORY_LEVELS);
    let shift = (DENSE_BRIDGE_SLOT_DIRECTORY_LEVELS - depth - 1) * DENSE_BRIDGE_SLOT_DIRECTORY_BITS;
    ((slot >> shift) & (DENSE_BRIDGE_SLOT_DIRECTORY_FANOUT as u32 - 1)) as usize
}

impl NativeHostBridgeCallContext {
    #[cfg(test)]
    pub(super) fn method_table_metrics(&self) -> DenseBridgeMethodTableMetrics {
        self.methods.metrics()
    }
}

#[cfg(test)]
pub(super) struct DenseBridgeMethodTableMetrics {
    pub(super) interface_rows: usize,
    pub(super) method_rows: usize,
    pub(super) occupied_methods: usize,
    pub(super) tree_probes: usize,
}

pub(super) enum NativeHostApiV3Context {
    Registration(NativeHostApiV3RegistrationContext),
    RegistrationV4(NativeHostApiV4RegistrationContext),
    BridgeCall(NativeHostBridgeCallContext),
}

/// Coordinates registration-context closeout with ABI entries that already resolved a handle.
pub(super) struct NativeHostRegistrationScopeState {
    closing: AtomicBool,
    active_pins: AtomicUsize,
    drain_lock: Mutex<()>,
    drained: Condvar,
}

impl Default for NativeHostRegistrationScopeState {
    fn default() -> Self {
        Self {
            closing: AtomicBool::new(false),
            active_pins: AtomicUsize::new(0),
            drain_lock: Mutex::new(()),
            drained: Condvar::new(),
        }
    }
}

impl NativeHostRegistrationScopeState {
    pub(super) fn acquire(self: &Arc<Self>) -> Option<NativeHostRegistrationLease> {
        if self.closing.load(Ordering::Acquire) {
            return None;
        }

        self.active_pins.fetch_add(1, Ordering::AcqRel);
        if !self.closing.load(Ordering::Acquire) {
            return Some(NativeHostRegistrationLease {
                state: Arc::clone(self),
            });
        }

        self.release_pin();
        None
    }

    pub(super) fn close_and_wait(&self) {
        self.closing.store(true, Ordering::Release);
        let mut guard = self
            .drain_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while self.active_pins.load(Ordering::Acquire) != 0 {
            guard = self
                .drained
                .wait(guard)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn release_pin(&self) {
        // The close path checks the counter while holding this lock, then atomically releases it
        // as it waits. Releasing under the same lock prevents a final-pin notification from being
        // lost between that check and the condition-variable wait.
        let _guard = self
            .drain_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if self.active_pins.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.drained.notify_all();
        }
    }

    #[cfg(test)]
    pub(super) fn is_closing(&self) -> bool {
        self.closing.load(Ordering::Acquire)
    }
}

pub(super) struct NativeHostRegistrationLease {
    state: Arc<NativeHostRegistrationScopeState>,
}

impl Drop for NativeHostRegistrationLease {
    fn drop(&mut self) {
        self.state.release_pin();
    }
}

/// Pins a V3 registration context until the ABI entry has completed its registry mutation.
pub(super) struct NativeHostApiV3RegistrationContextPin {
    context: NativeHostApiV3RegistrationContext,
    _lease: NativeHostRegistrationLease,
}

impl NativeHostApiV3RegistrationContextPin {
    pub(super) fn new(context: NativeHostApiV3RegistrationContext) -> Option<Self> {
        let lease = context.lifetime.acquire()?;
        Some(Self {
            context,
            _lease: lease,
        })
    }

    #[cfg(test)]
    pub(super) fn is_closing(&self) -> bool {
        self.context.lifetime.is_closing()
    }
}

impl Deref for NativeHostApiV3RegistrationContextPin {
    type Target = NativeHostApiV3RegistrationContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

/// Pins a V4 registration context until the ABI entry has completed its registry mutation.
pub(super) struct NativeHostApiV4RegistrationContextPin {
    context: NativeHostApiV4RegistrationContext,
    _lease: NativeHostRegistrationLease,
}

impl NativeHostApiV4RegistrationContextPin {
    pub(super) fn new(context: NativeHostApiV4RegistrationContext) -> Option<Self> {
        let lease = context.lifetime.acquire()?;
        Some(Self {
            context,
            _lease: lease,
        })
    }

    #[cfg(test)]
    pub(super) fn is_closing(&self) -> bool {
        self.context.lifetime.is_closing()
    }
}

impl Deref for NativeHostApiV4RegistrationContextPin {
    type Target = NativeHostApiV4RegistrationContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

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

    struct DropProbe {
        dropped: Arc<AtomicBool>,
    }

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::Release);
        }
    }

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
        let context_dropped = Arc::new(AtomicBool::new(false));
        let handle = registry.insert(Arc::new(DropProbe {
            dropped: context_dropped.clone(),
        }));
        let acquired = Arc::new(Barrier::new(2));
        let release = Arc::new(Barrier::new(2));
        let worker = {
            let registry = registry.clone();
            let acquired = acquired.clone();
            let release = release.clone();
            thread::spawn(move || {
                let _context = registry.get(handle).expect("in-flight lookup");
                acquired.wait();
                release.wait();
            })
        };

        acquired.wait();
        assert!(registry.remove(handle));
        assert!(
            registry.get(handle).is_none(),
            "remove must reject all new lookups"
        );
        assert!(
            !context_dropped.load(Ordering::Acquire),
            "the in-flight lookup Arc must retain the removed context"
        );
        release.wait();

        worker.join().expect("in-flight lookup worker");
        assert!(
            context_dropped.load(Ordering::Acquire),
            "the context should release after the final in-flight lookup Arc exits"
        );
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
