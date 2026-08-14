use std::ops::Deref;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use arc_swap::{ArcSwap, ArcSwapOption};

use super::super::bridge_scope::NativeHostBridgeCallContext;
use super::super::ecs_registration::NativeHostApiV3RegistrationContext;
use super::super::registration_policy::NativeHostApiV4RegistrationContext;

const HANDLE_SLOT_BITS: u32 = 32;
const HANDLE_SLOT_MASK: u64 = u32::MAX as u64;
const FIRST_GENERATION: u32 = 1;
pub(in super::super) const HOST_CONTEXT_PAGE_SLOTS: usize = 256;

/// The shared raw-handle namespace prevents a registration handle from being treated as a bridge
/// handle after a slot is reused.
pub(in super::super) enum NativeHostApiV3Context {
    Registration(NativeHostApiV3RegistrationContext),
    RegistrationV4(NativeHostApiV4RegistrationContext),
    BridgeCall(NativeHostBridgeCallContext),
}

/// Coordinates registration-context closeout with ABI entries that already resolved a handle.
pub(in super::super) struct NativeHostRegistrationScopeState {
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

    pub(in super::super) fn close_and_wait(&self) {
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

struct NativeHostRegistrationLease {
    state: Arc<NativeHostRegistrationScopeState>,
}

impl Drop for NativeHostRegistrationLease {
    fn drop(&mut self) {
        self.state.release_pin();
    }
}

/// Pins a V3 registration context until the ABI entry has completed its registry mutation.
pub(in super::super) struct NativeHostApiV3RegistrationContextPin {
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
pub(in super::super) struct NativeHostApiV4RegistrationContextPin {
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
pub(in super::super) struct HostContextRegistry<T> {
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
    pub(super) fn directory_metrics(&self) -> HostContextDirectoryMetrics {
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
pub(in super::super) struct HostContextDirectoryMetrics {
    pub(super) page_count: usize,
    pub(super) directory_page_reference_copies: usize,
    pub(super) slot_arc_copies: usize,
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
