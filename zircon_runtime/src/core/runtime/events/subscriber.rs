use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};
use std::time::{Duration, Instant};

use crate::core::framework::events::{
    EngineEvent, EngineEventDeliveryPolicy, EngineEventReceiveError,
    EngineEventReceiveTimeoutError, EngineEventSubscription, EngineEventTryReceiveError,
};

use super::diagnostics::EventBusDiagnosticsState;
use super::topic::{EventBusState, EventTopic};

const OVERFLOW_TIMEOUT_WAIT_SLICE: Duration = Duration::from_secs(24 * 60 * 60);

pub(super) enum EventDeliveryStatus {
    Delivered,
    Disconnected,
}

struct QueuedEngineEvent {
    event: Arc<EngineEvent>,
    queued_at: Option<Instant>,
}

struct DequeuedEngineEvent {
    queued: QueuedEngineEvent,
    queue_age: Option<Duration>,
}

pub(super) struct EventSubscriber {
    id: u64,
    policy: EngineEventDeliveryPolicy,
    queue_state: Mutex<EventQueueState>,
    queue_ready: Condvar,
    diagnostics: Arc<EventBusDiagnosticsState>,
}

struct EventQueueState {
    queue: VecDeque<QueuedEngineEvent>,
    active: bool,
}

impl EventSubscriber {
    pub(super) fn new(
        id: u64,
        policy: EngineEventDeliveryPolicy,
        diagnostics: Arc<EventBusDiagnosticsState>,
    ) -> Self {
        Self {
            id,
            policy,
            queue_state: Mutex::new(EventQueueState {
                queue: VecDeque::new(),
                active: true,
            }),
            queue_ready: Condvar::new(),
            diagnostics,
        }
    }

    pub(super) fn id(&self) -> u64 {
        self.id
    }

    #[cfg(test)]
    pub(super) fn poison_queue_state_for_test(&self) {
        let _queue = self
            .queue_state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        panic!("poison EventBus subscriber queue");
    }

    pub(super) fn deliver(&self, event: Arc<EngineEvent>) -> EventDeliveryStatus {
        let mut queue_state = self.lock_queue_state();
        if !queue_state.active {
            return EventDeliveryStatus::Disconnected;
        }

        let capacity = match self.policy {
            EngineEventDeliveryPolicy::Lossless => None,
            EngineEventDeliveryPolicy::BoundedDropOldest { capacity } => Some(capacity.get()),
            EngineEventDeliveryPolicy::Latest => Some(1),
        };
        let dropped = capacity
            .is_some_and(|capacity| queue_state.queue.len() == capacity)
            .then(|| {
                queue_state
                    .queue
                    .pop_front()
                    .expect("a full event queue must contain an event")
            });

        queue_state.queue.push_back(QueuedEngineEvent {
            event,
            queued_at: None,
        });
        let queued_at = match dropped {
            Some(dropped) => self
                .diagnostics
                .record_replaced_and_capture_time(dropped.queued_at),
            None => self.diagnostics.record_enqueued_and_capture_time(),
        };
        queue_state
            .queue
            .back_mut()
            .expect("the delivered event must remain queued")
            .queued_at = queued_at;
        drop(queue_state);
        self.queue_ready.notify_one();
        EventDeliveryStatus::Delivered
    }

    pub(super) fn deactivate_and_drain(&self) {
        let queued = {
            let mut queue_state = self.lock_queue_state();
            if !queue_state.active {
                return;
            }
            queue_state.active = false;
            std::mem::take(&mut queue_state.queue)
        };
        for queued in queued {
            self.diagnostics.record_dequeued(queued.queued_at);
        }
        self.diagnostics.record_disconnected();
        self.queue_ready.notify_all();
    }

    fn receive(&self) -> Result<Arc<EngineEvent>, EngineEventReceiveError> {
        let queued = {
            let mut queue_state = self.lock_queue_state();
            loop {
                if let Some(queued) = self.pop_front_while_locked(&mut queue_state) {
                    break queued;
                }
                if !queue_state.active {
                    return Err(EngineEventReceiveError::Disconnected);
                }
                self.diagnostics.record_receiver_waiting();
                queue_state = self
                    .queue_ready
                    .wait(queue_state)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                self.diagnostics.record_receiver_resumed();
            }
        };
        Ok(self.finalize_dequeued_event(queued))
    }

