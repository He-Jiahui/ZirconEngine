use std::sync::Arc;

use super::subscriber::EventSubscriber;
use super::topic::{EventBusState, EventTopic};

impl EventBusState {
    pub(super) fn unsubscribe(&self, topic: &Arc<EventTopic>, subscriber: &Arc<EventSubscriber>) {
        let _delivery = topic.lock_delivery();
        subscriber.deactivate_and_drain();
        let removed = topic.remove_subscribers_while_delivery_locked(&[subscriber.id()]);
        drop(_delivery);
        if removed {
            self.remove_topic_if_empty(topic);
        }
    }
}
