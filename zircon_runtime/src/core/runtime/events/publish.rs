use crate::core::framework::events::{EngineEvent, EventBusDiagnosticsSnapshot};
use std::sync::Arc;

use super::EventBus;
use super::subscriber::EventDeliveryStatus;
use super::topic::{EventBusState, EventTopic};

#[derive(Default)]
struct DisconnectedSubscriberIds {
    first: Option<u64>,
    additional: Vec<u64>,
}

impl DisconnectedSubscriberIds {
    fn push(&mut self, subscriber_id: u64) {
        if self.first.is_none() {
            self.first = Some(subscriber_id);
        } else {
            self.additional.push(subscriber_id);
        }
    }

    fn remove_from(mut self, topic: &EventTopic) -> bool {
        let Some(first) = self.first else {
            return false;
        };
        if self.additional.is_empty() {
            return topic.remove_subscribers_while_delivery_locked(std::slice::from_ref(&first));
        }
        self.additional.push(first);
        topic.remove_subscribers_while_delivery_locked(&self.additional)
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        usize::from(self.first.is_some()) + self.additional.len()
    }
}

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
            let mut disconnected_ids = DisconnectedSubscriberIds::default();
            let subscribers = topic.snapshot_subscribers();
            if subscribers.is_empty() {
                self.diagnostics.record_publish_duration(started);
                return;
            }
            let event = Arc::new(event);
            if let Some((last_subscriber, preceding_subscribers)) = subscribers.split_last() {
                for subscriber in preceding_subscribers {
                    if matches!(
                        subscriber.deliver(Arc::clone(&event)),
                        EventDeliveryStatus::Disconnected
                    ) {
                        disconnected_ids.push(subscriber.id());
                    }
                }
                if matches!(
                    last_subscriber.deliver(event),
                    EventDeliveryStatus::Disconnected
                ) {
                    disconnected_ids.push(last_subscriber.id());
                }
            }
            disconnected_ids.remove_from(&topic)
        };

        if removed_subscribers {
            self.remove_topic_if_empty(&topic);
        }
        self.diagnostics.record_publish_duration(started);
    }
}

#[cfg(test)]
#[path = "publish/single_disconnect_inline_tests.rs"]
mod single_disconnect_inline_tests;
