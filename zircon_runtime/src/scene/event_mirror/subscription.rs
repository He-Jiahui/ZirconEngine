use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

#[cfg(test)]
use std::time::Duration;

use serde::Serialize;

use crate::scene::ecs::{Event, EventObserverHandle};
use crate::scene::World;

use super::{RuntimeEventMirrorError, RuntimeEventMirrorRegistration};

pub(crate) const RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS: usize = 64;
pub(crate) const RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES: usize = 128 * 1024;
pub(crate) const RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS: usize = 16 * 1024;
pub(crate) const RUNTIME_EVENT_MIRROR_QUEUE_MAX_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;

trait ErasedRuntimeEventMirrorSubscription: Send + Sync {
    fn connect(&mut self, world: &mut World) -> bool;
    fn disconnect(&mut self, world: &mut World) -> bool;
    fn drain_payloads(
        &mut self,
    ) -> Result<RuntimeEventMirrorDrainPage, RuntimeEventMirrorQueueFailure>;
    fn drain_payloads_up_to(
        &mut self,
        max_deliveries: usize,
    ) -> Result<RuntimeEventMirrorDrainPage, RuntimeEventMirrorQueueFailure>;
}

struct TypedRuntimeEventMirrorSubscription<E> {
    observer: Option<EventObserverHandle>,
    queue: Arc<Mutex<RuntimeEventMirrorQueue>>,
    _marker: PhantomData<fn() -> E>,
}

impl<E> Default for TypedRuntimeEventMirrorSubscription<E> {
    fn default() -> Self {
        Self {
            observer: None,
            queue: Arc::new(Mutex::new(RuntimeEventMirrorQueue::default())),
            _marker: PhantomData,
        }
    }
}

impl<E> ErasedRuntimeEventMirrorSubscription for TypedRuntimeEventMirrorSubscription<E>
where
    E: Event + Serialize,
{
    fn connect(&mut self, world: &mut World) -> bool {
        if self.observer.is_some() {
            return false;
        }
        let queue = self.queue.clone();
        let Some(observer) = world.observe_event_delivery::<E, _>(move |event| {
            lock_runtime_event_mirror_queue(&queue).publish(event)
        }) else {
            return false;
        };
        self.observer = Some(observer);
        true
    }

    fn disconnect(&mut self, world: &mut World) -> bool {
        let Some(observer) = self.observer else {
            return false;
        };
        if !world.unobserve_event_delivery(observer) {
            return false;
        }
        self.observer = None;
        true
    }

    fn drain_payloads(
        &mut self,
    ) -> Result<RuntimeEventMirrorDrainPage, RuntimeEventMirrorQueueFailure> {
        self.drain_payloads_up_to(RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS)
    }

    fn drain_payloads_up_to(
        &mut self,
        max_deliveries: usize,
    ) -> Result<RuntimeEventMirrorDrainPage, RuntimeEventMirrorQueueFailure> {
        lock_runtime_event_mirror_queue(&self.queue).drain_page(max_deliveries)
    }
}

#[derive(Default)]
/// Subscription-owned transport authority. Payloads are serialized once at the producer
/// boundary and retained here until a bounded ABI page commits them.
struct RuntimeEventMirrorQueue {
    pending: VecDeque<QueuedRuntimeEventPayload>,
    pending_payload_bytes: usize,
    failure: Option<RuntimeEventMirrorQueueFailure>,
}

impl RuntimeEventMirrorQueue {
    fn publish<E>(&mut self, event: &E) -> bool
    where
        E: Serialize,
    {
        let payload = match serde_json::to_vec(event) {
            Ok(payload) => payload,
            Err(error) => {
                self.record_failure(RuntimeEventMirrorQueueFailure::Serialize(error.to_string()));
                return false;
            }
        };
        let encoded_payload_bytes = payload.len();
        if encoded_payload_bytes > RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES {
            self.record_failure(RuntimeEventMirrorQueueFailure::PayloadTooLarge {
                payload_bytes: encoded_payload_bytes,
                max_payload_bytes: RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES,
            });
            return false;
        }
        let Some(next_payload_bytes) = self
            .pending_payload_bytes
            .checked_add(encoded_payload_bytes)
        else {
            self.record_overflow();
            return false;
        };
        if self.pending.len() >= RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS
            || next_payload_bytes > RUNTIME_EVENT_MIRROR_QUEUE_MAX_PAYLOAD_BYTES
        {
            self.record_overflow();
            return false;
        }
        self.pending.push_back(QueuedRuntimeEventPayload {
            payload,
            enqueued_at: Instant::now(),
        });
        self.pending_payload_bytes = next_payload_bytes;
        true
    }

