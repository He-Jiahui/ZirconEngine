use std::collections::hash_map::Entry;
use std::sync::Arc;

use crossbeam_channel::unbounded;

use crate::core::framework::channel::{ChannelReceiver, ChannelSender};
use crate::core::framework::events::EngineEvent;

use super::EventBus;

impl EventBus {
    pub fn subscribe(&self, topic: impl Into<String>) -> ChannelReceiver<EngineEvent> {
        let topic = topic.into();
        let (tx, rx) = unbounded();
        let mut subscribers = self.subscribers.lock().unwrap();
        match subscribers.entry(topic) {
            Entry::Vacant(entry) => {
                entry.insert(Arc::<[ChannelSender<EngineEvent>]>::from([tx]));
            }
            Entry::Occupied(mut entry) => {
                let topic_subscribers = entry.get_mut();
                let updated_subscribers = match topic_subscribers.as_ref() {
                    [] => Arc::<[ChannelSender<EngineEvent>]>::from([tx]),
                    [subscriber] => {
                        Arc::<[ChannelSender<EngineEvent>]>::from([subscriber.clone(), tx])
                    }
                    [first_subscriber, second_subscriber] => {
                        Arc::<[ChannelSender<EngineEvent>]>::from([
                            first_subscriber.clone(),
                            second_subscriber.clone(),
                            tx,
                        ])
                    }
                    [first_subscriber, second_subscriber, third_subscriber] => {
                        Arc::<[ChannelSender<EngineEvent>]>::from([
                            first_subscriber.clone(),
                            second_subscriber.clone(),
                            third_subscriber.clone(),
                            tx,
                        ])
                    }
                    [first_subscriber, second_subscriber, third_subscriber, fourth_subscriber] => {
                        Arc::<[ChannelSender<EngineEvent>]>::from([
                            first_subscriber.clone(),
                            second_subscriber.clone(),
                            third_subscriber.clone(),
                            fourth_subscriber.clone(),
                            tx,
                        ])
                    }
                    current_subscribers => {
                        let mut updated_subscribers =
                            Vec::with_capacity(current_subscribers.len() + 1);
                        updated_subscribers.extend(current_subscribers.iter().cloned());
                        updated_subscribers.push(tx);
                        updated_subscribers.into()
                    }
                };
                *topic_subscribers = updated_subscribers;
            }
        }
        rx
    }
}
