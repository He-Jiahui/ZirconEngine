use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use super::inbox::{EditorMessageInbox, EditorMessageInboxEnqueue};
use super::retention::EditorMessageRetention;
use super::{
    EditorMessage, EditorMessageDelivery, EditorMessageInboxLimits, EditorMessageInboxStats,
    EditorMessageProtocol, EditorMessageRequest, EditorMessageResponse, EditorSubscriberId,
    EditorTopic, EditorUiDeltaBarrierKind, EditorUiDeltaBatch, EditorUiDeltaQueue, ViewDirtySet,
};
use crate::core::editor_event::{EditorEventSequence, ViewInstanceId};
use zircon_runtime_interface::ui::event_ui::UiReflectionNodePatch;

#[derive(Clone, Debug)]
pub(crate) struct EditorMessageBus {
    next_subscriber_id: u64,
    next_delivery_sequence: u64,
    inbox_limits: EditorMessageInboxLimits,
    subscribers: BTreeMap<EditorSubscriberId, BTreeSet<EditorTopic>>,
    subscriptions: BTreeMap<EditorTopic, BTreeSet<EditorSubscriberId>>,
    inboxes: BTreeMap<EditorSubscriberId, Arc<Mutex<EditorMessageInbox>>>,
    dirty: ViewDirtySet,
    ui_deltas: EditorUiDeltaQueue,
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
            ui_deltas: EditorUiDeltaQueue::default(),
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
        self.inboxes.insert(
            subscriber,
            Arc::new(Mutex::new(EditorMessageInbox::new(self.inbox_limits))),
        );
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
        match self.prepare_publish(topic, message) {
            Ok(plan) => self.finish_dispatch(plan),
            Err((report, _message)) => report,
        }
    }

    pub fn broadcast(
        &mut self,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> EditorMessageDispatchReport {
        match self.prepare_broadcast(topic, message) {
            Ok(plan) => self.finish_dispatch(plan),
            Err((report, _message)) => report,
        }
    }

    pub fn request(
        &mut self,
        target: EditorSubscriberId,
        topic: EditorTopic,
        message: EditorMessage,
        handler: &mut impl EditorRequestHandler,
    ) -> Result<EditorMessageResponse, EditorMessageBusError> {
        let (request, plan) = self.prepare_request(target, topic, message)?;
        let report = self.finish_dispatch(plan);
        if report.backpressured().contains(&target) {
            return Err(EditorMessageBusError::Backpressured { subscriber: target });
        }
        let response = handler.handle_editor_request(&request);
        self.complete_request(target, &response)?;
        Ok(response)
    }

    pub(super) fn prepare_publish(
        &mut self,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> Result<EditorMessageDispatchPlan, (EditorMessageDispatchReport, EditorMessage)> {
        let Some(subscribers) = self.subscriptions.get(&topic) else {
            return Err((
                EditorMessageDispatchReport::new(EditorMessageProtocol::Publish, topic, Vec::new()),
                message,
            ));
        };
        let targets = subscribers.iter().copied().collect::<Vec<_>>();
        self.prepare_dispatch(EditorMessageProtocol::Publish, topic, message, targets)
    }

    pub(super) fn prepare_broadcast(
        &mut self,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> Result<EditorMessageDispatchPlan, (EditorMessageDispatchReport, EditorMessage)> {
        let targets = self.subscribers.keys().copied().collect::<Vec<_>>();
        self.prepare_dispatch(EditorMessageProtocol::Broadcast, topic, message, targets)
    }

    pub(super) fn prepare_request(
        &mut self,
        target: EditorSubscriberId,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> Result<(EditorMessageRequest, EditorMessageDispatchPlan), EditorMessageBusError> {
        self.ensure_subscriber(target)?;
        let sequence = self
            .allocate_delivery_sequence()
            .ok_or(EditorMessageBusError::DeliverySequenceExhausted)?;
        let delivery = EditorMessageDelivery::with_sequence(
            EditorMessageProtocol::Request,
            topic,
            message,
            sequence,
        );
        let request = EditorMessageRequest::from_delivery(target, delivery.clone());
        let plan = self.dispatch_plan(delivery, [target]);
        Ok((request, plan))
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
            .map(|inbox| lock_inbox(inbox).deliveries())
            .unwrap_or_default()
    }

    pub fn drain_deliveries(
        &mut self,
        subscriber: EditorSubscriberId,
    ) -> Vec<EditorMessageDelivery> {
        self.inboxes
            .get(&subscriber)
            .map(|inbox| lock_inbox(inbox).drain())
            .unwrap_or_default()
    }

    pub fn inbox_stats(&self, subscriber: EditorSubscriberId) -> Option<EditorMessageInboxStats> {
        self.inboxes
            .get(&subscriber)
            .map(|inbox| lock_inbox(inbox).stats(self.next_delivery_sequence))
    }

    pub(super) fn inbox_handle(
        &self,
        subscriber: EditorSubscriberId,
    ) -> Option<Arc<Mutex<EditorMessageInbox>>> {
        self.inboxes.get(&subscriber).cloned()
    }

    pub(super) fn inbox_stats_snapshot(
        &self,
        subscriber: EditorSubscriberId,
    ) -> Option<(Arc<Mutex<EditorMessageInbox>>, u64)> {
        self.inbox_handle(subscriber)
            .map(|inbox| (inbox, self.next_delivery_sequence))
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

    /// Marks an existing view without cloning its identifier when it is already dirty.
    pub fn mark_view_dirty_ref(
        &mut self,
        view: &crate::core::editor_event::ViewInstanceId,
        mask: super::EditorViewInvalidationMask,
    ) {
        self.dirty.mark_ref(view, mask);
    }

    /// Merges one projected dirty set while the bus mutex is held only once.
    pub fn mark_view_dirty_set(&mut self, dirty: &ViewDirtySet) {
        for (view, mask) in dirty.iter() {
            self.dirty.mark_ref(view, mask);
        }
    }

    pub fn dirty_set(&self) -> &ViewDirtySet {
        &self.dirty
    }

    pub fn drain_dirty(&mut self) -> ViewDirtySet {
        std::mem::take(&mut self.dirty)
    }

    pub fn push_editor_ui_patch(&mut self, view: ViewInstanceId, patch: UiReflectionNodePatch) {
        self.ui_deltas.push_patch(view, patch);
    }

    pub fn push_editor_ui_barrier(
        &mut self,
        kind: EditorUiDeltaBarrierKind,
        sequence: EditorEventSequence,
    ) {
        self.ui_deltas.push_barrier(kind, sequence);
    }

    pub fn drain_view_updates(&mut self) -> (ViewDirtySet, EditorUiDeltaBatch) {
        (std::mem::take(&mut self.dirty), self.ui_deltas.drain())
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

    fn prepare_dispatch(
        &mut self,
        protocol: EditorMessageProtocol,
        topic: EditorTopic,
        message: EditorMessage,
        targets: Vec<EditorSubscriberId>,
    ) -> Result<EditorMessageDispatchPlan, (EditorMessageDispatchReport, EditorMessage)> {
        let Some(sequence) = self.allocate_delivery_sequence() else {
            return Err((
                EditorMessageDispatchReport::failed(
                    protocol,
                    topic,
                    EditorMessageDispatchError::DeliverySequenceExhausted,
                ),
                message,
            ));
        };
        let delivery = EditorMessageDelivery::with_sequence(protocol, topic, message, sequence);
        Ok(self.dispatch_plan(delivery, targets))
    }

    fn dispatch_plan(
        &self,
        delivery: EditorMessageDelivery,
        targets: impl IntoIterator<Item = EditorSubscriberId>,
    ) -> EditorMessageDispatchPlan {
        let targets = targets
            .into_iter()
            .filter_map(|subscriber| {
                self.inbox_handle(subscriber)
                    .map(|inbox| EditorMessageDispatchTarget { subscriber, inbox })
            })
            .collect();
        EditorMessageDispatchPlan { delivery, targets }
    }

    fn finish_dispatch(&mut self, plan: EditorMessageDispatchPlan) -> EditorMessageDispatchReport {
        let enqueue = plan.dispatch();
        if !enqueue.delivered.is_empty() {
            self.mark_message_dirty(plan.delivery.message());
        }
        plan.into_report(enqueue)
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

pub(super) struct EditorMessageDispatchPlan {
    delivery: EditorMessageDelivery,
    targets: Vec<EditorMessageDispatchTarget>,
}

struct EditorMessageDispatchTarget {
    subscriber: EditorSubscriberId,
    inbox: Arc<Mutex<EditorMessageInbox>>,
}

impl EditorMessageDispatchPlan {
    pub(super) fn dispatch(&self) -> EditorMessageEnqueueReport {
        match self.delivery.retention() {
            EditorMessageRetention::Lossless => self.dispatch_lossless(),
            EditorMessageRetention::Latest(_) | EditorMessageRetention::Bounded => {
                self.dispatch_best_effort()
            }
        }
    }

    pub(super) fn message(&self) -> &EditorMessage {
        self.delivery.message()
    }

    pub(super) fn into_report(
        &self,
        enqueue: EditorMessageEnqueueReport,
    ) -> EditorMessageDispatchReport {
        EditorMessageDispatchReport::from_enqueue(
            self.delivery.protocol(),
            self.delivery.topic().clone(),
            enqueue,
        )
    }

    fn dispatch_lossless(&self) -> EditorMessageEnqueueReport {
        // Targets originate from BTree indexes, so every fanout acquires inboxes by subscriber ID.
        // This keeps all-or-nothing lossless admission free from cross-fanout lock inversions.
        let mut inboxes = self
            .targets
            .iter()
            .map(|target| lock_inbox(target.inbox.as_ref()))
            .collect::<Vec<_>>();
        let retained_bytes = self.delivery.retained_bytes();
        let mut report = EditorMessageEnqueueReport::default();
        for (target, inbox) in self.targets.iter().zip(inboxes.iter_mut()) {
            if !inbox.can_enqueue_lossless(retained_bytes) {
                inbox.note_lossless_backpressure();
                report.backpressured.push(target.subscriber);
            }
        }
        if !report.backpressured.is_empty() {
            return report;
        }
        for (target, inbox) in self.targets.iter().zip(inboxes.iter_mut()) {
            let outcome = inbox.enqueue(self.delivery.clone());
            debug_assert!(matches!(outcome, EditorMessageInboxEnqueue::Enqueued));
            record_enqueue_outcome(&mut report, target.subscriber, outcome);
        }
        report
    }

    fn dispatch_best_effort(&self) -> EditorMessageEnqueueReport {
        let mut report = EditorMessageEnqueueReport::default();
        for target in &self.targets {
            let outcome = lock_inbox(target.inbox.as_ref()).enqueue(self.delivery.clone());
            record_enqueue_outcome(&mut report, target.subscriber, outcome);
        }
        report
    }
}

fn record_enqueue_outcome(
    report: &mut EditorMessageEnqueueReport,
    subscriber: EditorSubscriberId,
    outcome: EditorMessageInboxEnqueue,
) {
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

fn lock_inbox(inbox: &Mutex<EditorMessageInbox>) -> MutexGuard<'_, EditorMessageInbox> {
    inbox
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Default)]
pub(super) struct EditorMessageEnqueueReport {
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