    fn drain_page(
        &mut self,
        max_deliveries: usize,
    ) -> Result<RuntimeEventMirrorDrainPage, RuntimeEventMirrorQueueFailure> {
        if let Some(failure) = self.failure.take() {
            return Err(failure);
        }
        let max_deliveries = max_deliveries.min(RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS);
        let mut page = Vec::with_capacity(self.pending.len().min(max_deliveries));
        let mut page_payload_bytes = 0_usize;
        while page.len() < max_deliveries {
            let Some(next) = self.pending.front() else {
                break;
            };
            if page_payload_bytes + next.payload.len() > RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES
            {
                break;
            }
            let next = self
                .pending
                .pop_front()
                .expect("runtime event mirror queue front must remain available");
            page_payload_bytes += next.payload.len();
            self.pending_payload_bytes -= next.payload.len();
            page.push(RuntimeEventMirrorPayload {
                bytes: next.payload,
            });
        }
        Ok(RuntimeEventMirrorDrainPage {
            payloads: page,
            remaining_deliveries: u32::try_from(self.pending.len())
                .expect("runtime event mirror queue count is bounded by u32 limits"),
            oldest_pending_age_millis: self.oldest_pending_age_millis(),
        })
    }

    fn oldest_pending_age_millis(&self) -> u64 {
        self.pending
            .front()
            .map(|payload| {
                u64::try_from(payload.enqueued_at.elapsed().as_millis()).unwrap_or(u64::MAX)
            })
            .unwrap_or_default()
    }

    fn record_overflow(&mut self) {
        self.record_failure(RuntimeEventMirrorQueueFailure::Overflow {
            pending_events: self.pending.len(),
            pending_payload_bytes: self.pending_payload_bytes,
            max_events: RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS,
            max_payload_bytes: RUNTIME_EVENT_MIRROR_QUEUE_MAX_PAYLOAD_BYTES,
        });
    }

    fn record_failure(&mut self, failure: RuntimeEventMirrorQueueFailure) {
        if self.failure.is_none() {
            self.failure = Some(failure);
        }
    }
}

struct QueuedRuntimeEventPayload {
    payload: Vec<u8>,
    enqueued_at: Instant,
}

/// A payload stays in its producer-side JSON representation until a consumer chooses how to
/// materialize it. The runtime ABI encoder consumes these bytes directly, while the legacy
/// scene-facing drain keeps its existing `serde_json::Value` contract.
pub(crate) struct RuntimeEventMirrorPayload {
    bytes: Vec<u8>,
}

impl RuntimeEventMirrorPayload {
    pub(crate) fn json_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// A fixed-size delivery page plus the unconsumed subscription-owned backlog.
pub(crate) struct RuntimeEventMirrorDrainPage {
    pub(crate) payloads: Vec<RuntimeEventMirrorPayload>,
    pub(crate) remaining_deliveries: u32,
    pub(crate) oldest_pending_age_millis: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RuntimeEventMirrorQueueFailure {
    Serialize(String),
    PayloadTooLarge {
        payload_bytes: usize,
        max_payload_bytes: usize,
    },
    Overflow {
        pending_events: usize,
        pending_payload_bytes: usize,
        max_events: usize,
        max_payload_bytes: usize,
    },
}

pub struct RuntimeEventMirrorSubscription {
    registration: Option<RuntimeEventMirrorRegistration>,
    erased: Box<dyn ErasedRuntimeEventMirrorSubscription>,
    connected: bool,
}

impl RuntimeEventMirrorSubscription {
    pub fn descriptor(&self) -> &super::RuntimeEventMirrorDescriptor {
        self.registration().descriptor()
    }

    pub(crate) fn typed<E>() -> Self
    where
        E: Event + Serialize,
    {
        Self {
            registration: None,
            erased: Box::<TypedRuntimeEventMirrorSubscription<E>>::default(),
            connected: false,
        }
    }

    pub(crate) fn attach_registration(&mut self, registration: RuntimeEventMirrorRegistration) {
        self.registration = Some(registration);
    }

    pub(crate) fn connect(&mut self, world: &mut World) -> bool {
        let connected = self.erased.connect(world);
        self.connected |= connected;
        connected
    }

    pub(crate) fn disconnect(&mut self, world: &mut World) -> bool {
        let disconnected = self.erased.disconnect(world);
        if disconnected {
            self.connected = false;
        }
        disconnected
    }

    pub(crate) fn registration(&self) -> &RuntimeEventMirrorRegistration {
        self.registration
            .as_ref()
            .expect("runtime event mirror subscription has registration")
    }

    pub(crate) fn drain_payloads(
        &mut self,
    ) -> Result<RuntimeEventMirrorDrainPage, RuntimeEventMirrorError> {
        if !self.connected {
            let event_id = self.registration().descriptor().event_id.clone();
            return Err(RuntimeEventMirrorError::Disconnected { event_id });
        }
        match self.erased.drain_payloads() {
            Ok(page) => Ok(page),
            Err(failure) => Err(self.runtime_error_from_queue_failure(failure)),
        }
    }

