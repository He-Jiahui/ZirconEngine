use std::sync::Arc;

use crate::core::framework::channel::ChannelSender;
use crate::core::framework::events::EngineEvent;

use super::EventBus;

impl EventBus {
    fn exact_five_surviving_subscribers(
        subscribers: [&ChannelSender<EngineEvent>; 5],
        failed_tuple: (bool, bool, bool, bool, bool),
    ) -> Arc<[ChannelSender<EngineEvent>]> {
        let mut survivor_count = 0;
        let mut first_survivor = None;
        let mut second_survivor = None;
        let mut third_survivor = None;
        let mut fourth_survivor = None;
        let mut fifth_survivor = None;
        for (subscriber, subscriber_failed) in subscribers.into_iter().zip([
            failed_tuple.0,
            failed_tuple.1,
            failed_tuple.2,
            failed_tuple.3,
            failed_tuple.4,
        ]) {
            if !subscriber_failed {
                survivor_count += 1;
                Self::record_surviving_subscriber(
                    subscriber,
                    survivor_count,
                    &mut first_survivor,
                    &mut second_survivor,
                    &mut third_survivor,
                    &mut fourth_survivor,
                    &mut fifth_survivor,
                );
            }
        }
        Self::surviving_subscriber_slice(
            survivor_count,
            first_survivor,
            second_survivor,
            third_survivor,
            fourth_survivor,
            fifth_survivor,
        )
    }

    fn record_surviving_subscriber(
        subscriber: &ChannelSender<EngineEvent>,
        survivor_count: usize,
        first_survivor: &mut Option<ChannelSender<EngineEvent>>,
        second_survivor: &mut Option<ChannelSender<EngineEvent>>,
        third_survivor: &mut Option<ChannelSender<EngineEvent>>,
        fourth_survivor: &mut Option<ChannelSender<EngineEvent>>,
        fifth_survivor: &mut Option<ChannelSender<EngineEvent>>,
    ) {
        let survivor_slot = match survivor_count {
            1 => first_survivor,
            2 => second_survivor,
            3 => third_survivor,
            4 => fourth_survivor,
            5 => fifth_survivor,
            _ => unreachable!("exact five prune cannot retain more than five subscribers"),
        };
        *survivor_slot = Some(subscriber.clone());
    }

    fn surviving_subscriber_slice(
        survivor_count: usize,
        first_survivor: Option<ChannelSender<EngineEvent>>,
        second_survivor: Option<ChannelSender<EngineEvent>>,
        third_survivor: Option<ChannelSender<EngineEvent>>,
        fourth_survivor: Option<ChannelSender<EngineEvent>>,
        fifth_survivor: Option<ChannelSender<EngineEvent>>,
    ) -> Arc<[ChannelSender<EngineEvent>]> {
        match survivor_count {
            1 => Arc::<[ChannelSender<EngineEvent>]>::from([first_survivor.unwrap()]),
            2 => Arc::<[ChannelSender<EngineEvent>]>::from([
                first_survivor.unwrap(),
                second_survivor.unwrap(),
            ]),
            3 => Arc::<[ChannelSender<EngineEvent>]>::from([
                first_survivor.unwrap(),
                second_survivor.unwrap(),
                third_survivor.unwrap(),
            ]),
            4 => Arc::<[ChannelSender<EngineEvent>]>::from([
                first_survivor.unwrap(),
                second_survivor.unwrap(),
                third_survivor.unwrap(),
                fourth_survivor.unwrap(),
            ]),
            5 => Arc::<[ChannelSender<EngineEvent>]>::from([
                first_survivor.unwrap(),
                second_survivor.unwrap(),
                third_survivor.unwrap(),
                fourth_survivor.unwrap(),
                fifth_survivor.unwrap(),
            ]),
            _ => unreachable!("exact five prune caller handles all-failed topics directly"),
        }
    }

