use std::collections::{hash_map::Entry, HashMap};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

use crate::core::framework::events::{
    EngineEventDeliveryPolicy, EventBusDiagnosticsMode, EventBusDiagnosticsSnapshot,
};

use super::diagnostics::EventBusDiagnosticsState;
use super::subscriber::{EventSubscriber, EventSubscription};

type EventTopicMap = HashMap<String, Arc<EventTopic>>;
type EventSubscriberSnapshot = Arc<[Arc<EventSubscriber>]>;

pub(super) struct EventBusState {
    topics: Mutex<EventTopicMap>,
    next_subscriber_id: AtomicU64,
    pub(super) diagnostics: Arc<EventBusDiagnosticsState>,
}

impl EventBusState {
    pub(super) fn new(diagnostics_mode: EventBusDiagnosticsMode) -> Self {
        Self {
            topics: Mutex::new(HashMap::new()),
            next_subscriber_id: AtomicU64::new(0),
            diagnostics: Arc::new(EventBusDiagnosticsState::new(diagnostics_mode)),
        }
    }

    pub(super) fn subscribe(
        self: &Arc<Self>,
        topic: String,
        policy: EngineEventDeliveryPolicy,
    ) -> EventSubscription {
        self.subscribe_after_reservation(topic, policy, || {})
    }

    #[cfg(test)]
    pub(super) fn subscribe_after_reservation_for_test(
        self: &Arc<Self>,
        topic: String,
        policy: EngineEventDeliveryPolicy,
        after_reservation: impl FnOnce(),
    ) -> EventSubscription {
        self.subscribe_after_reservation(topic, policy, after_reservation)
    }

    fn subscribe_after_reservation(
        self: &Arc<Self>,
        topic: String,
        policy: EngineEventDeliveryPolicy,
        after_reservation: impl FnOnce(),
    ) -> EventSubscription {
        let subscriber = Arc::new(EventSubscriber::new(
            self.next_subscriber_id.fetch_add(1, Ordering::Relaxed),
            policy,
            Arc::clone(&self.diagnostics),
        ));
        let (topic, reservation) = {
            let mut topics = self.lock_topics();
            let topic = Arc::clone(match topics.entry(topic) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    let topic = Arc::new(EventTopic::new(entry.key().clone()));
                    entry.insert(topic)
                }
            });
            let reservation = topic.reserve_subscription();
            (topic, reservation)
        };
        after_reservation();
        topic.add_subscriber(Arc::clone(&subscriber));
        drop(reservation);
        EventSubscription::new(Arc::clone(self), topic, subscriber)
    }

    pub(super) fn topic(&self, name: &str) -> Option<Arc<EventTopic>> {
        self.lock_topics().get(name).cloned()
    }

    pub(super) fn diagnostic_report(&self) -> EventBusDiagnosticsSnapshot {
        let topics = self.lock_topics();
        let subscriber_count = topics.values().map(|topic| topic.subscriber_count()).sum();
        self.diagnostics.snapshot(topics.len(), subscriber_count)
    }

    pub(super) fn remove_topic_if_empty(&self, topic: &Arc<EventTopic>) {
        let mut topics = self.lock_topics();
        if topic.is_removable()
            && topics
                .get(topic.name())
                .is_some_and(|current| Arc::ptr_eq(current, topic))
        {
            topics.remove(topic.name());
        }
    }

    fn lock_topics(&self) -> MutexGuard<'_, EventTopicMap> {
        self.topics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for EventBusState {
    fn drop(&mut self) {
        let topics = self
            .topics
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for topic in topics.values() {
            let _delivery = topic.lock_delivery();
            for subscriber in topic.snapshot_subscribers().iter() {
                subscriber.deactivate_and_drain();
            }
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

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn add_subscriber(&self, subscriber: Arc<EventSubscriber>) {
        let _delivery = self.lock_delivery();
        let mut subscribers = self.lock_subscribers();
        let mut updated = Vec::with_capacity(subscribers.len() + 1);
        updated.extend(subscribers.iter().cloned());
        updated.push(subscriber);
        *subscribers = updated.into();
    }

    fn reserve_subscription(self: &Arc<Self>) -> PendingSubscription {
        self.pending_subscriptions.fetch_add(1, Ordering::AcqRel);
        PendingSubscription {
            topic: Arc::clone(self),
        }
    }

    pub(super) fn remove_subscribers_while_delivery_locked(&self, subscriber_ids: &[u64]) -> bool {
        let mut subscribers = self.lock_subscribers();
        if !subscribers
            .iter()
            .any(|subscriber| subscriber_ids.contains(&subscriber.id()))
        {
            return false;
        }

        let mut retained = Vec::with_capacity(subscribers.len());
        retained.extend(
            subscribers
                .iter()
                .filter(|subscriber| !subscriber_ids.contains(&subscriber.id()))
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
mod tests {
    use std::sync::Arc;

    use crate::core::framework::events::EngineEventDeliveryPolicy;
    use crate::core::EngineEvent;

    use super::super::EventBus;

    fn poison_in_worker(action: impl FnOnce() + Send + 'static) {
        assert!(
            std::thread::spawn(action).join().is_err(),
            "test worker must poison its held mutex"
        );
    }

    #[test]
    fn existing_topic_subscription_does_not_clone_the_lookup_key() {
        let source = include_str!("topic.rs");
        let end = source.find("mod tests {").expect("test module");
        let implementation = &source[..end];

        assert!(implementation.contains("match topics.entry(topic)"));
        assert!(implementation.contains("Entry::Occupied"));
        assert!(implementation.contains("Entry::Vacant"));
        assert!(!implementation.contains("entry(topic.clone())"));
    }

    #[test]
    fn poisoned_event_bus_mutexes_recover_without_losing_delivery() {
        let bus = EventBus::default();
        let subscription = bus.subscribe("runtime.poison", EngineEventDeliveryPolicy::Lossless);

        let state = Arc::clone(&bus.state);
        poison_in_worker(move || {
            let _topics = state.topics.lock().unwrap();
            panic!("poison EventBus topic map");
        });

        let topic = bus
            .state
            .topic("runtime.poison")
            .expect("poison-safe topic map must retain the subscription");
        let topic_for_subscribers = Arc::clone(&topic);
        poison_in_worker(move || {
            let _subscribers = topic_for_subscribers.subscribers.lock().unwrap();
            panic!("poison EventBus subscriber snapshot");
        });
        let topic_for_delivery = Arc::clone(&topic);
        poison_in_worker(move || {
            let _delivery = topic_for_delivery.delivery.lock().unwrap();
            panic!("poison EventBus per-topic delivery lock");
        });
        let subscriber = Arc::clone(
            topic
                .snapshot_subscribers()
                .first()
                .expect("subscription must remain present after poison recovery"),
        );
        poison_in_worker(move || {
            subscriber.poison_queue_state_for_test();
        });

        bus.publish(EngineEvent {
            topic: "runtime.poison".to_string(),
            payload: serde_json::json!({ "recovered": true }),
        });

        assert_eq!(subscription.recv().unwrap().payload["recovered"], true);
        let report = bus.diagnostic_report();
        assert_eq!(report.topics, 1);
        assert_eq!(report.subscribers, 1);
        assert_eq!(report.published, 1);
        assert_eq!(report.delivered, 1);
    }
}