    pub(crate) fn drain_payloads_up_to(
        &mut self,
        max_deliveries: usize,
    ) -> Result<RuntimeEventMirrorDrainPage, RuntimeEventMirrorError> {
        if !self.connected {
            let event_id = self.registration().descriptor().event_id.clone();
            return Err(RuntimeEventMirrorError::Disconnected { event_id });
        }
        match self.erased.drain_payloads_up_to(max_deliveries) {
            Ok(page) => Ok(page),
            Err(failure) => Err(self.runtime_error_from_queue_failure(failure)),
        }
    }

    pub(crate) fn drain(&mut self) -> Result<Vec<serde_json::Value>, RuntimeEventMirrorError> {
        let event_id = self.registration().descriptor().event_id.clone();
        self.drain_payloads()?
            .payloads
            .into_iter()
            .map(|payload| {
                serde_json::from_slice(payload.json_bytes()).map_err(|error| {
                    RuntimeEventMirrorError::Serialize {
                        event_id: event_id.clone(),
                        message: error.to_string(),
                    }
                })
            })
            .collect()
    }

    fn runtime_error_from_queue_failure(
        &self,
        failure: RuntimeEventMirrorQueueFailure,
    ) -> RuntimeEventMirrorError {
        match failure {
            RuntimeEventMirrorQueueFailure::Serialize(message) => {
                RuntimeEventMirrorError::Serialize {
                    event_id: self.registration().descriptor().event_id.clone(),
                    message,
                }
            }
            RuntimeEventMirrorQueueFailure::PayloadTooLarge {
                payload_bytes,
                max_payload_bytes,
            } => RuntimeEventMirrorError::PayloadTooLarge {
                event_id: self.registration().descriptor().event_id.clone(),
                payload_bytes,
                max_payload_bytes,
            },
            RuntimeEventMirrorQueueFailure::Overflow {
                pending_events,
                pending_payload_bytes,
                max_events,
                max_payload_bytes,
            } => RuntimeEventMirrorError::QueueOverflow {
                event_id: self.registration().descriptor().event_id.clone(),
                pending_events,
                pending_payload_bytes,
                max_events,
                max_payload_bytes,
            },
        }
    }
}

fn lock_runtime_event_mirror_queue(
    queue: &Arc<Mutex<RuntimeEventMirrorQueue>>,
) -> MutexGuard<'_, RuntimeEventMirrorQueue> {
    queue
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drain_page_reports_remaining_deliveries_and_oldest_pending_age() {
        let payload = vec![b'x'; RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES];
        let observed_at = Instant::now();
        let mut queue = RuntimeEventMirrorQueue {
            pending: VecDeque::from([
                QueuedRuntimeEventPayload {
                    payload: payload.clone(),
                    enqueued_at: observed_at
                        .checked_sub(Duration::from_millis(12))
                        .expect("test instant supports a recent offset"),
                },
                QueuedRuntimeEventPayload {
                    payload,
                    enqueued_at: observed_at
                        .checked_sub(Duration::from_millis(7))
                        .expect("test instant supports a recent offset"),
                },
            ]),
            pending_payload_bytes: RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES * 2,
            failure: None,
        };

        let page = queue
            .drain_page(RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS)
            .expect("queue drain page");

        assert_eq!(page.payloads.len(), 1);
        assert_eq!(page.remaining_deliveries, 1);
        assert!(page.oldest_pending_age_millis >= 7);
    }

    #[test]
    fn drain_page_limit_leaves_unconsumed_payloads_in_subscription_authority() {
        let mut queue = RuntimeEventMirrorQueue {
            pending: VecDeque::from([
                QueuedRuntimeEventPayload {
                    payload: b"first".to_vec(),
                    enqueued_at: Instant::now(),
                },
                QueuedRuntimeEventPayload {
                    payload: b"second".to_vec(),
                    enqueued_at: Instant::now(),
                },
            ]),
            pending_payload_bytes: b"first".len() + b"second".len(),
            failure: None,
        };

        let first = queue.drain_page(1).expect("limited first queue page");
        assert_eq!(first.payloads.len(), 1);
        assert_eq!(first.remaining_deliveries, 1);

        let deferred = queue.drain_page(0).expect("zero-limit queue page");
        assert!(deferred.payloads.is_empty());
        assert_eq!(deferred.remaining_deliveries, 1);

        let second = queue.drain_page(1).expect("limited second queue page");
        assert_eq!(second.payloads.len(), 1);
        assert_eq!(second.remaining_deliveries, 0);
    }
}
