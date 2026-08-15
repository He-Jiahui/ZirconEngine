use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use zircon_runtime_interface::{
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle,
};

use crate::core::gateway::{EditorRuntimeGateway, EditorRuntimeGatewayHandle};

use super::{
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerRegistration,
    EditorRuntimeEventConsumerRegistry, EditorRuntimeEventPumpBudget, EditorRuntimeEventPumpReport,
};

mod execution_support;
mod round_robin;

use execution_support::{
    p95_duration, unsubscribe_consumer, validate_delivery, LifecycleExecutionGuard,
    PumpExecutionGuard,
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
    pending_encoded_bytes_upper_bound: usize,
    pending_since: Option<Instant>,
    last_observed_runtime_remaining_deliveries: Option<usize>,
    last_observed_runtime_oldest_pending_age_millis: Option<u64>,
    runtime_backlog_observed_at: Option<Instant>,
}

#[derive(Clone)]
struct ActiveConsumerSnapshot {
    consumer_id: String,
    registration: EditorRuntimeEventConsumerRegistration,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    generation: u64,
    has_pending: bool,
}

#[derive(Clone)]
struct ActiveConsumerIdentity {
    consumer_id: String,
    subscription: ZrRuntimePluginEventSubscriptionHandle,
    generation: u64,
}

struct PendingDeliveryBatch {
    deliveries: VecDeque<ZrRuntimePluginEventDeliveryV1>,
    last_sequence: Option<u64>,
    encoded_bytes_upper_bound: usize,
    pending_since: Option<Instant>,
}

/// Restores the locally owned delivery batch if the callback unwinds before the normal commit.
///
/// The delivery currently executing in a callback has transferred payload ownership and cannot be
/// replayed without changing the consumer ABI. Every later delivery and the last successful
/// sequence are nevertheless returned to the matching generation before unwinding continues.
struct PendingDeliveryBatchRestoreGuard<'a> {
    host: &'a EditorRuntimeEventConsumerHost,
    snapshot: &'a ActiveConsumerSnapshot,
    batch: Option<PendingDeliveryBatch>,
}

impl<'a> PendingDeliveryBatchRestoreGuard<'a> {
    fn new(
        host: &'a EditorRuntimeEventConsumerHost,
        snapshot: &'a ActiveConsumerSnapshot,
        batch: PendingDeliveryBatch,
    ) -> Self {
        Self {
            host,
            snapshot,
            batch: Some(batch),
        }
    }

    fn batch(&self) -> &PendingDeliveryBatch {
        self.batch
            .as_ref()
            .expect("pending delivery batch remains owned until it is restored")
    }

    fn batch_mut(&mut self) -> &mut PendingDeliveryBatch {
        self.batch
            .as_mut()
            .expect("pending delivery batch remains owned until it is restored")
    }

    fn restore(&mut self) -> bool {
        let Some(batch) = self.batch.take() else {
            return true;
        };
        self.host.restore_pending_batch(self.snapshot, batch)
    }
}

