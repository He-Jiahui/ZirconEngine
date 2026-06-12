use crate::core::framework::channel::ChannelSender;
use crate::core::framework::events::EngineEvent;

use super::EventBus;

impl EventBus {
    pub(super) fn record_failed_subscriber(
        failed_event: EngineEvent,
        subscriber: &ChannelSender<EngineEvent>,
        subscriber_count: usize,
        failed_topic: &mut Option<String>,
        first_failed_subscriber: &mut Option<ChannelSender<EngineEvent>>,
        failed_subscribers: &mut Option<Vec<ChannelSender<EngineEvent>>>,
    ) {
        if failed_topic.is_none() {
            *failed_topic = Some(failed_event.topic);
        }
        if let Some(failed_subscriber_list) = failed_subscribers.as_mut() {
            failed_subscriber_list.push(subscriber.clone());
        } else if let Some(first_failed_subscriber) = first_failed_subscriber.take() {
            let mut failed_subscriber_list = Vec::with_capacity(subscriber_count);
            failed_subscriber_list.push(first_failed_subscriber);
            failed_subscriber_list.push(subscriber.clone());
            *failed_subscribers = Some(failed_subscriber_list);
        } else {
            *first_failed_subscriber = Some(subscriber.clone());
        }
    }

    pub(super) fn subscriber_failed(
        subscriber: &ChannelSender<EngineEvent>,
        failed_subscribers: &[ChannelSender<EngineEvent>],
    ) -> bool {
        if let [failed_subscriber] = failed_subscribers {
            return subscriber.same_channel(failed_subscriber);
        }
        if let [first_failed_subscriber, second_failed_subscriber] = failed_subscribers {
            return subscriber.same_channel(first_failed_subscriber)
                || subscriber.same_channel(second_failed_subscriber);
        }
        if let [first_failed_subscriber, second_failed_subscriber, third_failed_subscriber] =
            failed_subscribers
        {
            return subscriber.same_channel(first_failed_subscriber)
                || subscriber.same_channel(second_failed_subscriber)
                || subscriber.same_channel(third_failed_subscriber);
        }
        if let [first_failed_subscriber, second_failed_subscriber, third_failed_subscriber, fourth_failed_subscriber] =
            failed_subscribers
        {
            return subscriber.same_channel(first_failed_subscriber)
                || subscriber.same_channel(second_failed_subscriber)
                || subscriber.same_channel(third_failed_subscriber)
                || subscriber.same_channel(fourth_failed_subscriber);
        }
        if let [first_failed_subscriber, second_failed_subscriber, third_failed_subscriber, fourth_failed_subscriber, fifth_failed_subscriber] =
            failed_subscribers
        {
            return subscriber.same_channel(first_failed_subscriber)
                || subscriber.same_channel(second_failed_subscriber)
                || subscriber.same_channel(third_failed_subscriber)
                || subscriber.same_channel(fourth_failed_subscriber)
                || subscriber.same_channel(fifth_failed_subscriber);
        }
        failed_subscribers
            .iter()
            .any(|failed_subscriber| subscriber.same_channel(failed_subscriber))
    }
}
