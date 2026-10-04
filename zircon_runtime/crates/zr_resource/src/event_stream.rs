//! 面向缓存失效与增量同步的有界共享日志；每个订阅者持有自己的游标，载荷只存一份。
//! 更新可合并，旧事件可因容量或时限淘汰；缺口要求调用端重新读取权威快照后再继续同步。

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};
use std::time::{Duration, Instant};

use super::{
    ResourceEvent, ResourceEventKind, ResourceId, ResourceKind, ResourceRegistryError,
    ResourceResult,
};

const RESOURCE_EVENT_LOG_ENTRY_CAPACITY: usize = 4_096;
const RESOURCE_EVENT_LOG_BYTE_CAPACITY: usize = 4 * 1024 * 1024;
const RESOURCE_EVENT_LOG_MAX_AGE: Duration = Duration::from_secs(60);

type ResourceEventIdentity = (ResourceKind, ResourceId);

/// 日志当前保留量与累计淘汰统计；字节数是估算值，不能作为实际堆占用或完整历史的证明。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceEventStreamDiagnostics {
    pub depth: usize,
    pub approximate_bytes: usize,
    pub oldest_age: Duration,
    pub coalesced_count: u64,
    pub dropped_count: u64,
    pub lagged_read_count: u64,
    pub sequence_exhausted: bool,
    pub rejected_publish_count: u64,
}

