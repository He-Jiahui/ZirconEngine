use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use zircon_runtime_interface::{
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle,
};

use crate::core::gateway::{EditorRuntimeGateway, EditorRuntimeGatewayHandle};

use super::{
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerRegistration,
    EditorRuntimeEventConsumerRegistry, EditorRuntimeEventPumpBudget, EditorRuntimeEventPumpReport,
};

const EXECUTION_IDLE: u8 = 0;
const EXECUTION_PUMP: u8 = 1;
const EXECUTION_LIFECYCLE: u8 = 2;

struct ActiveConsumer {
    registration: EditorRuntimeEventConsumerRegistration,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    generation: u64,
    last_sequence: Option<u64>,
    pending: VecDeque<ZrRuntimePluginEventDeliveryV1>,
}

#[derive(Clone)]
struct ActiveConsumerSnapshot {
    consumer_id: String,
    registration: EditorRuntimeEventConsumerRegistration,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    generation: u64,
}

#[derive(Clone)]
struct ActiveConsumerIdentity {
    consumer_id: String,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    generation: u64,
}

pub struct EditorRuntimeEventConsumerHost {
    gateway: EditorRuntimeGatewayHandle,
    registry: Mutex<EditorRuntimeEventConsumerRegistry>,
    active: Mutex<BTreeMap<String, ActiveConsumer>>,
    play_session_id: Mutex<Option<u64>>,
    next_consumer_generation: AtomicU64,
    round_robin_cursor: Mutex<Option<String>>,
    last_pump_report: Mutex<EditorRuntimeEventPumpReport>,
    execution_state: AtomicU8,
}

impl Default for EditorRuntimeEventConsumerHost {
    fn default() -> Self {
        Self::new(EditorRuntimeGatewayHandle::detached())
    }
}

impl EditorRuntimeEventConsumerHost {
    pub fn new(gateway: EditorRuntimeGatewayHandle) -> Self {
        Self {
            gateway,
            registry: Mutex::new(EditorRuntimeEventConsumerRegistry::default()),
            active: Mutex::new(BTreeMap::new()),
            play_session_id: Mutex::new(None),
            next_consumer_generation: AtomicU64::new(0),
            round_robin_cursor: Mutex::new(None),
            last_pump_report: Mutex::new(EditorRuntimeEventPumpReport::default()),
            execution_state: AtomicU8::new(EXECUTION_IDLE),
        }
    }

    pub fn runtime_session_id(&self) -> u64 {
        self.gateway.session_handle().raw()
    }

    pub fn register(
        &self,
        registry: EditorRuntimeEventConsumerRegistry,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        self.registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .extend(registry)
    }

    pub(crate) fn prepare_registration(
        &self,
        registry: EditorRuntimeEventConsumerRegistry,
    ) -> Result<EditorRuntimeEventConsumerRegistry, EditorRuntimeEventConsumerError> {
        let mut candidate = self
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        candidate.extend(registry)?;
        Ok(candidate)
    }

    pub(crate) fn install_prepared_registration(
        &self,
        registry: EditorRuntimeEventConsumerRegistry,
    ) {
        *self
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = registry;
    }

    pub fn begin_play_session(
        &self,
        play_session_id: u64,
        enabled_capabilities: &[String],
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let _lifecycle_guard =
            LifecycleExecutionGuard::enter(&self.execution_state, "begin play session")?;
        let mut session = self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(active) = *session {
            return Err(EditorRuntimeEventConsumerError::SessionAlreadyActive {
                play_session_id: active,
            });
        }
        *session = Some(play_session_id);
        drop(session);

        if let Err(error) = self.reconcile_enabled_capabilities_inner(enabled_capabilities) {
            return Err(match self.end_play_session_inner(play_session_id) {
                Ok(()) => error,
                Err(cleanup) => EditorRuntimeEventConsumerError::with_cleanup(
                    "begin runtime event consumer session",
                    error,
                    cleanup,
                ),
            });
        }
        Ok(())
    }

    pub fn reconcile_enabled_capabilities(
        &self,
        enabled_capabilities: &[String],
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let _lifecycle_guard = LifecycleExecutionGuard::enter(
            &self.execution_state,
            "reconcile enabled capabilities",
        )?;
        self.reconcile_enabled_capabilities_inner(enabled_capabilities)
    }

