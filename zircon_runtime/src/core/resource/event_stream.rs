use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use super::{ResourceEvent, ResourceEventKind};

const RESOURCE_EVENT_LOG_ENTRY_CAPACITY: usize = 4_096;
const RESOURCE_EVENT_LOG_BYTE_CAPACITY: usize = 4 * 1024 * 1024;
const RESOURCE_EVENT_LOG_MAX_AGE: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceEventStreamDiagnostics {
    pub depth: usize,
    pub approximate_bytes: usize,
    pub oldest_age: Duration,
    pub coalesced_count: u64,
    pub dropped_count: u64,
    pub lagged_read_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceEventGap {
    pub expected_sequence: u64,
    pub oldest_available_sequence: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceEventTryRecvError {
    Empty,
    Lagged(ResourceEventGap),
    Disconnected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceEventRecvError {
    Lagged(ResourceEventGap),
    Disconnected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceEventRecvTimeoutError {
    Timeout,
    Lagged(ResourceEventGap),
    Disconnected,
}

impl Display for ResourceEventTryRecvError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Display for ResourceEventRecvError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Display for ResourceEventRecvTimeoutError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ResourceEventTryRecvError {}
impl std::error::Error for ResourceEventRecvError {}
impl std::error::Error for ResourceEventRecvTimeoutError {}

#[derive(Clone, Debug)]
struct LoggedResourceEvent {
    sequence: u64,
    published_at: Instant,
    approximate_bytes: usize,
    event: ResourceEvent,
}

#[derive(Debug)]
struct ResourceEventLogState {
    next_sequence: u64,
    entries: VecDeque<LoggedResourceEvent>,
    approximate_bytes: usize,
    coalesced_count: u64,
    dropped_count: u64,
    lagged_read_count: u64,
}

impl Default for ResourceEventLogState {
    fn default() -> Self {
        Self {
            next_sequence: 1,
            entries: VecDeque::new(),
            approximate_bytes: 0,
            coalesced_count: 0,
            dropped_count: 0,
            lagged_read_count: 0,
        }
    }
}

#[derive(Debug, Default)]
struct ResourceEventHub {
    state: Mutex<ResourceEventLogState>,
    changed: Condvar,
    publisher_count: AtomicUsize,
}

impl ResourceEventHub {
    fn lock_state(&self) -> MutexGuard<'_, ResourceEventLogState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[derive(Debug)]
pub(crate) struct ResourceEventPublisher {
    hub: Arc<ResourceEventHub>,
}

impl Default for ResourceEventPublisher {
    fn default() -> Self {
        let hub = Arc::new(ResourceEventHub::default());
        hub.publisher_count.store(1, Ordering::Release);
        Self { hub }
    }
}

impl Clone for ResourceEventPublisher {
    fn clone(&self) -> Self {
        self.hub.publisher_count.fetch_add(1, Ordering::AcqRel);
        Self {
            hub: self.hub.clone(),
        }
    }
}

impl Drop for ResourceEventPublisher {
    fn drop(&mut self) {
        let _state = self.hub.lock_state();
        if self.hub.publisher_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.hub.changed.notify_all();
        }
    }
}

impl ResourceEventPublisher {
    pub(crate) fn subscribe(&self) -> ResourceEventReceiver {
        let next_sequence = self.hub.lock_state().next_sequence;
        ResourceEventReceiver {
            hub: self.hub.clone(),
            cursor: Mutex::new(next_sequence),
        }
    }

    pub(crate) fn publish(&self, event: ResourceEvent) {
        let now = Instant::now();
        let mut state = self.hub.lock_state();
        evict_expired(&mut state, now);
        if is_coalescable(event.kind) {
            if let Some(index) = state.entries.iter().rposition(|existing| {
                existing.event.id == event.id && existing.event.resource_kind == event.resource_kind
            }) {
                if is_coalescable(state.entries[index].event.kind) {
                    if let Some(previous) = state.entries.remove(index) {
                        state.approximate_bytes = state
                            .approximate_bytes
                            .saturating_sub(previous.approximate_bytes);
                        state.coalesced_count = state.coalesced_count.saturating_add(1);
                    }
                }
            }
        }
        let sequence = state.next_sequence;
        state.next_sequence = state.next_sequence.wrapping_add(1);
        let approximate_bytes = approximate_event_bytes(&event);
        state.approximate_bytes = state.approximate_bytes.saturating_add(approximate_bytes);
        state.entries.push_back(LoggedResourceEvent {
            sequence,
            published_at: now,
            approximate_bytes,
            event,
        });
        enforce_capacity(&mut state);
        drop(state);
        self.hub.changed.notify_all();
    }

    pub(crate) fn diagnostics(&self) -> ResourceEventStreamDiagnostics {
        let now = Instant::now();
        let mut state = self.hub.lock_state();
        evict_expired(&mut state, now);
        ResourceEventStreamDiagnostics {
            depth: state.entries.len(),
            approximate_bytes: state.approximate_bytes,
            oldest_age: state
                .entries
                .front()
                .map(|entry| now.saturating_duration_since(entry.published_at))
                .unwrap_or_default(),
            coalesced_count: state.coalesced_count,
            dropped_count: state.dropped_count,
            lagged_read_count: state.lagged_read_count,
        }
    }

    #[cfg(test)]
    pub(crate) fn poison_state(&self) {
        let _guard = self.hub.state.lock().unwrap();
        panic!("poison resource event stream");
    }
}

#[derive(Debug)]
pub struct ResourceEventReceiver {
    hub: Arc<ResourceEventHub>,
    cursor: Mutex<u64>,
}

impl ResourceEventReceiver {
    pub fn len(&self) -> usize {
        let cursor = lock_cursor(&self.cursor);
        self.hub
            .lock_state()
            .entries
            .iter()
            .filter(|entry| entry.sequence >= *cursor)
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn try_recv(&self) -> Result<ResourceEvent, ResourceEventTryRecvError> {
        let mut cursor = lock_cursor(&self.cursor);
        let mut state = self.hub.lock_state();
        match take_next(&mut state, &mut cursor) {
            EventRead::Event(event) => Ok(event),
            EventRead::Lagged(gap) => Err(ResourceEventTryRecvError::Lagged(gap)),
            EventRead::Empty if self.is_disconnected() => {
                Err(ResourceEventTryRecvError::Disconnected)
            }
            EventRead::Empty => Err(ResourceEventTryRecvError::Empty),
        }
    }

    pub fn recv(&self) -> Result<ResourceEvent, ResourceEventRecvError> {
        let mut cursor = lock_cursor(&self.cursor);
        let mut state = self.hub.lock_state();
        loop {
            match take_next(&mut state, &mut cursor) {
                EventRead::Event(event) => return Ok(event),
                EventRead::Lagged(gap) => return Err(ResourceEventRecvError::Lagged(gap)),
                EventRead::Empty if self.is_disconnected() => {
                    return Err(ResourceEventRecvError::Disconnected);
                }
                EventRead::Empty => {
                    state = self
                        .hub
                        .changed
                        .wait(state)
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                }
            }
        }
    }

    pub fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<ResourceEvent, ResourceEventRecvTimeoutError> {
        let deadline = Instant::now().checked_add(timeout);
        let mut cursor = lock_cursor(&self.cursor);
        let mut state = self.hub.lock_state();
        loop {
            match take_next(&mut state, &mut cursor) {
                EventRead::Event(event) => return Ok(event),
                EventRead::Lagged(gap) => {
                    return Err(ResourceEventRecvTimeoutError::Lagged(gap));
                }
                EventRead::Empty if self.is_disconnected() => {
                    return Err(ResourceEventRecvTimeoutError::Disconnected);
                }
                EventRead::Empty => {}
            }
            let remaining = deadline
                .map(|deadline| deadline.saturating_duration_since(Instant::now()))
                .unwrap_or(timeout);
            if remaining.is_zero() {
                return Err(ResourceEventRecvTimeoutError::Timeout);
            }
            let (next_state, wait) = self
                .hub
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            if wait.timed_out() && deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                return Err(ResourceEventRecvTimeoutError::Timeout);
            }
        }
    }

    fn is_disconnected(&self) -> bool {
        self.hub.publisher_count.load(Ordering::Acquire) == 0
    }
}

fn lock_cursor(cursor: &Mutex<u64>) -> MutexGuard<'_, u64> {
    cursor
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

enum EventRead {
    Event(ResourceEvent),
    Empty,
    Lagged(ResourceEventGap),
}

fn take_next(state: &mut ResourceEventLogState, cursor: &mut u64) -> EventRead {
    evict_expired(state, Instant::now());
    let Some(next) = state.entries.iter().find(|entry| entry.sequence >= *cursor) else {
        if *cursor != state.next_sequence {
            let gap = ResourceEventGap {
                expected_sequence: *cursor,
                oldest_available_sequence: state.next_sequence,
            };
            *cursor = state.next_sequence;
            state.lagged_read_count = state.lagged_read_count.saturating_add(1);
            return EventRead::Lagged(gap);
        }
        return EventRead::Empty;
    };
    if next.sequence > *cursor {
        let gap = ResourceEventGap {
            expected_sequence: *cursor,
            oldest_available_sequence: next.sequence,
        };
        *cursor = next.sequence;
        state.lagged_read_count = state.lagged_read_count.saturating_add(1);
        return EventRead::Lagged(gap);
    }
    *cursor = (*cursor).wrapping_add(1);
    EventRead::Event(next.event.clone())
}

fn is_coalescable(kind: ResourceEventKind) -> bool {
    matches!(kind, ResourceEventKind::Added | ResourceEventKind::Updated)
}

pub(crate) fn approximate_event_bytes(event: &ResourceEvent) -> usize {
    std::mem::size_of::<ResourceEvent>()
        + event
            .locator
            .as_ref()
            .map(approximate_locator_bytes)
            .unwrap_or(0)
        + event
            .previous_locator
            .as_ref()
            .map(approximate_locator_bytes)
            .unwrap_or(0)
}

fn approximate_locator_bytes(locator: &super::ResourceLocator) -> usize {
    locator.path().len() + locator.label().map(str::len).unwrap_or(0) + 12
}

fn evict_expired(state: &mut ResourceEventLogState, now: Instant) {
    while state.entries.front().is_some_and(|entry| {
        now.saturating_duration_since(entry.published_at) > RESOURCE_EVENT_LOG_MAX_AGE
    }) {
        drop_oldest(state);
    }
}

fn enforce_capacity(state: &mut ResourceEventLogState) {
    while state.entries.len() > RESOURCE_EVENT_LOG_ENTRY_CAPACITY
        || state.approximate_bytes > RESOURCE_EVENT_LOG_BYTE_CAPACITY
    {
        drop_oldest(state);
    }
}

fn drop_oldest(state: &mut ResourceEventLogState) {
    if let Some(removed) = state.entries.pop_front() {
        state.approximate_bytes = state
            .approximate_bytes
            .saturating_sub(removed.approximate_bytes);
        state.dropped_count = state.dropped_count.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceScheme};

    fn event(id: usize, kind: ResourceEventKind, revision: u64) -> ResourceEvent {
        ResourceEvent {
            kind,
            resource_kind: ResourceKind::Texture,
            id: ResourceId::from_stable_label(&format!("event-{id}")),
            locator: None,
            previous_locator: None,
            revision,
        }
    }

    #[test]
    fn stalled_resource_event_consumer_observes_a_bounded_gap() {
        let publisher = ResourceEventPublisher::default();
        let receiver = publisher.subscribe();
        for id in 0..(RESOURCE_EVENT_LOG_ENTRY_CAPACITY + 32) {
            publisher.publish(event(id, ResourceEventKind::Renamed, 1));
        }

        assert!(matches!(
            receiver.try_recv(),
            Err(ResourceEventTryRecvError::Lagged(_))
        ));
        let diagnostics = publisher.diagnostics();
        assert_eq!(diagnostics.depth, RESOURCE_EVENT_LOG_ENTRY_CAPACITY);
        assert_eq!(diagnostics.dropped_count, 32);
        assert_eq!(diagnostics.lagged_read_count, 1);
    }

    #[test]
    fn resource_event_stream_scale_matrix_stays_bounded_at_one_thousand_and_one_hundred_thousand() {
        for event_count in [1usize, 1_000, 100_000] {
            let publisher = ResourceEventPublisher::default();
            let receiver = publisher.subscribe();
            for id in 0..event_count {
                publisher.publish(event(id, ResourceEventKind::Renamed, 1));
            }

            let retained = event_count.min(RESOURCE_EVENT_LOG_ENTRY_CAPACITY);
            let diagnostics = publisher.diagnostics();
            assert_eq!(diagnostics.depth, retained);
            assert!(diagnostics.approximate_bytes <= RESOURCE_EVENT_LOG_BYTE_CAPACITY);
            assert_eq!(diagnostics.dropped_count as usize, event_count - retained);

            if event_count > RESOURCE_EVENT_LOG_ENTRY_CAPACITY {
                assert!(matches!(
                    receiver.try_recv(),
                    Err(ResourceEventTryRecvError::Lagged(_))
                ));
            }
            let mut consumed = 0usize;
            while receiver.try_recv().is_ok() {
                consumed = consumed.saturating_add(1);
            }
            assert_eq!(consumed, retained);
        }
    }

    #[test]
    fn resource_event_log_coalesces_updates_but_preserves_lifecycle_edges() {
        let publisher = ResourceEventPublisher::default();
        let receiver = publisher.subscribe();
        publisher.publish(event(1, ResourceEventKind::Added, 1));
        publisher.publish(event(1, ResourceEventKind::Updated, 2));
        publisher.publish(event(1, ResourceEventKind::Renamed, 2));
        publisher.publish(event(1, ResourceEventKind::Updated, 3));
        publisher.publish(event(1, ResourceEventKind::Removed, 3));

        assert!(matches!(
            receiver.try_recv(),
            Err(ResourceEventTryRecvError::Lagged(_))
        ));
        assert_eq!(
            receiver.try_recv().unwrap().kind,
            ResourceEventKind::Updated
        );
        assert_eq!(
            receiver.try_recv().unwrap().kind,
            ResourceEventKind::Renamed
        );
        assert_eq!(
            receiver.try_recv().unwrap().kind,
            ResourceEventKind::Updated
        );
        assert_eq!(
            receiver.try_recv().unwrap().kind,
            ResourceEventKind::Removed
        );
        assert_eq!(publisher.diagnostics().coalesced_count, 1);
    }

    #[test]
    fn resource_event_log_enforces_the_byte_budget_independently_from_entry_count() {
        let publisher = ResourceEventPublisher::default();
        let receiver = publisher.subscribe();
        let large_label = "x".repeat(RESOURCE_EVENT_LOG_BYTE_CAPACITY / 2);
        for id in 0..3 {
            let mut resource_event = event(id, ResourceEventKind::Renamed, 1);
            resource_event.locator = Some(
                ResourceLocator::new(
                    ResourceScheme::Memory,
                    format!("event-{id}"),
                    Some(large_label.clone()),
                )
                .unwrap(),
            );
            publisher.publish(resource_event);
        }

        assert!(matches!(
            receiver.try_recv(),
            Err(ResourceEventTryRecvError::Lagged(_))
        ));
        let diagnostics = publisher.diagnostics();
        assert!(diagnostics.depth < 3);
        assert!(diagnostics.approximate_bytes <= RESOURCE_EVENT_LOG_BYTE_CAPACITY);
        assert!(diagnostics.dropped_count > 0);
    }

    #[test]
    fn resource_event_receiver_disconnects_after_the_last_publisher_is_dropped() {
        let publisher = ResourceEventPublisher::default();
        let receiver = publisher.subscribe();
        drop(publisher);

        assert_eq!(
            receiver.try_recv(),
            Err(ResourceEventTryRecvError::Disconnected)
        );
    }

    #[test]
    fn blocking_resource_event_receiver_wakes_when_the_last_publisher_is_dropped() {
        let publisher = ResourceEventPublisher::default();
        let receiver = publisher.subscribe();
        let (started_tx, started_rx) = std::sync::mpsc::sync_channel(0);
        let receiver_thread = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            receiver.recv()
        });

        started_rx.recv().unwrap();
        drop(publisher);

        assert_eq!(
            receiver_thread.join().unwrap(),
            Err(ResourceEventRecvError::Disconnected)
        );
    }

    #[test]
    fn ten_thousand_subscribers_share_one_logged_event() {
        let publisher = ResourceEventPublisher::default();
        let receivers = (0..10_000)
            .map(|_| publisher.subscribe())
            .collect::<Vec<_>>();

        publisher.publish(event(7, ResourceEventKind::Renamed, 3));

        assert_eq!(publisher.diagnostics().depth, 1);
        for receiver in receivers {
            let received = receiver.try_recv().unwrap();
            assert_eq!(received.revision, 3);
            assert_eq!(received.kind, ResourceEventKind::Renamed);
        }
        assert_eq!(publisher.diagnostics().depth, 1);
    }

    #[test]
    fn resource_event_log_expires_old_entries_by_ttl() {
        let mut state = ResourceEventLogState::default();
        let event = event(1, ResourceEventKind::Renamed, 1);
        let approximate_bytes = approximate_event_bytes(&event);
        state.approximate_bytes = approximate_bytes;
        state.entries.push_back(LoggedResourceEvent {
            sequence: 1,
            published_at: Instant::now() - RESOURCE_EVENT_LOG_MAX_AGE - Duration::from_millis(1),
            approximate_bytes,
            event,
        });

        evict_expired(&mut state, Instant::now());

        assert!(state.entries.is_empty());
        assert_eq!(state.approximate_bytes, 0);
        assert_eq!(state.dropped_count, 1);
    }

    #[test]
    fn resource_event_log_reports_a_gap_after_eviction_empties_the_log() {
        let mut state = ResourceEventLogState {
            next_sequence: 9,
            dropped_count: 8,
            ..Default::default()
        };
        let mut cursor = 1;

        let read = take_next(&mut state, &mut cursor);

        assert!(matches!(
            read,
            EventRead::Lagged(ResourceEventGap {
                expected_sequence: 1,
                oldest_available_sequence: 9,
            })
        ));
        assert_eq!(cursor, 9);
        assert_eq!(state.lagged_read_count, 1);
    }
}