    fn try_receive(&self) -> Result<Arc<EngineEvent>, EngineEventTryReceiveError> {
        let result = {
            let mut queue_state = self.lock_queue_state();
            if let Some(queued) = self.pop_front_while_locked(&mut queue_state) {
                Ok(queued)
            } else if queue_state.active {
                Err(EngineEventTryReceiveError::Empty)
            } else {
                Err(EngineEventTryReceiveError::Disconnected)
            }
        };
        match result {
            Ok(queued) => Ok(self.finalize_dequeued_event(queued)),
            Err(error) => Err(error),
        }
    }

    fn receive_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Arc<EngineEvent>, EngineEventReceiveTimeoutError> {
        let deadline = Instant::now().checked_add(timeout);
        let queued = {
            let mut queue_state = self.lock_queue_state();
            loop {
                if let Some(queued) = self.pop_front_while_locked(&mut queue_state) {
                    break queued;
                }
                if !queue_state.active {
                    return Err(EngineEventReceiveTimeoutError::Disconnected);
                }
                let (remaining, timeout_expires_after_wait) = match deadline {
                    Some(deadline) => {
                        let Some(remaining) = deadline.checked_duration_since(Instant::now())
                        else {
                            return Err(EngineEventReceiveTimeoutError::Timeout);
                        };
                        (remaining, true)
                    }
                    None => (OVERFLOW_TIMEOUT_WAIT_SLICE, false),
                };
                self.diagnostics.record_receiver_waiting();
                let (next_state, wait_result) = self
                    .queue_ready
                    .wait_timeout(queue_state, remaining)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                queue_state = next_state;
                self.diagnostics.record_receiver_resumed();
                if timeout_expires_after_wait
                    && wait_result.timed_out()
                    && queue_state.queue.is_empty()
                    && queue_state.active
                {
                    return Err(EngineEventReceiveTimeoutError::Timeout);
                }
            }
        };
        Ok(self.finalize_dequeued_event(queued))
    }

    fn pop_front_while_locked(
        &self,
        queue_state: &mut EventQueueState,
    ) -> Option<DequeuedEngineEvent> {
        let queued = queue_state.queue.pop_front()?;
        let queue_age = queued.queued_at.map(|queued_at| queued_at.elapsed());
        self.diagnostics.record_dequeued_depth();
        Some(DequeuedEngineEvent { queued, queue_age })
    }

    fn finalize_dequeued_event(&self, dequeued: DequeuedEngineEvent) -> Arc<EngineEvent> {
        self.diagnostics.record_dequeued_age(dequeued.queue_age);
        dequeued.queued.event
    }

    fn lock_queue_state(&self) -> MutexGuard<'_, EventQueueState> {
        self.queue_state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub(super) struct EventSubscription {
    state: Weak<EventBusState>,
    topic: Arc<EventTopic>,
    subscriber: Arc<EventSubscriber>,
}

impl EventSubscription {
    pub(super) fn new(
        state: Arc<EventBusState>,
        topic: Arc<EventTopic>,
        subscriber: Arc<EventSubscriber>,
    ) -> Self {
        Self {
            state: Arc::downgrade(&state),
            topic,
            subscriber,
        }
    }
}

impl EngineEventSubscription for EventSubscription {
    fn recv(&self) -> Result<Arc<EngineEvent>, EngineEventReceiveError> {
        self.subscriber.receive()
    }

    fn try_recv(&self) -> Result<Arc<EngineEvent>, EngineEventTryReceiveError> {
        self.subscriber.try_receive()
    }

    fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Arc<EngineEvent>, EngineEventReceiveTimeoutError> {
        self.subscriber.receive_timeout(timeout)
    }
}

impl Drop for EventSubscription {
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            state.unsubscribe(&self.topic, &self.subscriber);
        } else {
            self.subscriber.deactivate_and_drain();
        }
    }
}