impl Drop for PendingDeliveryBatchRestoreGuard<'_> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
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
                        pending_encoded_bytes_upper_bound: 0,
                        pending_since: None,
                        last_observed_runtime_remaining_deliveries: None,
                        last_observed_runtime_oldest_pending_age_millis: None,
                        runtime_backlog_observed_at: None,
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
        let mut visited_consumer_count = 0;
        let mut first_error = None;

        for snapshot in &snapshots {
            if report.applied() >= budget.max_events() || started.elapsed() >= budget.max_elapsed()
            {
                break;
            }
            visited_consumer_count += 1;
            if !snapshot.has_pending {
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
                let page_encoded_bytes = page.encoded_bytes();
                let runtime_remaining_deliveries = page.runtime_remaining_deliveries();
                let runtime_oldest_pending_age_millis = page.runtime_oldest_pending_age_millis();
                report.record_drained_page(
                    page.deliveries().len(),
                    page_encoded_bytes,
                    page.runtime_drain_elapsed(),
                    page.decode_elapsed(),
                );
                runtime_drain_samples.push(page.runtime_drain_elapsed());
                decode_samples.push(page.decode_elapsed());
                report.record_dropped(self.append_drained_deliveries(
                    snapshot,
                    page.into_deliveries(),
                    page_encoded_bytes,
                    runtime_remaining_deliveries,
                    runtime_oldest_pending_age_millis,
                ));
            }

            let Some(batch) = self.take_pending_batch(snapshot) else {
                continue;
            };
            let mut pending = PendingDeliveryBatchRestoreGuard::new(self, snapshot, batch);
            let mut applied_for_consumer = 0;
            while report.applied() < budget.max_events()
                && applied_for_consumer < budget.max_events_per_consumer()
                && started.elapsed() < budget.max_elapsed()
            {
                let Some(delivery) = pending.batch_mut().deliveries.pop_front() else {
                    break;
                };
                if let Err(error) = validate_delivery(
                    snapshot,
                    runtime_session_id,
                    pending.batch().last_sequence,
                    &delivery,
                ) {
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
                pending.batch_mut().last_sequence = Some(sequence);
            }
            let committed = pending.restore();
            debug_assert!(committed, "lifecycle owner changed during an active pump");
        }
        report.set_drain_percentiles(
            p95_duration(&mut runtime_drain_samples),
            p95_duration(&mut decode_samples),
        );
        self.advance_round_robin_start(&snapshots, visited_consumer_count);
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
                has_pending: !consumer.pending.is_empty(),
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
        encoded_bytes_upper_bound: usize,
        runtime_remaining_deliveries: usize,
        runtime_oldest_pending_age_millis: u64,
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
        debug_assert!(consumer.pending.is_empty());
        consumer.last_observed_runtime_remaining_deliveries = Some(runtime_remaining_deliveries);
        consumer.last_observed_runtime_oldest_pending_age_millis =
            Some(runtime_oldest_pending_age_millis);
        consumer.runtime_backlog_observed_at = Some(Instant::now());
        if deliveries.is_empty() {
            return 0;
        }
        consumer.pending_encoded_bytes_upper_bound = encoded_bytes_upper_bound;
        consumer.pending_since = Some(Instant::now());
        consumer.pending.extend(deliveries);
        0
    }

    fn take_pending_batch(
        &self,
        snapshot: &ActiveConsumerSnapshot,
    ) -> Option<PendingDeliveryBatch> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let consumer = active.get_mut(&snapshot.consumer_id)?;
        if consumer.generation != snapshot.generation
            || consumer.subscription != snapshot.subscription
        {
            return None;
        }
        if consumer.pending.is_empty() {
            return None;
        }
        Some(PendingDeliveryBatch {
            deliveries: std::mem::take(&mut consumer.pending),
            last_sequence: consumer.last_sequence,
            encoded_bytes_upper_bound: std::mem::take(
                &mut consumer.pending_encoded_bytes_upper_bound,
            ),
            pending_since: consumer.pending_since.take(),
        })
    }

    fn restore_pending_batch(
        &self,
        snapshot: &ActiveConsumerSnapshot,
        pending: PendingDeliveryBatch,
    ) -> bool {
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
        consumer.last_sequence = pending.last_sequence;
        consumer.pending = pending.deliveries;
        if consumer.pending.is_empty() {
            consumer.pending_encoded_bytes_upper_bound = 0;
            consumer.pending_since = None;
        } else {
            consumer.pending_encoded_bytes_upper_bound = pending.encoded_bytes_upper_bound;
            consumer.pending_since = pending.pending_since;
        }
        true
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
        let pending_encoded_bytes_upper_bound = active
            .values()
            .map(|consumer| consumer.pending_encoded_bytes_upper_bound)
            .sum();
        let pending_oldest_age = active
            .values()
            .filter_map(|consumer| consumer.pending_since.map(|since| since.elapsed()))
            .max()
            .unwrap_or_default();
        let last_observed_runtime_backlog = active
            .values()
            .try_fold(
                (0_usize, 0_u64, Duration::ZERO),
                |(remaining_total, oldest_pending_age, observation_age), consumer| {
                    Some((
                        remaining_total
                            .saturating_add(consumer.last_observed_runtime_remaining_deliveries?),
                        oldest_pending_age
                            .max(consumer.last_observed_runtime_oldest_pending_age_millis?),
                        observation_age.max(consumer.runtime_backlog_observed_at?.elapsed()),
                    ))
                },
            )
            .filter(|_| !active.is_empty());
        drop(active);
        report.set_queue_pressure(
            queue_depth,
            pending_sequence_span,
            pending_encoded_bytes_upper_bound,
            pending_oldest_age,
        );
        report.set_last_observed_runtime_backlog(last_observed_runtime_backlog);
        self.store_pump_report(*report);
    }

    fn store_pump_report(&self, report: EditorRuntimeEventPumpReport) {
        *self
            .last_pump_report
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = report;
    }
}
