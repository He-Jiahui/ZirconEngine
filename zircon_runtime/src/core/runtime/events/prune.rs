use std::sync::Arc;

use super::subscriber::EventSubscriber;
use super::topic::{EventBusState, EventTopic};

impl EventBusState {
    pub(super) fn unsubscribe(&self, topic: &Arc<EventTopic>, subscriber: &Arc<EventSubscriber>) {
        let (deactivated_queue, removed) = {
            let _delivery = topic.lock_delivery();
            let deactivated_queue = subscriber.deactivate();
            let removed = topic.remove_subscribers_while_delivery_locked(&[subscriber.id()]);
            drop(_delivery);
            (deactivated_queue, removed)
        };
        if removed {
            self.remove_topic_if_empty(topic);
        }
        if let Some(queued) = deactivated_queue {
            // Queue-age accounting can walk a large lossless queue. Keep it
            // out of the topic delivery critical section so publishers are
            // blocked only for the state transition and snapshot removal.
            subscriber.record_deactivated_queue(queued);
        }
    }
}

#[cfg(test)]
#[path = "tests/prune.rs"]
mod tests;
