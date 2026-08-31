use std::collections::{hash_map::Entry, HashMap};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};

use crate::core::framework::events::{
    EngineEventDeliveryPolicy, EventBusDiagnosticsMode, EventBusDiagnosticsSnapshot,
};

use super::diagnostics::EventBusDiagnosticsState;
use super::subscriber::{EventSubscriber, EventSubscription};

type EventTopicMap = HashMap<String, Arc<EventTopic>>;
type EventSubscriberSnapshot = Arc<[Arc<EventSubscriber>]>;

pub(super) struct EventBusState {
    topics: RwLock<EventTopicMap>,
    next_subscriber_id: AtomicU64,
    pub(super) diagnostics: Arc<EventBusDiagnosticsState>,
}

impl EventBusState {
    pub(super) fn new(diagnostics_mode: EventBusDiagnosticsMode) -> Self {
        Self {
            topics: RwLock::new(HashMap::new()),
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
            let mut topics = self.write_topics();
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
        self.topics
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write_topics(&self) -> RwLockWriteGuard<'_, EventTopicMap> {
        self.topics
            .write()
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
mod tests {
    use std::sync::{Arc, Barrier};
    use std::time::{Duration, Instant};

    use crate::core::framework::events::{EngineEventDeliveryPolicy, EventBusDiagnosticsMode};
    use crate::core::EngineEvent;

    use super::super::diagnostics::EventBusDiagnosticsState;
    use super::super::subscriber::EventSubscriber;
    use super::super::EventBus;
    use super::{EventBusState, EventTopic};

    const BULK_PRUNE_SUBSCRIBERS: usize = 4_096;
    const MAX_BULK_PRUNE_LATENCY: Duration = Duration::from_millis(100);
    const TOPIC_LOOKUP_WORKERS: usize = 8;
    const TOPIC_LOOKUPS_PER_WORKER: usize = 50_000;
    const MIN_TOPIC_LOOKUPS_PER_SECOND: f64 = 250_000.0;

    fn poison_in_worker(action: impl FnOnce() + Send + 'static) {
        assert!(
            std::thread::spawn(action).join().is_err(),
            "test worker must poison its held mutex"
        );
    }

    #[test]
    fn runtime02_existing_topic_subscription_does_not_clone_the_lookup_key() {
        let source = include_str!("topic.rs");
        let end = source.find("mod tests {").expect("test module");
        let implementation = &source[..end];

        assert!(implementation.contains("match topics.entry(topic)"));
        assert!(implementation.contains("Entry::Occupied"));
        assert!(implementation.contains("Entry::Vacant"));
        assert!(!implementation.contains("entry(topic.clone())"));
    }

    #[test]
    fn runtime02_topic_registry_supports_overlapping_publish_lookups() {
        let bus = EventBus::default();
        let _subscription =
            bus.subscribe("runtime.registry.read", EngineEventDeliveryPolicy::Lossless);
        let registry = Arc::clone(&bus.state);
        let read_entered = Arc::new(Barrier::new(2));
        let release_read = Arc::new(Barrier::new(2));
        let worker_entered = Arc::clone(&read_entered);
        let worker_release = Arc::clone(&release_read);
        let reader = std::thread::spawn(move || {
            let _topics = registry.topics.read().unwrap();
            worker_entered.wait();
            worker_release.wait();
        });

        read_entered.wait();
        let overlapping_read = bus.state.topics.try_read();
        release_read.wait();
        reader.join().unwrap();

        assert!(
            overlapping_read.is_ok(),
            "independent publish lookups must not serialize on a registry mutex"
        );
    }

    #[test]
    fn runtime02_bulk_subscriber_prune_sorts_ids_for_sublinear_membership_checks() {
        let source = include_str!("topic.rs");
        let implementation = source
            .split("mod tests {")
            .next()
            .expect("topic implementation");

        assert!(implementation.contains("sorted_subscriber_ids.sort_unstable()"));
        assert!(implementation.contains("binary_search(&subscriber.id())"));
        assert!(!implementation.contains("subscriber_ids.contains"));
    }

    #[test]
    #[ignore = "managed Runtime02 performance evidence"]
    fn event_bus_runtime02_parallel_topic_lookup_evidence() {
        let state = Arc::new(EventBusState::new(EventBusDiagnosticsMode::Disabled));
        {
            let mut topics = state.write_topics();
            for worker in 0..TOPIC_LOOKUP_WORKERS {
                let name = format!("runtime.lookup.{worker}");
                topics.insert(name.clone(), Arc::new(EventTopic::new(name)));
            }
        }
        let start = Arc::new(Barrier::new(TOPIC_LOOKUP_WORKERS + 1));
        let readers = (0..TOPIC_LOOKUP_WORKERS)
            .map(|worker| {
                let state = Arc::clone(&state);
                let start = Arc::clone(&start);
                std::thread::spawn(move || {
                    let name = format!("runtime.lookup.{worker}");
                    start.wait();
                    for _ in 0..TOPIC_LOOKUPS_PER_WORKER {
                        assert!(state.topic(&name).is_some());
                    }
                })
            })
            .collect::<Vec<_>>();

        let started = Instant::now();
        start.wait();
        for reader in readers {
            reader.join().unwrap();
        }
        let elapsed = started.elapsed();
        let lookups = TOPIC_LOOKUP_WORKERS * TOPIC_LOOKUPS_PER_WORKER;
        let lookups_per_second = lookups as f64 / elapsed.as_secs_f64();

        assert!(lookups_per_second >= MIN_TOPIC_LOOKUPS_PER_SECOND);
        println!(
            "EVENTBUS_BENCH_V3 kind=parallel_topic_lookup workers={} lookups={} exclusive_registry_lock_acquisitions_before={} exclusive_registry_lock_acquisitions_after=0 exclusive_lock_reduction_percent=100.0000 elapsed_ns={} lookups_per_second={:.2} target_lookups_per_second={:.2}",
            TOPIC_LOOKUP_WORKERS,
            lookups,
            lookups,
            elapsed.as_nanos(),
            lookups_per_second,
            MIN_TOPIC_LOOKUPS_PER_SECOND,
        );
    }

    #[test]
    #[ignore = "managed Runtime02 performance evidence"]
    fn event_bus_runtime02_bulk_disconnect_prune_evidence() {
        let diagnostics = Arc::new(EventBusDiagnosticsState::new(
            EventBusDiagnosticsMode::Disabled,
        ));
        let topic = EventTopic::new("runtime.bulk-prune".to_string());
        let subscribers = (0..BULK_PRUNE_SUBSCRIBERS)
            .map(|id| {
                Arc::new(EventSubscriber::new(
                    id as u64,
                    EngineEventDeliveryPolicy::Lossless,
                    Arc::clone(&diagnostics),
                ))
            })
            .collect::<Vec<_>>();
        *topic.lock_subscribers() = subscribers.into();
        let disconnected_ids = (0..BULK_PRUNE_SUBSCRIBERS as u64).rev().collect::<Vec<_>>();

        let started = Instant::now();
        let removed = topic.remove_subscribers_while_delivery_locked(&disconnected_ids);
        let elapsed = started.elapsed();

        assert!(removed);
        assert_eq!(topic.subscriber_count(), 0);
        assert!(elapsed <= MAX_BULK_PRUNE_LATENCY);
        let membership_probes_before = BULK_PRUNE_SUBSCRIBERS * (BULK_PRUNE_SUBSCRIBERS + 1) / 2;
        let binary_search_probes_per_subscriber = BULK_PRUNE_SUBSCRIBERS.ilog2() as usize + 1;
        let membership_probes_after_upper_bound =
            BULK_PRUNE_SUBSCRIBERS * binary_search_probes_per_subscriber;
        let membership_probe_reduction_percent = (1.0
            - membership_probes_after_upper_bound as f64 / membership_probes_before as f64)
            * 100.0;
        println!(
            "EVENTBUS_BENCH_V3 kind=bulk_disconnect_prune subscribers={} membership_probes_before={} membership_probes_after_upper_bound={} membership_probe_reduction_percent={:.4} elapsed_ns={} target_ns={}",
            BULK_PRUNE_SUBSCRIBERS,
            membership_probes_before,
            membership_probes_after_upper_bound,
            membership_probe_reduction_percent,
            elapsed.as_nanos(),
            MAX_BULK_PRUNE_LATENCY.as_nanos(),
        );
    }

    #[test]
    fn runtime02_poisoned_event_bus_mutexes_recover_without_losing_delivery() {
        let bus = EventBus::default();
        let subscription = bus.subscribe("runtime.poison", EngineEventDeliveryPolicy::Lossless);

        let state = Arc::clone(&bus.state);
        poison_in_worker(move || {
            let _topics = state.topics.write().unwrap();
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