/// 游标与下一条可读事件之间的缺口。可用序号为 `None` 时，序号空间已经耗尽。
/// 消费者应先重新同步资源状态；接收端已经把游标推进到可继续读取的位置。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceEventGap {
    pub expected_sequence: u64,
    pub oldest_available_sequence: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceEventTryRecvError {
    Empty,
    Lagged(ResourceEventGap),
    SequenceExhausted,
    Disconnected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceEventRecvError {
    Lagged(ResourceEventGap),
    SequenceExhausted,
    Disconnected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceEventRecvTimeoutError {
    Timeout,
    Lagged(ResourceEventGap),
    SequenceExhausted,
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
struct ResourceEventLogNode {
    entry: LoggedResourceEvent,
    previous_slot: Option<usize>,
    next_slot: Option<usize>,
}

#[derive(Debug)]
struct ResourceEventLogEntries {
    slots: Vec<Option<ResourceEventLogNode>>,
    free_slots: Vec<usize>,
    oldest_slot: Option<usize>,
    newest_slot: Option<usize>,
    recent_slot_by_sequence: Vec<Option<(u64, usize)>>,
}

impl Default for ResourceEventLogEntries {
    fn default() -> Self {
        Self {
            slots: Vec::with_capacity(RESOURCE_EVENT_LOG_ENTRY_CAPACITY),
            free_slots: Vec::new(),
            oldest_slot: None,
            newest_slot: None,
            recent_slot_by_sequence: vec![None; RESOURCE_EVENT_LOG_ENTRY_CAPACITY],
        }
    }
}

impl ResourceEventLogEntries {
    fn len(&self) -> usize {
        self.slots.len().saturating_sub(self.free_slots.len())
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.oldest_slot.is_none()
    }

    fn get(&self, slot: usize) -> Option<&LoggedResourceEvent> {
        self.slots
            .get(slot)
            .and_then(Option::as_ref)
            .map(|node| &node.entry)
    }

    fn first(&self) -> Option<&LoggedResourceEvent> {
        self.oldest_slot.and_then(|slot| self.get(slot))
    }

    fn newest_slot(&self) -> Option<usize> {
        self.newest_slot
    }

    fn insert_back(&mut self, entry: LoggedResourceEvent) -> usize {
        let sequence = entry.sequence;
        let previous_slot = self.newest_slot;
        let slot = self.free_slots.pop().unwrap_or_else(|| {
            self.slots.push(None);
            self.slots.len() - 1
        });
        self.slots[slot] = Some(ResourceEventLogNode {
            entry,
            previous_slot,
            next_slot: None,
        });
        if let Some(previous_slot) = previous_slot {
            if let Some(previous) = self.slots[previous_slot].as_mut() {
                previous.next_slot = Some(slot);
            } else {
                self.oldest_slot = Some(slot);
                if let Some(inserted) = self.slots[slot].as_mut() {
                    inserted.previous_slot = None;
                }
            }
        } else {
            self.oldest_slot = Some(slot);
        }
        self.newest_slot = Some(slot);
        let recent_index = (sequence % self.recent_slot_by_sequence.len() as u64) as usize;
        self.recent_slot_by_sequence[recent_index] = Some((sequence, slot));
        slot
    }

    fn remove(&mut self, slot: usize) -> Option<LoggedResourceEvent> {
        let node = self.slots.get_mut(slot)?.take()?;
        if let Some(previous_slot) = node.previous_slot {
            if let Some(previous) = self.slots[previous_slot].as_mut() {
                previous.next_slot = node.next_slot;
            } else {
                self.oldest_slot = node.next_slot;
            }
        } else {
            self.oldest_slot = node.next_slot;
        }
        if let Some(next_slot) = node.next_slot {
            if let Some(next) = self.slots[next_slot].as_mut() {
                next.previous_slot = node.previous_slot;
            } else {
                self.newest_slot = node.previous_slot;
            }
        } else {
            self.newest_slot = node.previous_slot;
        }
        let recent_index =
            (node.entry.sequence % self.recent_slot_by_sequence.len() as u64) as usize;
        if self.recent_slot_by_sequence[recent_index] == Some((node.entry.sequence, slot)) {
            self.recent_slot_by_sequence[recent_index] = None;
        }
        self.free_slots.push(slot);
        Some(node.entry)
    }

    fn pop_front(&mut self) -> Option<(usize, LoggedResourceEvent)> {
        let slot = self.oldest_slot?;
        self.remove(slot).map(|entry| (slot, entry))
    }

    fn exact_slot(&self, sequence: u64) -> Option<usize> {
        let recent_index = (sequence % self.recent_slot_by_sequence.len() as u64) as usize;
        let (indexed_sequence, slot) = self.recent_slot_by_sequence[recent_index]?;
        (indexed_sequence == sequence
            && self
                .get(slot)
                .is_some_and(|entry| entry.sequence == sequence))
        .then_some(slot)
    }

    fn slot_at_or_after(&self, sequence: u64) -> Option<usize> {
        if let Some(slot) = self.exact_slot(sequence) {
            return Some(slot);
        }
        let mut slot = self.oldest_slot;
        while let Some(current_slot) = slot {
            let Some(node) = self.slots[current_slot].as_ref() else {
                return None;
            };
            if node.entry.sequence >= sequence {
                return Some(current_slot);
            }
            slot = node.next_slot;
        }
        None
    }

    fn first_at_or_after(&self, sequence: u64) -> Option<&LoggedResourceEvent> {
        self.slot_at_or_after(sequence)
            .and_then(|slot| self.get(slot))
    }

    fn count_at_or_after(&self, sequence: u64) -> usize {
        let mut slot = self.slot_at_or_after(sequence);
        let mut count = 0_usize;
        while let Some(current_slot) = slot {
            let Some(node) = self.slots[current_slot].as_ref() else {
                break;
            };
            count = count.saturating_add(1);
            slot = node.next_slot;
        }
        count
    }

    #[cfg(test)]
    fn values(&self) -> impl Iterator<Item = &LoggedResourceEvent> {
        std::iter::successors(self.oldest_slot, |slot| {
            self.slots[*slot].as_ref().and_then(|node| node.next_slot)
        })
        .filter_map(|slot| self.get(slot))
    }
}

#[derive(Debug)]
struct ResourceEventLogState {
    next_sequence: Option<u64>,
    entries: ResourceEventLogEntries,
    latest_slot_by_identity: HashMap<ResourceEventIdentity, usize>,
    approximate_bytes: usize,
    coalesced_count: u64,
    dropped_count: u64,
    lagged_read_count: u64,
    rejected_publish_count: u64,
}

impl Default for ResourceEventLogState {
    fn default() -> Self {
        Self {
            next_sequence: Some(1),
            entries: ResourceEventLogEntries::default(),
            latest_slot_by_identity: HashMap::with_capacity(RESOURCE_EVENT_LOG_ENTRY_CAPACITY),
            approximate_bytes: 0,
            coalesced_count: 0,
            dropped_count: 0,
            lagged_read_count: 0,
            rejected_publish_count: 0,
        }
    }
}

#[derive(Debug, Default)]
struct ResourceEventHub {
    state: Mutex<ResourceEventLogState>,
    changed: Condvar,
}

impl ResourceEventHub {
    fn lock_state(&self) -> MutexGuard<'_, ResourceEventLogState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[derive(Debug)]
struct ResourceEventPublisherLifetime {
    hub: Arc<ResourceEventHub>,
}

impl Drop for ResourceEventPublisherLifetime {
    fn drop(&mut self) {
        let _state = self.hub.lock_state();
        self.hub.changed.notify_all();
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResourceEventPublisher {
    hub: Arc<ResourceEventHub>,
    lifetime: Arc<ResourceEventPublisherLifetime>,
}

impl Default for ResourceEventPublisher {
    fn default() -> Self {
        let hub = Arc::new(ResourceEventHub::default());
        Self {
            hub: Arc::clone(&hub),
            lifetime: Arc::new(ResourceEventPublisherLifetime { hub }),
        }
    }
}

#[derive(Debug)]
// 批次预检只确认序号空间足够，并未提前保留序号；调用端必须一直持有发布串行锁，
// 直到应用权威状态并写入事件完成，避免其他批次抢占这段序号。
pub(crate) struct ResourceEventPublishPermit {
    first_sequence: Option<u64>,
    successor_sequence: Option<u64>,
    event_count: usize,
}

impl ResourceEventPublisher {
    /// 从订阅时刻之后的事件开始；初始化视图仍须读取管理器快照，不能靠此日志重建全部目录。
    pub(crate) fn subscribe(&self) -> ResourceEventReceiver {
        let next_sequence = self.hub.lock_state().next_sequence;
        ResourceEventReceiver {
            hub: self.hub.clone(),
            publisher_lifetime: Arc::downgrade(&self.lifetime),
            cursor: Mutex::new(next_sequence),
        }
    }

    pub(crate) fn prepare_publish(
        &self,
        event_count: usize,
    ) -> ResourceResult<ResourceEventPublishPermit> {
        let mut state = self.hub.lock_state();
        if event_count == 0 {
            return Ok(ResourceEventPublishPermit {
                first_sequence: None,
                successor_sequence: state.next_sequence,
                event_count,
            });
        }
        let Some(first_sequence) = state.next_sequence else {
            return Err(reject_sequence_exhaustion(&mut state, event_count));
        };
        let Ok(final_offset) = u64::try_from(event_count - 1) else {
            return Err(reject_sequence_exhaustion(&mut state, event_count));
        };
        let Some(final_sequence) = first_sequence.checked_add(final_offset) else {
            return Err(reject_sequence_exhaustion(&mut state, event_count));
        };
        Ok(ResourceEventPublishPermit {
            first_sequence: Some(first_sequence),
            successor_sequence: final_sequence.checked_add(1),
            event_count,
        })
    }

    pub(crate) fn publish_permitted(
        &self,
        permit: ResourceEventPublishPermit,
        events: Vec<ResourceEvent>,
    ) {
        assert_eq!(permit.event_count, events.len());
        if events.is_empty() {
            return;
        }
        let first_sequence = permit
            .first_sequence
            .expect("a non-empty event publication has a first sequence");
        let mut state = self.hub.lock_state();
        assert_eq!(state.next_sequence, Some(first_sequence));
        let now = Instant::now();
        evict_expired(&mut state, now);
        for (offset, event) in events.into_iter().enumerate() {
            let sequence = first_sequence
                .checked_add(u64::try_from(offset).expect("event offset fits the reserved range"))
                .expect("event sequence fits the reserved range");
            publish_one(&mut state, event, sequence, now);
        }
        state.next_sequence = permit.successor_sequence;
        drop(state);
        self.hub.changed.notify_all();
    }

    #[cfg(test)]
    pub(crate) fn publish_for_test(&self, event: ResourceEvent) {
        self.try_publish_for_test(event)
            .expect("test event sequence has capacity");
    }

    #[cfg(test)]
    pub(crate) fn try_publish_for_test(&self, event: ResourceEvent) -> ResourceResult<()> {
        let permit = self.prepare_publish(1)?;
        self.publish_permitted(permit, vec![event]);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_next_sequence_for_test(&self, next_sequence: Option<u64>) {
        self.hub.lock_state().next_sequence = next_sequence;
    }

    #[cfg(test)]
    pub(crate) fn drop_all_events_for_test(&self) {
        let mut state = self.hub.lock_state();
        while state.entries.oldest_slot.is_some() {
            drop_oldest(&mut state);
        }
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
                .first()
                .map(|entry| now.saturating_duration_since(entry.published_at))
                .unwrap_or_default(),
            coalesced_count: state.coalesced_count,
            dropped_count: state.dropped_count,
            lagged_read_count: state.lagged_read_count,
            sequence_exhausted: state.next_sequence.is_none(),
            rejected_publish_count: state.rejected_publish_count,
        }
    }

    #[cfg(test)]
    pub(crate) fn poison_state(&self) {
        let _guard = self.hub.state.lock().unwrap();
        panic!("poison resource event stream");
    }
}

fn reject_sequence_exhaustion(
    state: &mut ResourceEventLogState,
    requested_event_count: usize,
) -> ResourceRegistryError {
    state.rejected_publish_count = state.rejected_publish_count.saturating_add(1);
    ResourceRegistryError::EventSequenceExhausted {
        requested_event_count,
    }
}

fn publish_one(
    state: &mut ResourceEventLogState,
    event: ResourceEvent,
    sequence: u64,
    now: Instant,
) {
    let identity = event_identity(&event);
    let mut reusable_identity_slot = None;
    if is_coalescable(event.kind) {
        let latest_slot = state.entries.newest_slot().filter(|slot| {
            state
                .entries
                .get(*slot)
                .is_some_and(|entry| event_identity(&entry.event) == identity)
        });
        if let Some(slot) = latest_slot.or_else(|| indexed_slot(&state, identity)) {
            if state
                .entries
                .get(slot)
                .is_some_and(|entry| is_coalescable(entry.event.kind))
            {
                if let Some(previous) = state.entries.remove(slot) {
                    if state.latest_slot_by_identity.get(&identity) == Some(&slot) {
                        reusable_identity_slot = Some(slot);
                    }
                    state.approximate_bytes = state
                        .approximate_bytes
                        .saturating_sub(previous.approximate_bytes);
                    state.coalesced_count = state.coalesced_count.saturating_add(1);
                }
            }
        }
    }
    let approximate_bytes = approximate_event_bytes(&event);
    state.approximate_bytes = state.approximate_bytes.saturating_add(approximate_bytes);
    let slot = state.entries.insert_back(LoggedResourceEvent {
        sequence,
        published_at: now,
        approximate_bytes,
        event,
    });
    if reusable_identity_slot != Some(slot) {
        state.latest_slot_by_identity.insert(identity, slot);
    }
    enforce_capacity(state);
}

#[derive(Debug)]
/// 一个独立订阅游标。共享同一接收端会竞争同一游标；阻塞读取持有游标锁直到返回，
/// 同一接收端的并发读取会串行等待。需要独立消费进度时应分别订阅。
pub struct ResourceEventReceiver {
    hub: Arc<ResourceEventHub>,
    publisher_lifetime: Weak<ResourceEventPublisherLifetime>,
    cursor: Mutex<Option<u64>>,
}

impl ResourceEventReceiver {
    /// 当前游标之后仍保留的事件数，仅供调度参考；不会执行时限淘汰，也不代表事件连续无缺口。
    pub fn len(&self) -> usize {
        let cursor = lock_cursor(&self.cursor);
        let state = self.hub.lock_state();
        cursor
            .map(|cursor| state.entries.count_at_or_after(cursor))
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 非阻塞读取下一条事件；缺口应触发全量同步，断开或序号耗尽则需要重新建立发布关系。
    pub fn try_recv(&self) -> Result<ResourceEvent, ResourceEventTryRecvError> {
        let mut cursor = lock_cursor(&self.cursor);
        let mut state = self.hub.lock_state();
        match take_next(&mut state, &mut cursor) {
            EventRead::Event(event) => Ok(event),
            EventRead::Lagged(gap) => Err(ResourceEventTryRecvError::Lagged(gap)),
            EventRead::SequenceExhausted => Err(ResourceEventTryRecvError::SequenceExhausted),
            EventRead::Empty if self.is_disconnected() => {
                Err(ResourceEventTryRecvError::Disconnected)
            }
            EventRead::Empty => Err(ResourceEventTryRecvError::Empty),
        }
    }

    /// 等待事件或终止状态；等待期间独占此接收端的游标，适合一个消费者负责一条订阅。
    pub fn recv(&self) -> Result<ResourceEvent, ResourceEventRecvError> {
        let mut cursor = lock_cursor(&self.cursor);
        let mut state = self.hub.lock_state();
        loop {
            match take_next(&mut state, &mut cursor) {
                EventRead::Event(event) => return Ok(event),
                EventRead::Lagged(gap) => return Err(ResourceEventRecvError::Lagged(gap)),
                EventRead::SequenceExhausted => {
                    return Err(ResourceEventRecvError::SequenceExhausted);
                }
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

    /// 等待上限作用于日志的条件变量等待；并发调用争用游标锁或日志锁时仍需先等待锁。
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
                EventRead::SequenceExhausted => {
                    return Err(ResourceEventRecvTimeoutError::SequenceExhausted);
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
        // Upgrading while the event-state lock is held can make this receiver the final
        // lifetime owner; dropping that temporary Arc would then re-lock the same mutex.
        self.publisher_lifetime.strong_count() == 0
    }
}

fn lock_cursor(cursor: &Mutex<Option<u64>>) -> MutexGuard<'_, Option<u64>> {
    cursor
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

enum EventRead {
    Event(ResourceEvent),
    Empty,
    Lagged(ResourceEventGap),
    SequenceExhausted,
}

fn take_next(state: &mut ResourceEventLogState, cursor: &mut Option<u64>) -> EventRead {
    evict_expired(state, Instant::now());
    let Some(expected_sequence) = *cursor else {
        return EventRead::SequenceExhausted;
    };
    let Some((next_sequence, next_event)) = state
        .entries
        .first_at_or_after(expected_sequence)
        .map(|entry| (entry.sequence, entry.event.clone()))
    else {
        if *cursor != state.next_sequence {
            let gap = ResourceEventGap {
                expected_sequence,
                oldest_available_sequence: state.next_sequence,
            };
            *cursor = state.next_sequence;
            state.lagged_read_count = state.lagged_read_count.saturating_add(1);
            return EventRead::Lagged(gap);
        }
        return EventRead::Empty;
    };
    if next_sequence > expected_sequence {
        let gap = ResourceEventGap {
            expected_sequence,
            oldest_available_sequence: Some(next_sequence),
        };
        *cursor = Some(next_sequence);
        state.lagged_read_count = state.lagged_read_count.saturating_add(1);
        return EventRead::Lagged(gap);
    }
    *cursor = expected_sequence.checked_add(1);
    EventRead::Event(next_event)
}

fn event_identity(event: &ResourceEvent) -> ResourceEventIdentity {
    (event.resource_kind, event.id)
}

fn indexed_slot(state: &ResourceEventLogState, identity: ResourceEventIdentity) -> Option<usize> {
    let slot = state.latest_slot_by_identity.get(&identity).copied()?;
    let indexed = state.entries.get(slot)?;
    (event_identity(&indexed.event) == identity).then_some(slot)
}

fn remove_current_identity_mapping(
    state: &mut ResourceEventLogState,
    removed_slot: usize,
    removed: &LoggedResourceEvent,
) {
    let identity = event_identity(&removed.event);
    if let Entry::Occupied(mapping) = state.latest_slot_by_identity.entry(identity) {
        if *mapping.get() == removed_slot {
            mapping.remove();
        }
    }
}

fn is_coalescable(kind: ResourceEventKind) -> bool {
    matches!(kind, ResourceEventKind::Added | ResourceEventKind::Updated)
}

/// 估算保留日志的容量成本，计入字段与定位符文本长度；不追踪分配器开销或字符串预留容量。
pub fn approximate_event_bytes(event: &ResourceEvent) -> usize {
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
    while state.entries.first().is_some_and(|entry| {
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
    if let Some((slot, removed)) = state.entries.pop_front() {
        remove_current_identity_mapping(state, slot, &removed);
        state.approximate_bytes = state
            .approximate_bytes
            .saturating_sub(removed.approximate_bytes);
        state.dropped_count = state.dropped_count.saturating_add(1);
    }
}

#[cfg(test)]
#[path = "event_stream/tests/publication_index_tests.rs"]
mod publication_index_tests;

#[cfg(test)]
#[path = "event_stream/tests/event_order_tests.rs"]
mod event_order_tests;
