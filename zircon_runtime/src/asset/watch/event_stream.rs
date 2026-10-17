//! Bounded asset notifications: one payload per publication and a cursor per subscription.
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock, Weak};
use std::time::{Duration, Instant};

use super::{AssetChange, AssetWatchError};
use crate::asset::pipeline::manager::ProjectAssetManagementGeneration;
use crate::core::framework::channel::ChannelWakeCallback;

pub(crate) const CHANGE_LOG_LIMITS: (usize, usize) = (4096, 4 * 1024 * 1024);
pub(crate) const ERROR_LOG_LIMITS: (usize, usize) = (256, 256 * 1024);

const MAX_AGE: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetEventGap {
    pub expected_sequence: u64,
    pub oldest_available_sequence: u64,
    pub missed_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetEventTryRecvError {
    Empty,
    Lagged(AssetEventGap),
    SequenceExhausted,
    Disconnected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetEventRecvTimeoutError {
    Timeout,
    Lagged(AssetEventGap),
    SequenceExhausted,
    Disconnected,
}

impl std::fmt::Display for AssetEventTryRecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for AssetEventTryRecvError {}
impl std::fmt::Display for AssetEventRecvTimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for AssetEventRecvTimeoutError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AssetEventStreamDiagnostics {
    pub depth: usize,
    pub approximate_bytes: usize,
    pub dropped_count: u64,
    pub sequence_exhausted: bool,
}

struct Entry<T> {
    sequence: u64,
    at: Instant,
    bytes: usize,
    event: Arc<T>,
}
struct Cursor {
    next: AtomicU64,
    wake_pending: AtomicBool,
    wake: Option<ChannelWakeCallback>,
}
struct State<T> {
    entries: VecDeque<Entry<T>>,
    tail: u64,
    bytes: usize,
    dropped: u64,
    exhausted: bool,
    closed: bool,
    cursors: Vec<Weak<Cursor>>,
}
struct Shared<T> {
    state: Mutex<State<T>>,
    ready: Condvar,
    publishers: AtomicUsize,
    max_entries: usize,
    max_bytes: usize,
}

pub(crate) struct AssetEventLog<T> {
    shared: Arc<Shared<T>>,
}
impl<T> Clone for AssetEventLog<T> {
    fn clone(&self) -> Self {
        self.shared.publishers.fetch_add(1, Ordering::Relaxed);
        Self {
            shared: self.shared.clone(),
        }
    }
}
impl<T> Drop for AssetEventLog<T> {
    fn drop(&mut self) {
        if self.shared.publishers.fetch_sub(1, Ordering::AcqRel) == 1 {
            let wakes = {
                let mut state = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
                state.closed = true;
                state.entries.clear();
                state.bytes = 0;
                state
                    .cursors
                    .iter()
                    .filter_map(Weak::upgrade)
                    .filter_map(|cursor| cursor.wake.clone())
                    .collect::<Vec<_>>()
            };
            self.shared.ready.notify_all();
            for wake in wakes {
                wake();
            }
        }
    }
}

impl<T> AssetEventLog<T> {
    pub(crate) fn new(max_entries: usize, max_bytes: usize) -> Self {
        assert!(max_entries > 0 && max_bytes > 0);
        Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    entries: VecDeque::new(),
                    tail: 1,
                    bytes: 0,
                    dropped: 0,
                    exhausted: false,
                    closed: false,
                    cursors: Vec::new(),
                }),
                ready: Condvar::new(),
                publishers: AtomicUsize::new(1),
                max_entries,
                max_bytes,
            }),
        }
    }
    #[cfg(test)]
    pub(crate) fn lock_for_test(&self) -> impl Drop + '_ {
        self.shared.state.lock().unwrap_or_else(|e| e.into_inner())
    }
    #[cfg(test)]
    pub(crate) fn subscriber_count_for_test(&self) -> usize {
        self.shared
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .cursors
            .iter()
            .filter(|cursor| cursor.strong_count() > 0)
            .count()
    }
    pub(crate) fn subscribe(&self, wake: Option<ChannelWakeCallback>) -> AssetEventSubscription<T> {
        let mut state = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
        let cursor = Arc::new(Cursor {
            next: AtomicU64::new(state.tail),
            wake_pending: AtomicBool::new(false),
            wake,
        });
        state.cursors.retain(|cursor| cursor.strong_count() > 0);
        state.cursors.push(Arc::downgrade(&cursor));
        AssetEventSubscription {
            shared: Arc::downgrade(&self.shared),
            cursor,
        }
    }
    /// Returns callbacks for the transaction owner to invoke after releasing its generation fence.
    pub(crate) fn publish(
        &self,
        events: impl IntoIterator<Item = T>,
        charge: impl Fn(&T) -> usize,
    ) -> Vec<ChannelWakeCallback> {
        let mut state = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.closed {
            return Vec::new();
        }
        evict_expired(&mut state, Instant::now());
        let mut changed = false;
        for event in events {
            changed = true;
            let Some(tail) = state.tail.checked_add(1) else {
                state.exhausted = true;
                break;
            };
            let sequence = state.tail;
            state.tail = tail;
            let bytes = charge(&event).saturating_add(2 * std::mem::size_of::<usize>());
            // Charge the actual ring capacity, including unoccupied slots. External Arc leases
            // and each subscriber's cursor belong to those consumers, outside the retained log.
            let entry_bytes = std::mem::size_of::<Entry<T>>();
            if bytes.saturating_add(entry_bytes) > self.shared.max_bytes {
                state.dropped = state
                    .dropped
                    .saturating_add(state.entries.len() as u64)
                    .saturating_add(1);
                state.entries = VecDeque::new();
                state.bytes = 0;
                continue;
            }
            while state.entries.len() >= self.shared.max_entries
                || retained_bytes(&state).saturating_add(bytes) > self.shared.max_bytes
            {
                if state.entries.is_empty() {
                    // A previous burst must not prevent a larger, individually admissible event.
                    state.entries = VecDeque::new();
                    break;
                }
                pop_oldest(&mut state);
            }
            if state.entries.len() == state.entries.capacity() {
                let capacity = state.entries.capacity();
                let budget_slots =
                    self.shared.max_bytes.saturating_sub(state.bytes + bytes) / entry_bytes;
                let target = capacity
                    .saturating_mul(2)
                    .max(4)
                    .min(self.shared.max_entries)
                    .min(budget_slots);
                if target > capacity {
                    let additional = target - state.entries.len();
                    state.entries.reserve_exact(additional);
                } else {
                    pop_oldest(&mut state);
                }
            }
            // reserve_exact's reported capacity is authoritative, even for a different allocator.
            while retained_bytes(&state).saturating_add(bytes) > self.shared.max_bytes {
                if state.entries.is_empty() {
                    state.entries = VecDeque::with_capacity(1);
                    break;
                }
                pop_oldest(&mut state);
            }
            state.bytes += bytes;
            state.entries.push_back(Entry {
                sequence,
                at: Instant::now(),
                bytes,
                event: Arc::new(event),
            });
        }
        let mut wakes = Vec::new();
        if changed {
            state.cursors.retain(|weak| {
                let Some(cursor) = weak.upgrade() else {
                    return false;
                };
                if !cursor.wake_pending.swap(true, Ordering::Relaxed) {
                    if let Some(wake) = &cursor.wake {
                        wakes.push(wake.clone());
                    }
                }
                true
            });
        }
        drop(state);
        if changed {
            self.shared.ready.notify_all();
        }
        wakes
    }
}

