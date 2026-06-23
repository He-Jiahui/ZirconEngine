use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::{
    EditorMessage, EditorMessageDelivery, EditorMessageProtocol, EditorMessageRequest,
    EditorMessageResponse, EditorSubscriberId, EditorTopic, ViewDirtySet,
};

#[derive(Clone, Debug, Default)]
pub struct EditorMessageBus {
    next_subscriber_id: u64,
    subscribers: BTreeMap<EditorSubscriberId, BTreeSet<EditorTopic>>,
    subscriptions: BTreeMap<EditorTopic, BTreeSet<EditorSubscriberId>>,
    inboxes: BTreeMap<EditorSubscriberId, Vec<EditorMessageDelivery>>,
    dirty: ViewDirtySet,
}

impl EditorMessageBus {
    pub fn register_subscriber(
        &mut self,
        topics: impl IntoIterator<Item = EditorTopic>,
    ) -> EditorSubscriberId {
        let subscriber = self.allocate_subscriber_id();
        let topics = topics.into_iter().collect::<BTreeSet<_>>();
        for topic in &topics {
            self.subscriptions
                .entry(topic.clone())
                .or_default()
                .insert(subscriber);
        }
        self.subscribers.insert(subscriber, topics);
        subscriber
    }

    pub fn publish(
        &mut self,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> EditorMessageDispatchReport {
        self.mark_message_dirty(&message);
        let delivered = self
            .subscriptions
            .get(&topic)
            .map(|subscribers| subscribers.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        self.enqueue_deliveries(
            EditorMessageProtocol::Publish,
            &topic,
            &message,
            delivered.iter().copied(),
        );
        EditorMessageDispatchReport::new(EditorMessageProtocol::Publish, topic, delivered)
    }

    pub fn broadcast(
        &mut self,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> EditorMessageDispatchReport {
        self.mark_message_dirty(&message);
        let delivered = self.subscribers.keys().copied().collect::<Vec<_>>();
        self.enqueue_deliveries(
            EditorMessageProtocol::Broadcast,
            &topic,
            &message,
            delivered.iter().copied(),
        );
        EditorMessageDispatchReport::new(EditorMessageProtocol::Broadcast, topic, delivered)
    }

    pub fn request(
        &mut self,
        target: EditorSubscriberId,
        topic: EditorTopic,
        message: EditorMessage,
        handler: &mut impl EditorRequestHandler,
    ) -> Result<EditorMessageResponse, EditorMessageBusError> {
        if !self.subscribers.contains_key(&target) {
            return Err(EditorMessageBusError::UnknownSubscriber { subscriber: target });
        }

        self.mark_message_dirty(&message);
        let request = EditorMessageRequest::new(target, topic.clone(), message.clone());
        self.enqueue_deliveries(
            EditorMessageProtocol::Request,
            &topic,
            &message,
            std::iter::once(target),
        );
        let response = handler.handle_editor_request(&request);
        self.mark_message_dirty(response.message());
        Ok(response)
    }

    pub fn deliveries_for(&self, subscriber: EditorSubscriberId) -> &[EditorMessageDelivery] {
        self.inboxes
            .get(&subscriber)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn drain_deliveries(
        &mut self,
        subscriber: EditorSubscriberId,
    ) -> Vec<EditorMessageDelivery> {
        self.inboxes.remove(&subscriber).unwrap_or_default()
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

    fn allocate_subscriber_id(&mut self) -> EditorSubscriberId {
        self.next_subscriber_id = self.next_subscriber_id.saturating_add(1);
        EditorSubscriberId::new(self.next_subscriber_id)
    }

    fn enqueue_deliveries(
        &mut self,
        protocol: EditorMessageProtocol,
        topic: &EditorTopic,
        message: &EditorMessage,
        delivered: impl IntoIterator<Item = EditorSubscriberId>,
    ) {
        for subscriber in delivered {
            self.inboxes
                .entry(subscriber)
                .or_default()
                .push(EditorMessageDelivery::new(
                    protocol,
                    topic.clone(),
                    message.clone(),
                ));
        }
    }
}

pub trait EditorRequestHandler {
    fn handle_editor_request(&mut self, request: &EditorMessageRequest) -> EditorMessageResponse;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorMessageDispatchReport {
    protocol: EditorMessageProtocol,
    topic: EditorTopic,
    delivered: Vec<EditorSubscriberId>,
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorMessageBusError {
    UnknownSubscriber { subscriber: EditorSubscriberId },
}

impl fmt::Display for EditorMessageBusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSubscriber { subscriber } => {
                write!(
                    f,
                    "editor message subscriber {} is not registered",
                    subscriber.value()
                )
            }
        }
    }
}

impl std::error::Error for EditorMessageBusError {}
