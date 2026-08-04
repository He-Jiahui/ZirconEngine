use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::inbox::{EditorMessageInbox, EditorMessageInboxEnqueue};
use super::retention::{EditorMessageRetention, editor_message_retention};
use super::{
    EditorMessage, EditorMessageDelivery, EditorMessageInboxLimits, EditorMessageInboxStats,
    EditorMessageProtocol, EditorMessageRequest, EditorMessageResponse, EditorSubscriberId,
    EditorTopic, ViewDirtySet,
};

#[derive(Clone, Debug)]
pub(crate) struct EditorMessageBus {
    next_subscriber_id: u64,
    next_delivery_sequence: u64,
    inbox_limits: EditorMessageInboxLimits,
    subscribers: BTreeMap<EditorSubscriberId, BTreeSet<EditorTopic>>,
    subscriptions: BTreeMap<EditorTopic, BTreeSet<EditorSubscriberId>>,
    inboxes: BTreeMap<EditorSubscriberId, EditorMessageInbox>,
    dirty: ViewDirtySet,
}

impl Default for EditorMessageBus {
    fn default() -> Self {
        Self::with_inbox_limits(EditorMessageInboxLimits::default())
    }
}

impl EditorMessageBus {
    pub fn with_inbox_limits(inbox_limits: EditorMessageInboxLimits) -> Self {
        Self {
            next_subscriber_id: 0,
            next_delivery_sequence: 0,
            inbox_limits,
            subscribers: BTreeMap::new(),
            subscriptions: BTreeMap::new(),
            inboxes: BTreeMap::new(),
            dirty: ViewDirtySet::default(),
        }
    }

    pub fn register_subscriber(
        &mut self,
        topics: impl IntoIterator<Item = EditorTopic>,
    ) -> Result<EditorSubscriberId, EditorMessageBusError> {
        let subscriber = self.allocate_subscriber_id()?;
        let topics = topics.into_iter().collect::<BTreeSet<_>>();
        for topic in &topics {
            self.subscriptions
                .entry(topic.clone())
                .or_default()
                .insert(subscriber);
        }
        self.subscribers.insert(subscriber, topics);
        self.inboxes
            .insert(subscriber, EditorMessageInbox::new(self.inbox_limits));
        Ok(subscriber)
    }

    pub fn unregister_subscriber(&mut self, subscriber: EditorSubscriberId) -> bool {
        let Some(topics) = self.subscribers.remove(&subscriber) else {
            return false;
        };
        for topic in topics {
            let remove_topic = if let Some(subscribers) = self.subscriptions.get_mut(&topic) {
                subscribers.remove(&subscriber);
                subscribers.is_empty()
            } else {
                false
            };
            if remove_topic {
                self.subscriptions.remove(&topic);
            }
        }
        self.inboxes.remove(&subscriber);
        true
    }

