use std::collections::{hash_map::Entry, HashMap};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};

use crate::core::framework::events::{
    EngineEventDeliveryPolicy, EngineEventSubscribeError, EventBusDiagnosticsMode,
    EventBusDiagnosticsSnapshot, EventBusLimits,
};

use super::admission::SharedAdmission;
use super::diagnostics::EventBusDiagnosticsState;
use super::subscriber::{EventSubscriber, EventSubscription};

type EventTopicMap = HashMap<String, Arc<EventTopic>>;
type EventSubscriberSnapshot = Arc<[Arc<EventSubscriber>]>;

pub(super) struct EventBusState {
    topics: RwLock<EventTopicMap>,
    pub(super) admission: Arc<SharedAdmission>,
    next_subscriber_id: AtomicU64,
    pub(super) diagnostics: Arc<EventBusDiagnosticsState>,
}

impl EventBusState {
    pub(super) fn new(diagnostics_mode: EventBusDiagnosticsMode, limits: EventBusLimits) -> Self {
        Self {
            topics: RwLock::new(HashMap::new()),
            admission: Arc::new(SharedAdmission::new(limits)),
            next_subscriber_id: AtomicU64::new(0),
            diagnostics: Arc::new(EventBusDiagnosticsState::new(diagnostics_mode)),
        }
    }

    pub(super) fn subscribe(
        self: &Arc<Self>,
        topic: String,
        policy: EngineEventDeliveryPolicy,
    ) -> Result<EventSubscription, EngineEventSubscribeError> {
        self.subscribe_after_reservation(topic, policy, || {})
    }

    #[cfg(test)]
    pub(super) fn subscribe_after_reservation_for_test(
        self: &Arc<Self>,
        topic: String,
        policy: EngineEventDeliveryPolicy,
        after_reservation: impl FnOnce(),
    ) -> Result<EventSubscription, EngineEventSubscribeError> {
        self.subscribe_after_reservation(topic, policy, after_reservation)
    }

    fn subscribe_after_reservation(
        self: &Arc<Self>,
        topic: String,
        policy: EngineEventDeliveryPolicy,
        after_reservation: impl FnOnce(),
    ) -> Result<EventSubscription, EngineEventSubscribeError> {
        let topic = topic.into_boxed_str().into_string();
        let id = self
            .next_subscriber_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| EngineEventSubscribeError::IdentifierExhausted)?;
        self.admission.register(id, &topic)?;
        let subscriber = Arc::new(EventSubscriber::new(
            id,
            policy,
            Arc::clone(&self.diagnostics),
            Arc::clone(&self.admission),
        ));
        let max_topics = self.admission.lock().limits.max_topics.get();
        let (topic, reservation) = {
            let mut topics = self.write_topics();
            if !topics.contains_key(&topic) && topics.len() >= max_topics {
                drop(topics);
                self.admission.unregister(id);
                return Err(EngineEventSubscribeError::RegistryFull);
            }
            let topic = Arc::clone(match topics.entry(topic) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    let topic = Arc::new(EventTopic::new(entry.key().clone()));
                    entry.insert(topic)
                }
            });
            // 预留计数保护锁外的加入窗口；新订阅者尚未入快照，期间事件不会补发。
            let reservation = topic.reserve_subscription();
            (topic, reservation)
        };
        after_reservation();
        let added = topic.add_subscriber(Arc::clone(&subscriber));
        drop(reservation);
        if let Err(error) = added {
            self.admission.unregister(id);
            self.remove_topic_if_empty(&topic);
            return Err(error);
        }
        Ok(EventSubscription::new(Arc::clone(self), topic, subscriber))
    }

    pub(super) fn snapshot_topics(&self) -> Vec<Arc<EventTopic>> {
        self.read_topics().values().cloned().collect()
    }

    pub(super) fn topic(&self, name: &str) -> Option<Arc<EventTopic>> {
        self.read_topics().get(name).cloned()
    }

    pub(super) fn diagnostic_report(&self) -> EventBusDiagnosticsSnapshot {
        let topics = self.read_topics();
        let subscriber_count = topics.values().map(|topic| topic.subscriber_count()).sum();
        self.diagnostics.snapshot(topics.len(), subscriber_count)
    }

    pub(super) fn remove_topic_if_empty(&self, topic: &Arc<EventTopic>) {
        let mut topics = self.write_topics();
        if topic.is_removable()
            && topics
                .get(topic.name())
                .is_some_and(|current| Arc::ptr_eq(current, topic))
        {
            topics.remove(topic.name());
        }
    }

    fn read_topics(&self) -> RwLockReadGuard<'_, EventTopicMap> {
        self.topics.read().unwrap_or_else(|poisoned| {
            self.admission.fail_closed();
            poisoned.into_inner()
        })
    }

    fn write_topics(&self) -> RwLockWriteGuard<'_, EventTopicMap> {
        self.topics.write().unwrap_or_else(|poisoned| {
            self.admission.fail_closed();
            poisoned.into_inner()
        })
    }
}

