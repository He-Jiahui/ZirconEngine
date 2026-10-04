use super::admission::{Retained, RetentionLease};
use super::subscriber::QueuedEngineEvent;
use super::topic::EventBusState;
use super::EventBus;
use crate::core::framework::events::*;
use std::sync::Arc;

impl EventBus {
    /// Atomic queue admission. Rejection returns the original owned input to retry or fail.
    pub fn try_publish(
        &self,
        event: EngineEvent,
    ) -> Result<EngineEventPublishReceipt, EngineEventPublishRejected> {
        self.state.try_publish(event)
    }
    pub fn diagnostic_report(&self) -> EventBusDiagnosticsSnapshot {
        self.state.diagnostic_report()
    }
    pub fn retention_snapshot(&self) -> EventBusRetentionSnapshot {
        self.state.admission.snapshot()
    }
    pub fn close_admission(&self) -> EventBusCloseReceipt {
        self.state.close_admission()
    }
}

impl EventBusState {
    pub(super) fn try_publish(
        &self,
        event: EngineEvent,
    ) -> Result<EngineEventPublishReceipt, EngineEventPublishRejected> {
        let started = self.diagnostics.record_published_and_capture_time();
        let result = self.admit(&event);
        self.diagnostics.record_publish_duration(started);
        result.map_err(|reason| EngineEventPublishRejected { reason, event })
    }

