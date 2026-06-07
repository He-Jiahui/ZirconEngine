use std::sync::Arc;

use crossbeam_channel::SendError;

use crate::core::types::ChannelSender;

use super::{EngineEvent, EventBus};

impl EventBus {
    pub fn publish(&self, event: EngineEvent) {
        let Some(subscribers) = self.snapshot_topic_subscribers(&event.topic) else {
            return;
        };
        let _delivery_guard = self.delivery_lock.lock().unwrap();

        match subscribers.as_ref() {
            [] => return,
            [subscriber] => {
                if let Err(SendError(failed_event)) = subscriber.send(event) {
                    self.prune_topic_subscribers(
                        &failed_event.topic,
                        std::slice::from_ref(subscriber),
                    );
                }
            }
            [first_subscriber, second_subscriber] => {
                let mut failed_topic: Option<String> = None;
                let mut first_failed_subscriber: Option<ChannelSender<EngineEvent>> = None;
                let mut failed_subscribers: Option<Vec<ChannelSender<EngineEvent>>> = None;
                let subscriber_count = 2;
                if let Err(SendError(failed_event)) = first_subscriber.send(event.clone()) {
                    Self::record_failed_subscriber(
                        failed_event,
                        first_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if let Err(SendError(failed_event)) = second_subscriber.send(event) {
                    Self::record_failed_subscriber(
                        failed_event,
                        second_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if failed_topic.is_some() {
                    self.prune_failed_publish_subscribers(
                        failed_topic,
                        first_failed_subscriber,
                        failed_subscribers,
                    );
                }
            }
            [first_subscriber, second_subscriber, third_subscriber] => {
                let mut failed_topic: Option<String> = None;
                let mut first_failed_subscriber: Option<ChannelSender<EngineEvent>> = None;
                let mut failed_subscribers: Option<Vec<ChannelSender<EngineEvent>>> = None;
                let subscriber_count = 3;
                if let Err(SendError(failed_event)) = first_subscriber.send(event.clone()) {
                    Self::record_failed_subscriber(
                        failed_event,
                        first_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if let Err(SendError(failed_event)) = second_subscriber.send(event.clone()) {
                    Self::record_failed_subscriber(
                        failed_event,
                        second_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if let Err(SendError(failed_event)) = third_subscriber.send(event) {
                    Self::record_failed_subscriber(
                        failed_event,
                        third_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if failed_topic.is_some() {
                    self.prune_failed_publish_subscribers(
                        failed_topic,
                        first_failed_subscriber,
                        failed_subscribers,
                    );
                }
            }
            [first_subscriber, second_subscriber, third_subscriber, fourth_subscriber] => {
                let mut failed_topic: Option<String> = None;
                let mut first_failed_subscriber: Option<ChannelSender<EngineEvent>> = None;
                let mut failed_subscribers: Option<Vec<ChannelSender<EngineEvent>>> = None;
                let subscriber_count = 4;
                if let Err(SendError(failed_event)) = first_subscriber.send(event.clone()) {
                    Self::record_failed_subscriber(
                        failed_event,
                        first_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if let Err(SendError(failed_event)) = second_subscriber.send(event.clone()) {
                    Self::record_failed_subscriber(
                        failed_event,
                        second_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if let Err(SendError(failed_event)) = third_subscriber.send(event.clone()) {
                    Self::record_failed_subscriber(
                        failed_event,
                        third_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if let Err(SendError(failed_event)) = fourth_subscriber.send(event) {
                    Self::record_failed_subscriber(
                        failed_event,
                        fourth_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if failed_topic.is_some() {
                    self.prune_failed_publish_subscribers(
                        failed_topic,
                        first_failed_subscriber,
                        failed_subscribers,
                    );
                }
            }
            [first_subscriber, second_subscriber, third_subscriber, fourth_subscriber, fifth_subscriber] =>
            {
                let mut failed_topic: Option<String> = None;
                let mut first_failed_subscriber: Option<ChannelSender<EngineEvent>> = None;
                let mut failed_subscribers: Option<Vec<ChannelSender<EngineEvent>>> = None;
                let subscriber_count = 5;
                if let Err(SendError(failed_event)) = first_subscriber.send(event.clone()) {
                    Self::record_failed_subscriber(
                        failed_event,
                        first_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if let Err(SendError(failed_event)) = second_subscriber.send(event.clone()) {
                    Self::record_failed_subscriber(
                        failed_event,
                        second_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if let Err(SendError(failed_event)) = third_subscriber.send(event.clone()) {
                    Self::record_failed_subscriber(
                        failed_event,
                        third_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if let Err(SendError(failed_event)) = fourth_subscriber.send(event.clone()) {
                    Self::record_failed_subscriber(
                        failed_event,
                        fourth_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if let Err(SendError(failed_event)) = fifth_subscriber.send(event) {
                    Self::record_failed_subscriber(
                        failed_event,
                        fifth_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }
                if failed_topic.is_some() {
                    self.prune_failed_publish_subscribers(
                        failed_topic,
                        first_failed_subscriber,
                        failed_subscribers,
                    );
                }
            }
            [leading_subscribers @ .., last_subscriber] => {
                let mut failed_topic: Option<String> = None;
                let mut first_failed_subscriber: Option<ChannelSender<EngineEvent>> = None;
                let mut failed_subscribers: Option<Vec<ChannelSender<EngineEvent>>> = None;
                let subscriber_count = leading_subscribers.len() + 1;
                for subscriber in leading_subscribers {
                    if let Err(SendError(failed_event)) = subscriber.send(event.clone()) {
                        Self::record_failed_subscriber(
                            failed_event,
                            subscriber,
                            subscriber_count,
                            &mut failed_topic,
                            &mut first_failed_subscriber,
                            &mut failed_subscribers,
                        );
                    }
                }
                if let Err(SendError(failed_event)) = last_subscriber.send(event) {
                    Self::record_failed_subscriber(
                        failed_event,
                        last_subscriber,
                        subscriber_count,
                        &mut failed_topic,
                        &mut first_failed_subscriber,
                        &mut failed_subscribers,
                    );
                }

                if failed_topic.is_some() {
                    self.prune_failed_publish_subscribers(
                        failed_topic,
                        first_failed_subscriber,
                        failed_subscribers,
                    );
                }
            }
        }
    }

    fn snapshot_topic_subscribers(&self, topic: &str) -> Option<Arc<[ChannelSender<EngineEvent>]>> {
        let subscribers = self.subscribers.lock().unwrap();
        let snapshot = subscribers.get(topic)?;
        if snapshot.is_empty() {
            None
        } else {
            Some(snapshot.clone())
        }
    }

    fn prune_failed_publish_subscribers(
        &self,
        failed_topic: Option<String>,
        first_failed_subscriber: Option<ChannelSender<EngineEvent>>,
        failed_subscribers: Option<Vec<ChannelSender<EngineEvent>>>,
    ) {
        let Some(topic) = failed_topic else {
            return;
        };
        if let Some(failed_subscribers) = failed_subscribers {
            self.prune_topic_subscribers(&topic, &failed_subscribers);
        } else if let Some(failed_subscriber) = first_failed_subscriber {
            self.prune_topic_subscribers(&topic, std::slice::from_ref(&failed_subscriber));
        }
    }
}