    pub(super) fn prune_topic_subscribers(
        &self,
        topic: &str,
        failed_subscribers: &[ChannelSender<EngineEvent>],
    ) {
        let mut subscribers = self.subscribers.lock().unwrap();
        let should_remove_topic = if let Some(topic_subscribers) = subscribers.get_mut(topic) {
            match topic_subscribers.as_ref() {
                [] => true,
                [subscriber] => Self::subscriber_failed(subscriber, failed_subscribers),
                [first_subscriber, second_subscriber] => {
                    let first_failed =
                        Self::subscriber_failed(first_subscriber, failed_subscribers);
                    let second_failed =
                        Self::subscriber_failed(second_subscriber, failed_subscribers);
                    match (first_failed, second_failed) {
                        (true, true) => true,
                        (true, false) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    second_subscriber.clone()
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, true) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    first_subscriber.clone()
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, false) => false,
                    }
                }
                [first_subscriber, second_subscriber, third_subscriber] => {
                    let first_failed =
                        Self::subscriber_failed(first_subscriber, failed_subscribers);
                    let second_failed =
                        Self::subscriber_failed(second_subscriber, failed_subscribers);
                    let third_failed =
                        Self::subscriber_failed(third_subscriber, failed_subscribers);
                    match (first_failed, second_failed, third_failed) {
                        (true, true, true) => true,
                        (true, true, false) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    third_subscriber.clone()
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (true, false, true) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    second_subscriber.clone()
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, true, true) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    first_subscriber.clone()
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (true, false, false) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    second_subscriber.clone(),
                                    third_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, true, false) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    first_subscriber.clone(),
                                    third_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, false, true) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    first_subscriber.clone(),
                                    second_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, false, false) => false,
                    }
                }
                [first_subscriber, second_subscriber, third_subscriber, fourth_subscriber] => {
                    let first_failed =
                        Self::subscriber_failed(first_subscriber, failed_subscribers);
                    let second_failed =
                        Self::subscriber_failed(second_subscriber, failed_subscribers);
                    let third_failed =
                        Self::subscriber_failed(third_subscriber, failed_subscribers);
                    let fourth_failed =
                        Self::subscriber_failed(fourth_subscriber, failed_subscribers);
                    match (first_failed, second_failed, third_failed, fourth_failed) {
                        (true, true, true, true) => true,
                        (true, true, true, false) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    fourth_subscriber.clone()
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (true, true, false, true) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    third_subscriber.clone()
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (true, false, true, true) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    second_subscriber.clone()
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, true, true, true) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    first_subscriber.clone()
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (true, true, false, false) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    third_subscriber.clone(),
                                    fourth_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (true, false, true, false) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    second_subscriber.clone(),
                                    fourth_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (true, false, false, true) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    second_subscriber.clone(),
                                    third_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, true, true, false) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    first_subscriber.clone(),
                                    fourth_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, true, false, true) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    first_subscriber.clone(),
                                    third_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, false, true, true) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    first_subscriber.clone(),
                                    second_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (true, false, false, false) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    second_subscriber.clone(),
                                    third_subscriber.clone(),
                                    fourth_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, true, false, false) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    first_subscriber.clone(),
                                    third_subscriber.clone(),
                                    fourth_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, false, true, false) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    first_subscriber.clone(),
                                    second_subscriber.clone(),
                                    fourth_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, false, false, true) => {
                            let surviving_subscribers =
                                Arc::<[ChannelSender<EngineEvent>]>::from([
                                    first_subscriber.clone(),
                                    second_subscriber.clone(),
                                    third_subscriber.clone(),
                                ]);
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                        (false, false, false, false) => false,
                    }
                }
                [first_subscriber, second_subscriber, third_subscriber, fourth_subscriber, fifth_subscriber] =>
                {
                    let first_failed =
                        Self::subscriber_failed(first_subscriber, failed_subscribers);
                    let second_failed =
                        Self::subscriber_failed(second_subscriber, failed_subscribers);
                    let third_failed =
                        Self::subscriber_failed(third_subscriber, failed_subscribers);
                    let fourth_failed =
                        Self::subscriber_failed(fourth_subscriber, failed_subscribers);
                    let fifth_failed =
                        Self::subscriber_failed(fifth_subscriber, failed_subscribers);
                    let failed_tuple = (
                        first_failed,
                        second_failed,
                        third_failed,
                        fourth_failed,
                        fifth_failed,
                    );
                    match failed_tuple {
                        (true, true, true, true, true) => true,
                        (false, false, false, false, false) => false,
                        failed_tuple => {
                            let surviving_subscribers = Self::exact_five_surviving_subscribers(
                                [
                                    first_subscriber,
                                    second_subscriber,
                                    third_subscriber,
                                    fourth_subscriber,
                                    fifth_subscriber,
                                ],
                                failed_tuple,
                            );
                            *topic_subscribers = surviving_subscribers;
                            false
                        }
                    }
                }
                current_subscribers => {
                    let mut retained_subscribers: Option<Vec<ChannelSender<EngineEvent>>> = None;
                    let mut saw_failed_subscriber = false;
                    for (subscriber_index, subscriber) in current_subscribers.iter().enumerate() {
                        if Self::subscriber_failed(subscriber, failed_subscribers) {
                            if retained_subscribers.is_none()
                                && !saw_failed_subscriber
                                && subscriber_index > 0
                            {
                                let mut retained_subscriber_list =
                                    Vec::with_capacity(current_subscribers.len());
                                retained_subscriber_list.extend(
                                    current_subscribers[..subscriber_index].iter().cloned(),
                                );
                                retained_subscribers = Some(retained_subscriber_list);
                            }
                            saw_failed_subscriber = true;
                            continue;
                        }
                        if saw_failed_subscriber {
                            retained_subscribers
                                .get_or_insert_with(|| {
                                    Vec::with_capacity(current_subscribers.len())
                                })
                                .push(subscriber.clone());
                        }
                    }
                    if let Some(retained_subscribers) = retained_subscribers {
                        *topic_subscribers = retained_subscribers.into();
                        false
                    } else {
                        saw_failed_subscriber
                    }
                }
            }
        } else {
            false
        };

        if should_remove_topic {
            subscribers.remove(topic);
        }
    }
}