fn retained_bytes<T>(state: &State<T>) -> usize {
    state.bytes.saturating_add(
        state
            .entries
            .capacity()
            .saturating_mul(std::mem::size_of::<Entry<T>>()),
    )
}

fn pop_oldest<T>(state: &mut State<T>) {
    if let Some(entry) = state.entries.pop_front() {
        state.bytes -= entry.bytes;
        state.dropped = state.dropped.saturating_add(1);
    }
}
fn evict_expired<T>(state: &mut State<T>, now: Instant) {
    while state
        .entries
        .front()
        .is_some_and(|entry| now.saturating_duration_since(entry.at) >= MAX_AGE)
    {
        pop_oldest(state);
    }
}

pub struct AssetEventSubscription<T> {
    shared: Weak<Shared<T>>,
    cursor: Arc<Cursor>,
}
impl<T> Clone for AssetEventSubscription<T> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
            cursor: self.cursor.clone(),
        }
    }
}
impl<T> AssetEventSubscription<T> {
    pub fn try_recv(&self) -> Result<Arc<T>, AssetEventTryRecvError> {
        let shared = self
            .shared
            .upgrade()
            .ok_or(AssetEventTryRecvError::Disconnected)?;
        let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
        let next = self.cursor.next.load(Ordering::Relaxed);
        self.cursor.wake_pending.store(false, Ordering::Relaxed);
        evict_expired(&mut state, Instant::now());
        if state.closed {
            return Err(AssetEventTryRecvError::Disconnected);
        }
        let oldest = state
            .entries
            .front()
            .map_or(state.tail, |entry| entry.sequence);
        if next < oldest {
            let gap = AssetEventGap {
                expected_sequence: next,
                oldest_available_sequence: oldest,
                missed_count: oldest - next,
            };
            self.cursor.next.store(oldest, Ordering::Relaxed);
            return Err(AssetEventTryRecvError::Lagged(gap));
        }
        if next < state.tail {
            let entry = &state.entries[(next - oldest) as usize];
            self.cursor.next.store(next + 1, Ordering::Relaxed);
            return Ok(entry.event.clone());
        }
        Err(if state.exhausted {
            AssetEventTryRecvError::SequenceExhausted
        } else {
            AssetEventTryRecvError::Empty
        })
    }
    pub fn len(&self) -> usize {
        let Some(shared) = self.shared.upgrade() else {
            return 0;
        };
        let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
        let next = self.cursor.next.load(Ordering::Relaxed);
        evict_expired(&mut state, Instant::now());
        if state.closed {
            0
        } else {
            state
                .tail
                .saturating_sub(next)
                .min(state.entries.len().saturating_add(1) as u64) as usize
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn diagnostics(&self) -> AssetEventStreamDiagnostics {
        let Some(shared) = self.shared.upgrade() else {
            return AssetEventStreamDiagnostics::default();
        };
        let mut state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
        evict_expired(&mut state, Instant::now());
        AssetEventStreamDiagnostics {
            depth: state.entries.len(),
            approximate_bytes: retained_bytes(&state),
            dropped_count: state.dropped,
            sequence_exhausted: state.exhausted,
        }
    }
    pub fn recv_timeout(&self, timeout: Duration) -> Result<Arc<T>, AssetEventRecvTimeoutError> {
        let deadline = Instant::now().checked_add(timeout);
        loop {
            match self.try_recv() {
                Ok(event) => return Ok(event),
                Err(AssetEventTryRecvError::Lagged(gap)) => {
                    return Err(AssetEventRecvTimeoutError::Lagged(gap));
                }
                Err(AssetEventTryRecvError::Disconnected) => {
                    return Err(AssetEventRecvTimeoutError::Disconnected);
                }
                Err(AssetEventTryRecvError::SequenceExhausted) => {
                    return Err(AssetEventRecvTimeoutError::SequenceExhausted);
                }
                Err(AssetEventTryRecvError::Empty) => {}
            }
            let remaining = deadline.map_or(timeout, |deadline| {
                deadline.saturating_duration_since(Instant::now())
            });
            if remaining.is_zero() {
                return Err(AssetEventRecvTimeoutError::Timeout);
            }
            let shared = self
                .shared
                .upgrade()
                .ok_or(AssetEventRecvTimeoutError::Disconnected)?;
            let state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
            // Recheck under the publisher's mutex before sleeping to avoid a missed notification.
            let next = self.cursor.next.load(Ordering::Relaxed);
            if state.closed || state.tail != next || state.exhausted {
                continue;
            }
            drop(
                shared
                    .ready
                    .wait_timeout(state, remaining)
                    .unwrap_or_else(|e| e.into_inner()),
            );
        }
    }
    /// Iteration exposes loss and terminal states instead of silently discarding them.
    pub fn try_iter(&self) -> impl Iterator<Item = Result<Arc<T>, AssetEventTryRecvError>> + '_ {
        let mut finished = false;
        std::iter::from_fn(move || {
            if finished {
                return None;
            }
            match self.try_recv() {
                Err(AssetEventTryRecvError::Empty) => {
                    finished = true;
                    None
                }
                result => {
                    finished = result.is_err();
                    Some(result)
                }
            }
        })
    }
    fn reset_to_tail(&self) -> Result<u64, AssetEventTryRecvError> {
        let shared = self
            .shared
            .upgrade()
            .ok_or(AssetEventTryRecvError::Disconnected)?;
        let state = shared.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.closed {
            return Err(AssetEventTryRecvError::Disconnected);
        }
        if state.exhausted {
            return Err(AssetEventTryRecvError::SequenceExhausted);
        }
        self.cursor.next.store(state.tail, Ordering::Relaxed);
        self.cursor.wake_pending.store(false, Ordering::Relaxed);
        Ok(state.tail)
    }
}

#[derive(Clone)]
pub struct AssetChangeSubscription {
    inner: AssetEventSubscription<AssetChange>,
    gate: Weak<RwLock<()>>,
    generation: Weak<RwLock<Arc<ProjectAssetManagementGeneration>>>,
    project: Weak<RwLock<Option<crate::asset::project::ProjectManager>>>,
}
impl std::ops::Deref for AssetChangeSubscription {
    type Target = AssetEventSubscription<AssetChange>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl AssetChangeSubscription {
    pub(crate) fn new(
        inner: AssetEventSubscription<AssetChange>,
        gate: &Arc<RwLock<()>>,
        generation: &Arc<RwLock<Arc<ProjectAssetManagementGeneration>>>,
        project: &Arc<RwLock<Option<crate::asset::project::ProjectManager>>>,
    ) -> Self {
        Self {
            inner,
            gate: Arc::downgrade(gate),
            generation: Arc::downgrade(generation),
            project: Arc::downgrade(project),
        }
    }
    /// Snapshot publication and cursor reset share the producer's generation fence.
    pub fn resync(&self) -> Result<AssetChangeSnapshot, AssetEventTryRecvError> {
        let gate = self
            .gate
            .upgrade()
            .ok_or(AssetEventTryRecvError::Disconnected)?;
        let generation = self
            .generation
            .upgrade()
            .ok_or(AssetEventTryRecvError::Disconnected)?;
        let _fence = gate.read().unwrap_or_else(|e| e.into_inner());
        let project = self
            .project
            .upgrade()
            .ok_or(AssetEventTryRecvError::Disconnected)?;
        let catalog = project
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .map(|project| project.catalog_input_generation());
        let management = generation.read().unwrap_or_else(|e| e.into_inner()).clone();
        let next_sequence = self.inner.reset_to_tail()?;
        Ok(AssetChangeSnapshot {
            catalog,
            management,
            next_sequence,
        })
    }
}
/// A shared immutable directory and the first event following its publication boundary.
pub struct AssetChangeSnapshot {
    pub catalog: Option<Arc<crate::asset::project::ProjectCatalogInputGeneration>>,
    pub management: Arc<ProjectAssetManagementGeneration>,
    pub next_sequence: u64,
}

pub type AssetWatchErrorSubscription = AssetEventSubscription<AssetWatchError>;

pub(crate) fn change_bytes(event: &AssetChange) -> usize {
    std::mem::size_of_val(event)
        + event.uri.retained_heap_bytes()
        + event
            .previous_uri
            .as_ref()
            .map_or(0, |uri| uri.retained_heap_bytes())
}
pub(crate) fn error_bytes(event: &AssetWatchError) -> usize {
    std::mem::size_of_val(event)
        + event.message.capacity()
        + event.assets_root.capacity().saturating_mul(4)
        + event.paths.capacity() * std::mem::size_of::<std::path::PathBuf>()
        + event
            .paths
            .iter()
            .map(|path| path.capacity().saturating_mul(4))
            .sum::<usize>()
}

#[cfg(test)]
#[path = "tests/event_stream.rs"]
mod tests;

#[cfg(all(test, feature = "memory-validation"))]
#[path = "tests/event_memory_profile.rs"]
mod memory_profile;
