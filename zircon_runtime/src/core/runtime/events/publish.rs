use crate::core::framework::events::{EngineEvent, EventBusDiagnosticsSnapshot};
use std::sync::Arc;

use super::subscriber::EventDeliveryStatus;
use super::topic::EventBusState;
use super::EventBus;

impl EventBus {
    pub fn publish(&self, event: EngineEvent) {
        self.state.publish(event);
    }

    pub fn diagnostic_report(&self) -> EventBusDiagnosticsSnapshot {
        self.state.diagnostic_report()
    }
}

impl EventBusState {
    pub(super) fn publish(&self, event: EngineEvent) {
        let started = self.diagnostics.record_published_and_capture_time();

        let Some(topic) = self.topic(&event.topic) else {
            self.diagnostics.record_publish_duration(started);
            return;
        };
        let event = Arc::new(event);
        let removed_subscribers = {
            let _delivery = if let Some(delivery) = topic.try_lock_delivery() {
                delivery
            } else {
                let wait_started = self.diagnostics.capture_contention_time();
                self.diagnostics.record_publisher_waiting();
                let delivery = topic.lock_delivery();
                self.diagnostics.record_publisher_resumed(wait_started);
                delivery
            };
            let mut disconnected_ids: Option<Vec<u64>> = None;
            let subscribers = topic.snapshot_subscribers();
            if let Some((last_subscriber, preceding_subscribers)) = subscribers.split_last() {
                for subscriber in preceding_subscribers {
                    if matches!(
                        subscriber.deliver(Arc::clone(&event)),
                        EventDeliveryStatus::Disconnected
                    ) {
                        disconnected_ids
                            .get_or_insert_with(Vec::new)
                            .push(subscriber.id());
                    }
                }
                if matches!(
                    last_subscriber.deliver(event),
                    EventDeliveryStatus::Disconnected
                ) {
                    disconnected_ids
                        .get_or_insert_with(Vec::new)
                        .push(last_subscriber.id());
                }
            }
            disconnected_ids.is_some_and(|ids| topic.remove_subscribers_while_delivery_locked(&ids))
        };

        if removed_subscribers {
            self.remove_topic_if_empty(&topic);
        }
        self.diagnostics.record_publish_duration(started);
    }
}