impl Drop for EventBusState {
    fn drop(&mut self) {
        self.admission.lock().closed = true;
        let topics = self
            .topics
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut drained = Vec::new();
        for topic in topics.values() {
            let _delivery = topic.lock_delivery();
            for subscriber in topic.snapshot_subscribers().iter() {
                if let Some(queue) = subscriber.deactivate() {
                    drained.push((subscriber.clone(), queue));
                }
            }
        }
        for (subscriber, queue) in drained {
            subscriber.record_deactivated_queue(queue);
        }
    }
}

pub(super) struct EventTopic {
    name: String,
    subscribers: Mutex<EventSubscriberSnapshot>,
    delivery: Mutex<()>,
    pending_subscriptions: AtomicUsize,
}

impl EventTopic {
    fn new(name: String) -> Self {
        Self {
            name,
            subscribers: Mutex::new(Arc::from([])),
            delivery: Mutex::new(()),
            pending_subscriptions: AtomicUsize::new(0),
        }
    }

    pub(super) fn delivery_is_poisoned(&self) -> bool {
        self.delivery.is_poisoned() || self.subscribers.is_poisoned()
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn add_subscriber(
        &self,
        subscriber: Arc<EventSubscriber>,
    ) -> Result<(), EngineEventSubscribeError> {
        let _delivery = self.lock_delivery();
        if self.delivery_is_poisoned() {
            subscriber.admission.fail_closed();
            return Err(EngineEventSubscribeError::Poisoned);
        }
        let ledger = subscriber.admission.lock();
        if ledger.poisoned {
            return Err(EngineEventSubscribeError::Poisoned);
        }
        if ledger.closed {
            return Err(EngineEventSubscribeError::Closed);
        }
        drop(ledger);
        let mut subscribers = self.lock_subscribers();
        let mut updated = Vec::with_capacity(subscribers.len() + 1);
        updated.extend(subscribers.iter().cloned());
        updated.push(subscriber);
        updated.sort_unstable_by_key(|subscriber| subscriber.id());
        *subscribers = updated.into();
        Ok(())
    }

    fn reserve_subscription(self: &Arc<Self>) -> PendingSubscription {
        self.pending_subscriptions.fetch_add(1, Ordering::AcqRel);
        PendingSubscription {
            topic: Arc::clone(self),
        }
    }

    pub(super) fn remove_subscribers_while_delivery_locked(&self, subscriber_ids: &[u64]) -> bool {
        if let [subscriber_id] = subscriber_ids {
            let mut subscribers = self.lock_subscribers();
            if !subscribers
                .iter()
                .any(|subscriber| subscriber.id() == *subscriber_id)
            {
                return false;
            }
            let mut retained = Vec::with_capacity(subscribers.len() - 1);
            retained.extend(
                subscribers
                    .iter()
                    .filter(|subscriber| subscriber.id() != *subscriber_id)
                    .cloned(),
            );
            *subscribers = retained.into();
            return true;
        }
        if subscriber_ids.is_empty() {
            return false;
        }

        let mut sorted_subscriber_ids = subscriber_ids.to_vec();
        sorted_subscriber_ids.sort_unstable();
        let mut subscribers = self.lock_subscribers();
        if !subscribers.iter().any(|subscriber| {
            sorted_subscriber_ids
                .binary_search(&subscriber.id())
                .is_ok()
        }) {
            return false;
        }

        let mut retained = Vec::with_capacity(subscribers.len());
        retained.extend(
            subscribers
                .iter()
                .filter(|subscriber| {
                    sorted_subscriber_ids
                        .binary_search(&subscriber.id())
                        .is_err()
                })
                .cloned(),
        );
        *subscribers = retained.into();
        true
    }

    pub(super) fn snapshot_subscribers(&self) -> EventSubscriberSnapshot {
        Arc::clone(&self.lock_subscribers())
    }

    pub(super) fn subscriber_count(&self) -> usize {
        self.lock_subscribers().len()
    }

    pub(super) fn is_removable(&self) -> bool {
        self.pending_subscriptions.load(Ordering::Acquire) == 0
            && self.lock_subscribers().is_empty()
    }

    pub(super) fn lock_delivery(&self) -> MutexGuard<'_, ()> {
        self.delivery
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn try_lock_delivery(&self) -> Option<MutexGuard<'_, ()>> {
        match self.delivery.try_lock() {
            Ok(delivery) => Some(delivery),
            Err(TryLockError::Poisoned(poisoned)) => Some(poisoned.into_inner()),
            Err(TryLockError::WouldBlock) => None,
        }
    }

    pub(super) fn lock_subscribers(&self) -> MutexGuard<'_, EventSubscriberSnapshot> {
        self.subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

struct PendingSubscription {
    topic: Arc<EventTopic>,
}

impl Drop for PendingSubscription {
    fn drop(&mut self) {
        self.topic
            .pending_subscriptions
            .fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
#[path = "tests/topic.rs"]
mod tests;