    fn reconcile_enabled_capabilities_inner(
        &self,
        enabled_capabilities: &[String],
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let play_session_id = self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .ok_or(EditorRuntimeEventConsumerError::NoActiveSession)?;
        let registrations = self
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .registrations()
            .cloned()
            .collect::<Vec<_>>();
        let desired = registrations
            .into_iter()
            .filter(|registration| {
                let required = &registration.manifest().required_capability;
                required.is_empty()
                    || enabled_capabilities
                        .iter()
                        .any(|capability| capability == required)
            })
            .map(|registration| (registration.manifest().consumer_id.clone(), registration))
            .collect::<BTreeMap<_, _>>();
        let gateway = self.gateway.clone();
        let removed = {
            let active = self
                .active
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            active
                .iter()
                .filter(|(consumer_id, _)| !desired.contains_key(*consumer_id))
                .map(|(consumer_id, consumer)| ActiveConsumerIdentity {
                    consumer_id: consumer_id.clone(),
                    subscription: consumer.subscription,
                    generation: consumer.generation,
                })
                .collect::<Vec<_>>()
        };
        let mut first_error = None;
        for identity in removed {
            match unsubscribe_consumer(&gateway, &identity.consumer_id, identity.subscription) {
                Ok(()) => {
                    if let Some(consumer) = self.remove_active_consumer(&identity) {
                        consumer.registration.end_session(play_session_id);
                    }
                }
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }

        let existing = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut added = Vec::new();
        for (consumer_id, registration) in desired {
            if existing.contains(&consumer_id) {
                continue;
            }
            let manifest = registration.manifest().clone();
            let subscription = match gateway
                .subscribe_plugin_event(&manifest.event_id, &manifest.payload_schema)
            {
                Ok(Some(subscription)) => subscription,
                Ok(None) => {
                    let error = EditorRuntimeEventConsumerError::Unsupported {
                        consumer_id: manifest.consumer_id.clone(),
                    };
                    return Err(
                        match self.rollback_added_consumers(&gateway, &added, play_session_id) {
                            Some(cleanup) => EditorRuntimeEventConsumerError::with_cleanup(
                                "reconcile runtime event consumers",
                                error,
                                cleanup,
                            ),
                            None => error,
                        },
                    );
                }
                Err(message) => {
                    let error = EditorRuntimeEventConsumerError::Gateway {
                        consumer_id: manifest.consumer_id.clone(),
                        message: message.to_string(),
                    };
                    return Err(
                        match self.rollback_added_consumers(&gateway, &added, play_session_id) {
                            Some(cleanup) => EditorRuntimeEventConsumerError::with_cleanup(
                                "reconcile runtime event consumers",
                                error,
                                cleanup,
                            ),
                            None => error,
                        },
                    );
                }
            };
            registration.begin_session(play_session_id);
            let identity = ActiveConsumerIdentity {
                consumer_id: manifest.consumer_id,
                subscription,
                generation: self.allocate_consumer_generation(),
            };
            self.active
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert(
                    identity.consumer_id.clone(),
                    ActiveConsumer {
                        registration,
                        subscription,
                        generation: identity.generation,
                        last_sequence: None,
                        pending: VecDeque::new(),
                    },
                );
            added.push(identity);
        }
        Ok(())
    }

    pub fn pump(&self) -> Result<usize, EditorRuntimeEventConsumerError> {
        Ok(self
            .pump_with_budget(EditorRuntimeEventPumpBudget::default())?
            .applied())
    }

    pub fn pump_with_budget(
        &self,
        budget: EditorRuntimeEventPumpBudget,
    ) -> Result<EditorRuntimeEventPumpReport, EditorRuntimeEventConsumerError> {
        let Some(_pump_guard) = PumpExecutionGuard::enter(&self.execution_state) else {
            return Ok(EditorRuntimeEventPumpReport::default());
        };
        let play_session_id = self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .unwrap_or_default();
        if play_session_id == 0 {
            self.store_pump_report(EditorRuntimeEventPumpReport::default());
            return Ok(EditorRuntimeEventPumpReport::default());
        }
        let gateway = self.gateway.clone();
        let runtime_session_id = gateway.session_handle().raw();
        let snapshots = self.snapshot_active_consumers();
        let started = Instant::now();
        let mut report = EditorRuntimeEventPumpReport::default();
        let mut runtime_drain_samples = Vec::with_capacity(snapshots.len());
        let mut decode_samples = Vec::with_capacity(snapshots.len());
        let mut visited_any = false;
        let mut first_error = None;

        for snapshot in &snapshots {
            if report.applied() >= budget.max_events() || started.elapsed() >= budget.max_elapsed()
            {
                break;
            }
            visited_any = true;
            let page = match gateway.drain_plugin_events(snapshot.subscription) {
                Ok(page) => page,
                Err(message) => {
                    first_error.get_or_insert(EditorRuntimeEventConsumerError::Gateway {
                        consumer_id: snapshot.consumer_id.clone(),
                        message: message.to_string(),
                    });
                    continue;
                }
            };
            report.record_drained_page(
                page.deliveries().len(),
                page.encoded_bytes(),
                page.runtime_drain_elapsed(),
                page.decode_elapsed(),
                page.runtime_remaining_deliveries(),
                page.runtime_oldest_pending_age_millis(),
            );
            runtime_drain_samples.push(page.runtime_drain_elapsed());
            decode_samples.push(page.decode_elapsed());
            report.record_dropped(self.append_drained_deliveries(snapshot, page.into_deliveries()));

            let mut applied_for_consumer = 0;
            while report.applied() < budget.max_events()
                && applied_for_consumer < budget.max_events_per_consumer()
                && started.elapsed() < budget.max_elapsed()
            {
                let Some((delivery, last_sequence)) = self.take_next_delivery(snapshot) else {
                    break;
                };
                if let Err(error) =
                    validate_delivery(snapshot, runtime_session_id, last_sequence, &delivery)
                {
                    report.record_dropped(1);
                    first_error.get_or_insert(error);
                    break;
                }

                let sequence = delivery.sequence;
                let callback_started = Instant::now();
                let apply_result =
                    snapshot
                        .registration
                        .consume(play_session_id, sequence, delivery.payload);
                let callback_elapsed = callback_started.elapsed();
                if let Err(source) = apply_result {
                    report.record_dropped(1);
                    first_error.get_or_insert(EditorRuntimeEventConsumerError::Payload {
                        consumer_id: snapshot.consumer_id.clone(),
                        source,
                    });
                    break;
                }

                report.record_applied(callback_elapsed, budget.slow_callback_threshold());
                applied_for_consumer += 1;
                let committed = self.commit_delivery_sequence(snapshot, sequence);
                debug_assert!(committed, "lifecycle owner changed during an active pump");
            }
        }
        report.set_drain_percentiles(
            p95_duration(&mut runtime_drain_samples),
            p95_duration(&mut decode_samples),
        );
        self.advance_round_robin_start(&snapshots, visited_any);
        self.finish_pump_report(&mut report);
        first_error.map_or(Ok(report), Err)
    }

    pub fn last_pump_report(&self) -> EditorRuntimeEventPumpReport {
        *self
            .last_pump_report
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn end_play_session(
        &self,
        play_session_id: u64,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let _lifecycle_guard =
            LifecycleExecutionGuard::enter(&self.execution_state, "end play session")?;
        self.end_play_session_inner(play_session_id)
    }

    fn end_play_session_inner(
        &self,
        play_session_id: u64,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let active_play_session_id = *self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if active_play_session_id != Some(play_session_id) {
            return Err(EditorRuntimeEventConsumerError::RuntimeSessionMismatch {
                expected: active_play_session_id.unwrap_or_default(),
                actual: play_session_id,
            });
        }
        let gateway = self.gateway.clone();
        let consumers = {
            let active = self
                .active
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            active
                .iter()
                .map(|(consumer_id, consumer)| ActiveConsumerIdentity {
                    consumer_id: consumer_id.clone(),
                    subscription: consumer.subscription,
                    generation: consumer.generation,
                })
                .collect::<Vec<_>>()
        };
        let mut first_error = None;
        for identity in consumers {
            match unsubscribe_consumer(&gateway, &identity.consumer_id, identity.subscription) {
                Ok(()) => {
                    if let Some(consumer) = self.remove_active_consumer(&identity) {
                        consumer.registration.end_session(play_session_id);
                    }
                }
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        if self.active_consumer_count() == 0 {
            *self
                .play_session_id
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
        }
        first_error.map_or(Ok(()), Err)
    }

    pub fn active_consumer_count(&self) -> usize {
        self.active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub fn active_play_session_id(&self) -> Option<u64> {
        *self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn allocate_consumer_generation(&self) -> u64 {
        self.next_consumer_generation
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1)
    }

    fn remove_active_consumer(&self, identity: &ActiveConsumerIdentity) -> Option<ActiveConsumer> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let matches = active.get(&identity.consumer_id).is_some_and(|consumer| {
            consumer.generation == identity.generation
                && consumer.subscription == identity.subscription
        });
        matches
            .then(|| active.remove(&identity.consumer_id))
            .flatten()
    }

    fn rollback_added_consumers(
        &self,
        gateway: &dyn EditorRuntimeGateway,
        added: &[ActiveConsumerIdentity],
        play_session_id: u64,
    ) -> Option<EditorRuntimeEventConsumerError> {
        let mut first_error = None;
        for identity in added {
            match unsubscribe_consumer(gateway, &identity.consumer_id, identity.subscription) {
                Ok(()) => {
                    if let Some(consumer) = self.remove_active_consumer(identity) {
                        consumer.registration.end_session(play_session_id);
                    }
                }
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        first_error
    }

    fn snapshot_active_consumers(&self) -> Vec<ActiveConsumerSnapshot> {
        let active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut snapshots = active
            .iter()
            .map(|(consumer_id, consumer)| ActiveConsumerSnapshot {
                consumer_id: consumer_id.clone(),
                registration: consumer.registration.clone(),
                subscription: consumer.subscription,
                generation: consumer.generation,
            })
            .collect::<Vec<_>>();
        drop(active);

        let cursor = self
            .round_robin_cursor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(index) = cursor.and_then(|cursor| {
            snapshots
                .iter()
                .position(|snapshot| snapshot.consumer_id == cursor)
        }) {
            snapshots.rotate_left(index);
        }
        snapshots
    }

    fn append_drained_deliveries(
        &self,
        snapshot: &ActiveConsumerSnapshot,
        deliveries: Vec<ZrRuntimePluginEventDeliveryV1>,
    ) -> usize {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(consumer) = active.get_mut(&snapshot.consumer_id).filter(|consumer| {
            consumer.generation == snapshot.generation
                && consumer.subscription == snapshot.subscription
        }) else {
            return deliveries.len();
        };
        consumer.pending.extend(deliveries);
        0
    }

    fn take_next_delivery(
        &self,
        snapshot: &ActiveConsumerSnapshot,
    ) -> Option<(ZrRuntimePluginEventDeliveryV1, Option<u64>)> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let consumer = active.get_mut(&snapshot.consumer_id)?;
        (consumer.generation == snapshot.generation
            && consumer.subscription == snapshot.subscription)
            .then(|| {
                consumer
                    .pending
                    .pop_front()
                    .map(|delivery| (delivery, consumer.last_sequence))
            })
            .flatten()
    }

    fn commit_delivery_sequence(&self, snapshot: &ActiveConsumerSnapshot, sequence: u64) -> bool {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(consumer) = active.get_mut(&snapshot.consumer_id).filter(|consumer| {
            consumer.generation == snapshot.generation
                && consumer.subscription == snapshot.subscription
        }) else {
            return false;
        };
        consumer.last_sequence = Some(sequence);
        true
    }

    fn advance_round_robin_start(&self, snapshots: &[ActiveConsumerSnapshot], visited_any: bool) {
        if !visited_any || snapshots.is_empty() {
            return;
        }
        let next = snapshots
            .get(1 % snapshots.len())
            .map(|snapshot| snapshot.consumer_id.clone());
        *self
            .round_robin_cursor
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = next;
    }

    fn finish_pump_report(&self, report: &mut EditorRuntimeEventPumpReport) {
        let active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let queue_depth = active.values().map(|consumer| consumer.pending.len()).sum();
        let pending_sequence_span = active
            .values()
            .filter_map(|consumer| {
                let first = consumer.pending.front()?.sequence;
                let last = consumer.pending.back()?.sequence;
                Some(last.saturating_sub(first))
            })
            .max()
            .unwrap_or_default();
        drop(active);
        report.set_queue_pressure(queue_depth, pending_sequence_span);
        self.store_pump_report(*report);
    }

    fn store_pump_report(&self, report: EditorRuntimeEventPumpReport) {
        *self
            .last_pump_report
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = report;
    }
}

fn p95_duration(samples: &mut [Duration]) -> Duration {
    if samples.is_empty() {
        return Duration::ZERO;
    }
    samples.sort_unstable();
    let index = samples.len().saturating_mul(95).div_ceil(100) - 1;
    samples[index]
}

struct PumpExecutionGuard<'a> {
    execution_state: &'a AtomicU8,
}

impl<'a> PumpExecutionGuard<'a> {
    fn enter(execution_state: &'a AtomicU8) -> Option<Self> {
        execution_state
            .compare_exchange(
                EXECUTION_IDLE,
                EXECUTION_PUMP,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .ok()
            .map(|_| Self { execution_state })
    }
}

impl Drop for PumpExecutionGuard<'_> {
    fn drop(&mut self) {
        self.execution_state
            .store(EXECUTION_IDLE, Ordering::Release);
    }
}

struct LifecycleExecutionGuard<'a> {
    execution_state: &'a AtomicU8,
}

impl<'a> LifecycleExecutionGuard<'a> {
    fn enter(
        execution_state: &'a AtomicU8,
        operation: &'static str,
    ) -> Result<Self, EditorRuntimeEventConsumerError> {
        execution_state
            .compare_exchange(
                EXECUTION_IDLE,
                EXECUTION_LIFECYCLE,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .map(|_| Self { execution_state })
            .map_err(|_| EditorRuntimeEventConsumerError::LifecycleMutationBusy { operation })
    }
}

impl Drop for LifecycleExecutionGuard<'_> {
    fn drop(&mut self) {
        self.execution_state
            .store(EXECUTION_IDLE, Ordering::Release);
    }
}

fn validate_delivery(
    snapshot: &ActiveConsumerSnapshot,
    runtime_session_id: u64,
    last_sequence: Option<u64>,
    delivery: &ZrRuntimePluginEventDeliveryV1,
) -> Result<(), EditorRuntimeEventConsumerError> {
    let manifest = snapshot.registration.manifest();
    if delivery.play_session_id != runtime_session_id {
        return Err(EditorRuntimeEventConsumerError::WrongSession {
            consumer_id: snapshot.consumer_id.clone(),
            expected: runtime_session_id,
            actual: delivery.play_session_id,
        });
    }
    if delivery.subscription != snapshot.subscription {
        return Err(EditorRuntimeEventConsumerError::ForeignSubscription {
            consumer_id: snapshot.consumer_id.clone(),
        });
    }
    if delivery.event_id != manifest.event_id {
        return Err(EditorRuntimeEventConsumerError::EventMismatch {
            consumer_id: snapshot.consumer_id.clone(),
            expected: manifest.event_id.clone(),
            actual: delivery.event_id.clone(),
        });
    }
    if delivery.payload_schema != manifest.payload_schema {
        return Err(EditorRuntimeEventConsumerError::SchemaMismatch {
            consumer_id: snapshot.consumer_id.clone(),
            expected: manifest.payload_schema.clone(),
            actual: delivery.payload_schema.clone(),
        });
    }
    if last_sequence.is_some_and(|sequence| delivery.sequence <= sequence) {
        return Err(EditorRuntimeEventConsumerError::StaleSequence {
            consumer_id: snapshot.consumer_id.clone(),
            sequence: delivery.sequence,
        });
    }
    Ok(())
}

fn unsubscribe_consumer(
    gateway: &dyn EditorRuntimeGateway,
    consumer_id: &str,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
) -> Result<(), EditorRuntimeEventConsumerError> {
    match gateway.unsubscribe_plugin_event(subscription) {
        Ok(true) => Ok(()),
        Ok(false) => Err(EditorRuntimeEventConsumerError::Gateway {
            consumer_id: consumer_id.to_string(),
            message: "runtime did not remove the plugin event subscription".to_string(),
        }),
        Err(message) => Err(EditorRuntimeEventConsumerError::Gateway {
            consumer_id: consumer_id.to_string(),
            message: message.to_string(),
        }),
    }
}