    fn admit(
        &self,
        event: &EngineEvent,
    ) -> Result<EngineEventPublishReceipt, EngineEventPublishRejection> {
        {
            let ledger = self.admission.lock();
            if ledger.poisoned {
                return Err(EngineEventPublishRejection::Poisoned);
            }
            if ledger.closed {
                return Err(EngineEventPublishRejection::Closed);
            }
            if event.topic.is_empty()
                || event.topic.len() > ledger.limits.max_topic_name_bytes.get()
            {
                return Err(EngineEventPublishRejection::InvalidTopic);
            }
        }
        {
            let topic = self.topic(&event.topic);
            if self.admission.lock().poisoned {
                return Err(EngineEventPublishRejection::Poisoned);
            }
            let Some(topic) = topic else {
                return Err(EngineEventPublishRejection::NoSubscribers);
            };
            let _delivery = if let Some(delivery) = topic.try_lock_delivery() {
                delivery
            } else {
                let wait_started = self.diagnostics.capture_contention_time();
                self.diagnostics.record_publisher_waiting();
                let delivery = topic.lock_delivery();
                self.diagnostics.record_publisher_resumed(wait_started);
                delivery
            };
            if topic.delivery_is_poisoned() {
                self.admission.fail_closed();
                return Err(EngineEventPublishRejection::Poisoned);
            }
            let subscribers = topic.snapshot_subscribers();
            if subscribers.is_empty() {
                return Err(EngineEventPublishRejection::NoSubscribers);
            }
        }
        let limits = self.admission.lock().limits;
        // No user serializer or payload destructor runs under queue/ledger locks.
        let frozen = Arc::new(super::frozen::freeze(event, limits)?);
        let bytes = frozen
            .topic
            .len()
            .checked_add(frozen.payload.len())
            .ok_or(EngineEventPublishRejection::PayloadTooLarge)?;
        let topic = self.topic(&event.topic);
        if self.admission.lock().poisoned {
            return Err(EngineEventPublishRejection::Poisoned);
        }
        let Some(topic) = topic else {
            return Err(EngineEventPublishRejection::NoSubscribers);
        };
        let _delivery = if let Some(delivery) = topic.try_lock_delivery() {
            delivery
        } else {
            let wait_started = self.diagnostics.capture_contention_time();
            self.diagnostics.record_publisher_waiting();
            let delivery = topic.lock_delivery();
            self.diagnostics.record_publisher_resumed(wait_started);
            delivery
        };
        if topic.delivery_is_poisoned() {
            self.admission.fail_closed();
            return Err(EngineEventPublishRejection::Poisoned);
        }
        let subscribers = topic.snapshot_subscribers();
        if subscribers.is_empty() {
            return Err(EngineEventPublishRejection::NoSubscribers);
        }

        for subscriber in subscribers.iter() {
            if subscriber.is_poisoned() {
                self.admission.fail_closed();
                return Err(EngineEventPublishRejection::Poisoned);
            }
        }
        // Snapshots are sorted by monotonic registration ID. Global commit
        // serialization protects capacity while displaced envelopes retire.
        let _commit = self.admission.lock_commit();
        let mut queues: Vec<_> = subscribers
            .iter()
            .map(|subscriber| subscriber.lock_queue_state())
            .collect();
        let mut ledger = self.admission.lock();
        if ledger.poisoned {
            return Err(EngineEventPublishRejection::Poisoned);
        }
        if ledger.closed {
            return Err(EngineEventPublishRejection::Closed);
        }
        let mut removals = vec![Retained::default(); subscribers.len()];
        for (index, subscriber) in subscribers.iter().enumerate() {
            if !queues[index].active {
                return Err(EngineEventPublishRejection::Closed);
            }
            let Some(account) = ledger.subscribers.get(&subscriber.id()) else {
                return Err(EngineEventPublishRejection::Poisoned);
            };
            let quota = subscriber.policy.limits();
            if bytes > quota.max_bytes.get() {
                return Err(EngineEventPublishRejection::Backpressured {
                    scope: EventBudgetScope::Subscriber,
                });
            }
            loop {
                let projected = account
                    .retained
                    .subtract(removals[index].events, removals[index].bytes)
                    .and_then(|retained| retained.add(1, bytes));
                if projected.is_some_and(|retained| retained.fits(quota)) {
                    break;
                }
                if !subscriber.policy.allows_replacement() {
                    return Err(EngineEventPublishRejection::Backpressured {
                        scope: EventBudgetScope::Subscriber,
                    });
                }
                let Some(old) = queues[index].queue.get(removals[index].events) else {
                    return Err(EngineEventPublishRejection::Backpressured {
                        scope: EventBudgetScope::Subscriber,
                    });
                };
                removals[index] = removals[index]
                    .add(1, old.event.retained_bytes())
                    .ok_or(EngineEventPublishRejection::Poisoned)?;
            }
        }
        let fanout_bytes = bytes.checked_mul(subscribers.len()).ok_or(
            EngineEventPublishRejection::Backpressured {
                scope: EventBudgetScope::Global,
            },
        )?;
        let mut removed = Retained::default();
        for removal in &removals {
            removed = removed
                .add(removal.events, removal.bytes)
                .ok_or(EngineEventPublishRejection::Poisoned)?;
        }
        loop {
            let global = ledger
                .retained
                .subtract(removed.events, removed.bytes)
                .and_then(|r| r.add(subscribers.len(), fanout_bytes));
            let topic_retained = ledger
                .topics
                .get(&event.topic)
                .map(|a| a.retained)
                .ok_or(EngineEventPublishRejection::Poisoned)?;
            let topic_projected = topic_retained
                .subtract(removed.events, removed.bytes)
                .and_then(|r| r.add(subscribers.len(), fanout_bytes));
            let topic_fits = topic_projected.is_some_and(|r| r.fits(ledger.limits.topic));
            let global_fits = global.is_some_and(|r| r.fits(ledger.limits.global));
            if topic_fits && global_fits {
                break;
            }
            let mut evicted = false;
            for (index, subscriber) in subscribers.iter().enumerate() {
                if subscriber.policy.allows_replacement() {
                    if let Some(old) = queues[index].queue.get(removals[index].events) {
                        let old_bytes = old.event.retained_bytes();
                        removals[index] = removals[index]
                            .add(1, old_bytes)
                            .ok_or(EngineEventPublishRejection::Poisoned)?;
                        removed = removed
                            .add(1, old_bytes)
                            .ok_or(EngineEventPublishRejection::Poisoned)?;
                        evicted = true;
                        break;
                    }
                }
            }
            if !evicted {
                return Err(EngineEventPublishRejection::Backpressured {
                    scope: if !topic_fits {
                        EventBudgetScope::Topic
                    } else {
                        EventBudgetScope::Global
                    },
                });
            }
        }
        // Preallocate everything fallible before changing the ledger or any physical queue.
        let mut displaced = Vec::with_capacity(removed.events);
        let prepared: Vec<_> = subscribers
            .iter()
            .map(|s| EngineEventDelivery {
                event: Arc::clone(&frozen),
                lease: Arc::new(RetentionLease::new(
                    Arc::clone(&self.admission),
                    s.id(),
                    bytes,
                )),
            })
            .collect();
        for (index, queue) in queues.iter_mut().enumerate() {
            if removals[index].events == 0 {
                queue.queue.try_reserve_exact(1).map_err(|_| {
                    EngineEventPublishRejection::Backpressured {
                        scope: EventBudgetScope::Global,
                    }
                })?;
            }
        }
        // From here through ledger+queue commit there is no user code or fallible allocation.
        for (index, subscriber) in subscribers.iter().enumerate() {
            for _ in 0..removals[index].events {
                let old = queues[index]
                    .queue
                    .pop_front()
                    .expect("preflight reserved queued displacement");
                subscriber.diagnostics.record_evicted(old.queued_at);
                displaced.push(old);
            }
        }
        // Preflight above is the admission linearization point. Close may
        // close the gate now but waits on this topic's delivery lock to drain.
        // Already-admitted work finishes without any further failure path.
        drop(ledger);
        queues.clear();
        drop(displaced);
        for subscriber in subscribers.iter() {
            queues.push(subscriber.lock_queue_state());
        }
        let mut ledger = self.admission.lock();
        for ((subscriber, queue), delivery) in
            subscribers.iter().zip(queues.iter_mut()).zip(prepared)
        {
            ledger.charge(subscriber.id(), 1, bytes);
            delivery.lease.activate_charge();
            queue.queue.push_back(QueuedEngineEvent {
                event: delivery,
                queued_at: subscriber.diagnostics.record_enqueued_and_capture_time(),
            });
        }
        let receipt = EngineEventPublishReceipt {
            subscribers: subscribers.len(),
            replaced_events: removed.events,
            replaced_bytes: removed.bytes,
            admitted_bytes: fanout_bytes,
        };
        drop(ledger);
        drop(queues);
        drop(_commit);
        drop(_delivery);
        for subscriber in subscribers.iter() {
            subscriber.notify_ready();
        }
        Ok(receipt)
    }
}

#[cfg(test)]
#[path = "publish/tests/single_disconnect_inline_tests.rs"]
mod single_disconnect_inline_tests;

#[cfg(test)]
#[path = "publish/tests/optimization_batch_jm_runtime652_tests.rs"]
mod optimization_batch_jm_runtime652_tests;