    pub fn publish(
        &mut self,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> EditorMessageDispatchReport {
        let targets = self.targets_for_topic(&topic);
        if let Some(enqueue) = self.preflight_lossless_delivery(
            EditorMessageProtocol::Publish,
            &topic,
            &message,
            &targets,
        ) {
            return EditorMessageDispatchReport::from_enqueue(
                EditorMessageProtocol::Publish,
                topic,
                enqueue,
            );
        }
        let sequence = match self.allocate_delivery_sequence() {
            Some(sequence) => sequence,
            None => {
                return EditorMessageDispatchReport::failed(
                    EditorMessageProtocol::Publish,
                    topic,
                    EditorMessageDispatchError::DeliverySequenceExhausted,
                );
            }
        };
        let enqueue = self.enqueue_deliveries(
            sequence,
            EditorMessageProtocol::Publish,
            &topic,
            message,
            targets,
        );
        EditorMessageDispatchReport::from_enqueue(EditorMessageProtocol::Publish, topic, enqueue)
    }

    pub fn broadcast(
        &mut self,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> EditorMessageDispatchReport {
        let targets = self.subscribers.keys().copied().collect::<Vec<_>>();
        if let Some(enqueue) = self.preflight_lossless_delivery(
            EditorMessageProtocol::Broadcast,
            &topic,
            &message,
            &targets,
        ) {
            return EditorMessageDispatchReport::from_enqueue(
                EditorMessageProtocol::Broadcast,
                topic,
                enqueue,
            );
        }
        let sequence = match self.allocate_delivery_sequence() {
            Some(sequence) => sequence,
            None => {
                return EditorMessageDispatchReport::failed(
                    EditorMessageProtocol::Broadcast,
                    topic,
                    EditorMessageDispatchError::DeliverySequenceExhausted,
                );
            }
        };
        let enqueue = self.enqueue_deliveries(
            sequence,
            EditorMessageProtocol::Broadcast,
            &topic,
            message,
            targets,
        );
        EditorMessageDispatchReport::from_enqueue(EditorMessageProtocol::Broadcast, topic, enqueue)
    }

    pub fn request(
        &mut self,
        target: EditorSubscriberId,
        topic: EditorTopic,
        message: EditorMessage,
        handler: &mut impl EditorRequestHandler,
    ) -> Result<EditorMessageResponse, EditorMessageBusError> {
        let request = self.begin_request(target, topic, message)?;
        let response = handler.handle_editor_request(&request);
        self.complete_request(target, &response)?;
        Ok(response)
    }

    pub(super) fn begin_request(
        &mut self,
        target: EditorSubscriberId,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> Result<EditorMessageRequest, EditorMessageBusError> {
        self.ensure_subscriber(target)?;
        let sequence = self
            .allocate_delivery_sequence()
            .ok_or(EditorMessageBusError::DeliverySequenceExhausted)?;
        let request = EditorMessageRequest::new(target, topic.clone(), message.clone());
        let enqueue = self.enqueue_deliveries(
            sequence,
            EditorMessageProtocol::Request,
            &topic,
            message,
            std::iter::once(target),
        );
        if enqueue.backpressured.contains(&target) {
            return Err(EditorMessageBusError::Backpressured { subscriber: target });
        }
        Ok(request)
    }

    pub(super) fn complete_request(
        &mut self,
        target: EditorSubscriberId,
        response: &EditorMessageResponse,
    ) -> Result<(), EditorMessageBusError> {
        self.ensure_subscriber(target)?;
        self.mark_message_dirty(response.message());
        Ok(())
    }

    #[cfg(test)]
    pub fn deliveries_for(&self, subscriber: EditorSubscriberId) -> Vec<EditorMessageDelivery> {
        self.inboxes
            .get(&subscriber)
            .map(EditorMessageInbox::deliveries)
            .unwrap_or_default()
    }

    pub fn drain_deliveries(
        &mut self,
        subscriber: EditorSubscriberId,
    ) -> Vec<EditorMessageDelivery> {
        self.inboxes
            .get_mut(&subscriber)
            .map(EditorMessageInbox::drain)
            .unwrap_or_default()
    }

    pub fn inbox_stats(&self, subscriber: EditorSubscriberId) -> Option<EditorMessageInboxStats> {
        self.inboxes
            .get(&subscriber)
            .map(|inbox| inbox.stats(self.next_delivery_sequence))
    }

    pub fn mark_message_dirty(&mut self, message: &EditorMessage) {
        if let Some(dirty) = message.dirty() {
            self.dirty.mark(dirty.view().clone(), dirty.mask());
        }
    }

    pub fn mark_view_dirty(
        &mut self,
        view: crate::core::editor_event::ViewInstanceId,
        mask: super::EditorViewInvalidationMask,
    ) {
        self.dirty.mark(view, mask);
    }

    pub fn dirty_set(&self) -> &ViewDirtySet {
        &self.dirty
    }

    pub fn drain_dirty(&mut self) -> ViewDirtySet {
        std::mem::take(&mut self.dirty)
    }

    fn allocate_subscriber_id(&mut self) -> Result<EditorSubscriberId, EditorMessageBusError> {
        let next = self
            .next_subscriber_id
            .checked_add(1)
            .ok_or(EditorMessageBusError::SubscriberIdExhausted)?;
        self.next_subscriber_id = next;
        Ok(EditorSubscriberId::new(next))
    }

    fn ensure_subscriber(
        &self,
        subscriber: EditorSubscriberId,
    ) -> Result<(), EditorMessageBusError> {
        self.subscribers
            .contains_key(&subscriber)
            .then_some(())
            .ok_or(EditorMessageBusError::UnknownSubscriber { subscriber })
    }

    fn enqueue_deliveries(
        &mut self,
        sequence: u64,
        protocol: EditorMessageProtocol,
        topic: &EditorTopic,
        message: EditorMessage,
        targets: impl IntoIterator<Item = EditorSubscriberId>,
    ) -> EditorMessageEnqueueReport {
        let delivery =
            EditorMessageDelivery::with_sequence(protocol, topic.clone(), message, sequence);
        let limits = self.inbox_limits;
        let mut report = EditorMessageEnqueueReport::default();
        for subscriber in targets {
            let outcome = self
                .inboxes
                .entry(subscriber)
                .or_insert_with(|| EditorMessageInbox::new(limits))
                .enqueue(delivery.clone());
            match outcome {
                EditorMessageInboxEnqueue::Enqueued => report.delivered.push(subscriber),
                EditorMessageInboxEnqueue::Coalesced => {
                    report.delivered.push(subscriber);
                    report.coalesced.push(subscriber);
                }
                EditorMessageInboxEnqueue::CoalescedAfterDrop => {
                    report.delivered.push(subscriber);
                    report.coalesced.push(subscriber);
                    report.dropped.push(subscriber);
                }
                EditorMessageInboxEnqueue::EnqueuedAfterDrop => {
                    report.delivered.push(subscriber);
                    report.dropped.push(subscriber);
                }
                EditorMessageInboxEnqueue::Dropped => report.dropped.push(subscriber),
                EditorMessageInboxEnqueue::Backpressured => report.backpressured.push(subscriber),
            }
        }
        if !report.delivered.is_empty() {
            self.mark_message_dirty(delivery.message());
        }
        report
    }

    fn targets_for_topic(&self, topic: &EditorTopic) -> Vec<EditorSubscriberId> {
        self.subscriptions
            .get(topic)
            .map(|subscribers| subscribers.iter().copied().collect())
            .unwrap_or_default()
    }

    fn preflight_lossless_delivery(
        &mut self,
        protocol: EditorMessageProtocol,
        topic: &EditorTopic,
        message: &EditorMessage,
        targets: &[EditorSubscriberId],
    ) -> Option<EditorMessageEnqueueReport> {
        if editor_message_retention(protocol, message) != EditorMessageRetention::Lossless {
            return None;
        }

        let retained_bytes =
            EditorMessageDelivery::new(protocol, topic.clone(), message.clone()).retained_bytes();
        let backpressured = targets
            .iter()
            .copied()
            .filter(|subscriber| {
                self.inboxes
                    .get(subscriber)
                    .is_some_and(|inbox| !inbox.can_enqueue_lossless(retained_bytes))
            })
            .collect::<Vec<_>>();
        if backpressured.is_empty() {
            return None;
        }
        for subscriber in &backpressured {
            if let Some(inbox) = self.inboxes.get_mut(subscriber) {
                inbox.note_lossless_backpressure();
            }
        }
        Some(EditorMessageEnqueueReport {
            backpressured,
            ..EditorMessageEnqueueReport::default()
        })
    }

    fn allocate_delivery_sequence(&mut self) -> Option<u64> {
        let next = self.next_delivery_sequence.checked_add(1)?;
        self.next_delivery_sequence = next;
        Some(next)
    }

    #[cfg(test)]
    pub(crate) fn set_next_subscriber_id_for_test(&mut self, value: u64) {
        self.next_subscriber_id = value;
    }

    #[cfg(test)]
    pub(crate) fn set_next_delivery_sequence_for_test(&mut self, value: u64) {
        self.next_delivery_sequence = value;
    }
}

#[derive(Default)]
struct EditorMessageEnqueueReport {
    delivered: Vec<EditorSubscriberId>,
    coalesced: Vec<EditorSubscriberId>,
    dropped: Vec<EditorSubscriberId>,
    backpressured: Vec<EditorSubscriberId>,
}

pub trait EditorRequestHandler {
    fn handle_editor_request(&mut self, request: &EditorMessageRequest) -> EditorMessageResponse;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorMessageDispatchReport {
    protocol: EditorMessageProtocol,
    topic: EditorTopic,
    delivered: Vec<EditorSubscriberId>,
    coalesced: Vec<EditorSubscriberId>,
    dropped: Vec<EditorSubscriberId>,
    backpressured: Vec<EditorSubscriberId>,
    error: Option<EditorMessageDispatchError>,
}

impl EditorMessageDispatchReport {
    pub fn new(
        protocol: EditorMessageProtocol,
        topic: EditorTopic,
        delivered: Vec<EditorSubscriberId>,
    ) -> Self {
        Self {
            protocol,
            topic,
            delivered,
            coalesced: Vec::new(),
            dropped: Vec::new(),
            backpressured: Vec::new(),
            error: None,
        }
    }

    fn from_enqueue(
        protocol: EditorMessageProtocol,
        topic: EditorTopic,
        enqueue: EditorMessageEnqueueReport,
    ) -> Self {
        Self {
            protocol,
            topic,
            delivered: enqueue.delivered,
            coalesced: enqueue.coalesced,
            dropped: enqueue.dropped,
            backpressured: enqueue.backpressured,
            error: None,
        }
    }

    fn failed(
        protocol: EditorMessageProtocol,
        topic: EditorTopic,
        error: EditorMessageDispatchError,
    ) -> Self {
        Self {
            protocol,
            topic,
            delivered: Vec::new(),
            coalesced: Vec::new(),
            dropped: Vec::new(),
            backpressured: Vec::new(),
            error: Some(error),
        }
    }

    pub fn protocol(&self) -> EditorMessageProtocol {
        self.protocol
    }

    pub fn topic(&self) -> &EditorTopic {
        &self.topic
    }

    pub fn delivered(&self) -> &[EditorSubscriberId] {
        &self.delivered
    }

    pub fn coalesced(&self) -> &[EditorSubscriberId] {
        &self.coalesced
    }

    pub fn dropped(&self) -> &[EditorSubscriberId] {
        &self.dropped
    }

    pub fn backpressured(&self) -> &[EditorSubscriberId] {
        &self.backpressured
    }

    pub fn error(&self) -> Option<EditorMessageDispatchError> {
        self.error
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorMessageDispatchError {
    DeliverySequenceExhausted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorMessageBusError {
    SubscriberIdExhausted,
    DeliverySequenceExhausted,
    UnknownSubscriber { subscriber: EditorSubscriberId },
    Backpressured { subscriber: EditorSubscriberId },
}

impl fmt::Display for EditorMessageBusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SubscriberIdExhausted => {
                f.write_str("editor message subscriber id sequence is exhausted")
            }
            Self::DeliverySequenceExhausted => {
                f.write_str("editor message delivery sequence is exhausted")
            }
            Self::UnknownSubscriber { subscriber } => {
                write!(
                    f,
                    "editor message subscriber {} is not registered",
                    subscriber.value()
                )
            }
            Self::Backpressured { subscriber } => write!(
                f,
                "editor message subscriber {} cannot accept another lossless delivery",
                subscriber.value()
            ),
        }
    }
}

impl std::error::Error for EditorMessageBusError {}
