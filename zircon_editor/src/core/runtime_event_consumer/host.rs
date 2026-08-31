use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use zircon_runtime_interface::{
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle,
};

use crate::core::gateway::{EditorRuntimeGatewayHandle, GatewayOrigin, GatewaySessionIdentity};

use super::{
    EditorRuntimeEventBacklogObservation, EditorRuntimeEventConsumerCallbackPhase,
    EditorRuntimeEventConsumerDeliveryDisposition, EditorRuntimeEventConsumerError,
    EditorRuntimeEventConsumerFaultReceipt, EditorRuntimeEventConsumerFaultReceiptBudget,
    EditorRuntimeEventConsumerFaultReceiptJournal, EditorRuntimeEventConsumerRegistration,
    EditorRuntimeEventConsumerRegistry, EditorRuntimeEventPumpBudget, EditorRuntimeEventPumpReport,
};

mod contribution_lifecycle;
mod execution_support;
mod health;
mod lifecycle;
mod pending;
mod pump_execution;
mod retention;
mod round_robin;

pub(crate) use contribution_lifecycle::ContributionRetirementReport;
use execution_support::{invoke_consumer_callback, unsubscribe_consumer, LifecycleExecutionGuard};
use health::{
    ConsumerCallbackHealth, EditorRuntimeEventConsumerFaultPolicy,
    EditorRuntimeEventConsumerQuarantineReason,
};
use lifecycle::{combine_cleanup_errors, PendingRemoteCleanup};
use pending::{EditorRuntimeEventConsumerPendingDeliveryBudget, PendingDelivery};
use retention::EditorRuntimeEventConsumerRetentionBudget;

const EXECUTION_IDLE: u8 = 0;
const EXECUTION_PUMP: u8 = 1;
const EXECUTION_LIFECYCLE: u8 = 2;

/// Opaque plugin-event subscription qualified by the gateway transport that issued it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct QualifiedSubscription {
    raw: ZrRuntimePluginEventSubscriptionHandle,
    identity: GatewaySessionIdentity,
}

impl QualifiedSubscription {
    fn new(raw: ZrRuntimePluginEventSubscriptionHandle, identity: GatewaySessionIdentity) -> Self {
        Self { raw, identity }
    }

    fn raw(&self) -> ZrRuntimePluginEventSubscriptionHandle {
        self.raw
    }

    fn belongs_to(&self, origin: &GatewayOrigin) -> bool {
        &self.identity == origin.identity()
    }
}

struct ActiveConsumer {
    registration: EditorRuntimeEventConsumerRegistration,
    origin: GatewayOrigin,
    health: ConsumerCallbackHealth,
    subscription: QualifiedSubscription,
    generation: u64,
    last_sequence: Option<u64>,
    pending: VecDeque<PendingDelivery>,
    pending_retained_bytes: usize,
    last_observed_runtime_remaining_deliveries: Option<usize>,
    last_observed_runtime_oldest_pending_age_millis: Option<u64>,
    runtime_backlog_observed_at: Option<Instant>,
}

#[derive(Clone)]
struct ActiveConsumerSnapshot {
    consumer_id: String,
    registration: EditorRuntimeEventConsumerRegistration,
    origin: GatewayOrigin,
    subscription: QualifiedSubscription,
    generation: u64,
    has_pending: bool,
}

#[derive(Clone)]
struct ActiveConsumerIdentity {
    consumer_id: String,
    subscription: QualifiedSubscription,
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
    fault_receipts: Mutex<EditorRuntimeEventConsumerFaultReceiptJournal>,
    pending_delivery_budget: EditorRuntimeEventConsumerPendingDeliveryBudget,
    retention_budget: EditorRuntimeEventConsumerRetentionBudget,
    fault_policy: EditorRuntimeEventConsumerFaultPolicy,
    pending_retained_bytes: AtomicUsize,
    retained_bytes: AtomicUsize,
    quarantined_consumers: Mutex<BTreeMap<String, EditorRuntimeEventConsumerQuarantineReason>>,
    user_disabled_consumers: Mutex<BTreeSet<String>>,
    pending_remote_cleanup: Mutex<BTreeMap<String, PendingRemoteCleanup>>,
    execution_state: AtomicU8,
}

impl Default for EditorRuntimeEventConsumerHost {
    fn default() -> Self {
        Self::new(EditorRuntimeGatewayHandle::detached())
    }
}

impl EditorRuntimeEventConsumerHost {
    pub fn new(gateway: EditorRuntimeGatewayHandle) -> Self {
        Self::with_budgets(
            gateway,
            EditorRuntimeEventConsumerFaultReceiptBudget::default(),
            EditorRuntimeEventConsumerPendingDeliveryBudget::default(),
        )
    }

    pub fn with_fault_receipt_budget(
        gateway: EditorRuntimeGatewayHandle,
        fault_receipt_budget: EditorRuntimeEventConsumerFaultReceiptBudget,
    ) -> Self {
        Self::with_budgets(
            gateway,
            fault_receipt_budget,
            EditorRuntimeEventConsumerPendingDeliveryBudget::default(),
        )
    }

    pub fn with_pending_delivery_budget(
        gateway: EditorRuntimeGatewayHandle,
        pending_delivery_budget: EditorRuntimeEventConsumerPendingDeliveryBudget,
    ) -> Self {
        Self::with_budgets(
            gateway,
            EditorRuntimeEventConsumerFaultReceiptBudget::default(),
            pending_delivery_budget,
        )
    }

    pub fn with_fault_policy(
        gateway: EditorRuntimeGatewayHandle,
        fault_policy: EditorRuntimeEventConsumerFaultPolicy,
    ) -> Self {
        Self::with_budgets_and_fault_policy(
            gateway,
            EditorRuntimeEventConsumerFaultReceiptBudget::default(),
            EditorRuntimeEventConsumerPendingDeliveryBudget::default(),
            fault_policy,
        )
    }

    pub fn with_retention_budget(
        gateway: EditorRuntimeGatewayHandle,
        retention_budget: EditorRuntimeEventConsumerRetentionBudget,
    ) -> Self {
        Self::with_budgets_and_retention(
            gateway,
            EditorRuntimeEventConsumerFaultReceiptBudget::default(),
            EditorRuntimeEventConsumerPendingDeliveryBudget::default(),
            retention_budget,
        )
    }

    pub fn with_budgets(
        gateway: EditorRuntimeGatewayHandle,
        fault_receipt_budget: EditorRuntimeEventConsumerFaultReceiptBudget,
        pending_delivery_budget: EditorRuntimeEventConsumerPendingDeliveryBudget,
    ) -> Self {
        Self::with_budgets_and_fault_policy(
            gateway,
            fault_receipt_budget,
            pending_delivery_budget,
            EditorRuntimeEventConsumerFaultPolicy::default(),
        )
    }

    pub fn with_budgets_and_retention(
        gateway: EditorRuntimeGatewayHandle,
        fault_receipt_budget: EditorRuntimeEventConsumerFaultReceiptBudget,
        pending_delivery_budget: EditorRuntimeEventConsumerPendingDeliveryBudget,
        retention_budget: EditorRuntimeEventConsumerRetentionBudget,
    ) -> Self {
        Self::with_all_budgets_and_fault_policy(
            gateway,
            fault_receipt_budget,
            pending_delivery_budget,
            retention_budget,
            EditorRuntimeEventConsumerFaultPolicy::default(),
        )
    }

    fn with_budgets_and_fault_policy(
        gateway: EditorRuntimeGatewayHandle,
        fault_receipt_budget: EditorRuntimeEventConsumerFaultReceiptBudget,
        pending_delivery_budget: EditorRuntimeEventConsumerPendingDeliveryBudget,
        fault_policy: EditorRuntimeEventConsumerFaultPolicy,
    ) -> Self {
        Self::with_all_budgets_and_fault_policy(
            gateway,
            fault_receipt_budget,
            pending_delivery_budget,
            EditorRuntimeEventConsumerRetentionBudget::default(),
            fault_policy,
        )
    }

    fn with_all_budgets_and_fault_policy(
        gateway: EditorRuntimeGatewayHandle,
        fault_receipt_budget: EditorRuntimeEventConsumerFaultReceiptBudget,
        pending_delivery_budget: EditorRuntimeEventConsumerPendingDeliveryBudget,
        retention_budget: EditorRuntimeEventConsumerRetentionBudget,
        fault_policy: EditorRuntimeEventConsumerFaultPolicy,
    ) -> Self {
        Self {
            gateway,
            registry: Mutex::new(EditorRuntimeEventConsumerRegistry::default()),
            active: Mutex::new(BTreeMap::new()),
            play_session_id: Mutex::new(None),
            next_consumer_generation: AtomicU64::new(0),
            round_robin_cursor: Mutex::new(None),
            last_pump_report: Mutex::new(EditorRuntimeEventPumpReport::default()),
            fault_receipts: Mutex::new(EditorRuntimeEventConsumerFaultReceiptJournal::new(
                fault_receipt_budget,
            )),
            pending_delivery_budget,
            retention_budget,
            fault_policy,
            pending_retained_bytes: AtomicUsize::new(0),
            retained_bytes: AtomicUsize::new(0),
            quarantined_consumers: Mutex::new(BTreeMap::new()),
            user_disabled_consumers: Mutex::new(BTreeSet::new()),
            pending_remote_cleanup: Mutex::new(BTreeMap::new()),
            execution_state: AtomicU8::new(EXECUTION_IDLE),
        }
    }

    pub fn runtime_session_id(&self) -> u64 {
        self.gateway.session_handle().raw()
    }

    pub fn fault_receipts(&self) -> Vec<EditorRuntimeEventConsumerFaultReceipt> {
        self.fault_receipts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .snapshot()
    }

    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes.load(Ordering::Relaxed)
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

        self.quarantined_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        self.user_disabled_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();

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
        let quarantined = self
            .quarantined_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let user_disabled = self
            .user_disabled_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let desired = registrations
            .into_iter()
            .filter(|registration| {
                let required = &registration.manifest().required_capability;
                required.is_empty()
                    || enabled_capabilities
                        .iter()
                        .any(|capability| capability == required)
            })
            .filter(|registration| !quarantined.contains_key(&registration.manifest().consumer_id))
            .filter(|registration| !user_disabled.contains(&registration.manifest().consumer_id))
            .map(|registration| (registration.manifest().consumer_id.clone(), registration))
            .collect::<BTreeMap<_, _>>();
        self.retry_pending_remote_cleanup();
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
                    subscription: consumer.subscription.clone(),
                    generation: consumer.generation,
                })
                .collect::<Vec<_>>()
        };
        let mut first_error = None;
        for identity in removed {
            if let Err(error) = self.retire_active_consumer(&identity, play_session_id) {
                first_error.get_or_insert(error);
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
            let generation = match self.allocate_consumer_generation() {
                Ok(generation) => generation,
                Err(error) => {
                    return Err(
                        match self.rollback_added_consumers(&added, play_session_id) {
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
            let origin = self.gateway.current_lease().origin();
            let subscription = match origin
                .gateway()
                .subscribe_plugin_event(&manifest.event_id, &manifest.payload_schema)
            {
                Ok(Some(subscription)) => subscription,
                Ok(None) => {
                    let error = EditorRuntimeEventConsumerError::Unsupported {
                        consumer_id: manifest.consumer_id.clone(),
                    };
                    return Err(
                        match self.rollback_added_consumers(&added, play_session_id) {
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
                        match self.rollback_added_consumers(&added, play_session_id) {
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
            let subscription = QualifiedSubscription::new(subscription, origin.identity().clone());
            if let Err(_error) = invoke_consumer_callback(
                &manifest.consumer_id,
                EditorRuntimeEventConsumerCallbackPhase::BeginSession,
                None,
                || registration.begin_session(play_session_id),
            ) {
                let cleanup_error =
                    unsubscribe_consumer(&origin, &manifest.consumer_id, &subscription).err();
                if cleanup_error.is_some() {
                    self.defer_remote_cleanup(&manifest.consumer_id, subscription.clone(), origin);
                }
                self.record_callback_fault(
                    &manifest.consumer_id,
                    play_session_id,
                    EditorRuntimeEventConsumerCallbackPhase::BeginSession,
                    None,
                    cleanup_error.as_ref(),
                );
                self.quarantine_consumer(
                    &manifest.consumer_id,
                    EditorRuntimeEventConsumerQuarantineReason::CallbackPanicked,
                );
                continue;
            }
            let identity = ActiveConsumerIdentity {
                consumer_id: manifest.consumer_id,
                subscription: subscription.clone(),
                generation,
            };
            self.active
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert(
                    identity.consumer_id.clone(),
                    ActiveConsumer {
                        registration,
                        origin,
                        health: ConsumerCallbackHealth::default(),
                        subscription,
                        generation: identity.generation,
                        last_sequence: None,
                        pending: VecDeque::new(),
                        pending_retained_bytes: 0,
                        last_observed_runtime_remaining_deliveries: None,
                        last_observed_runtime_oldest_pending_age_millis: None,
                        runtime_backlog_observed_at: None,
                    },
                );
            added.push(identity);
        }
        Ok(())
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

    /// Makes one terminal cleanup pass even when subscription rollback left no active play
    /// session. Local active entries are retired first; deferred origin cleanup is then attempted
    /// once and its error remains observable to the host coordinator.
    pub(crate) fn shutdown(&self) -> Result<(), EditorRuntimeEventConsumerError> {
        let _lifecycle_guard = LifecycleExecutionGuard::enter(
            &self.execution_state,
            "shutdown runtime event consumers",
        )?;
        let active_play_session_id = *self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match active_play_session_id {
            Some(play_session_id) => self.end_play_session_inner(play_session_id),
            None => self.flush_pending_remote_cleanup().map_or(Ok(()), Err),
        }
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
        let consumers = {
            let active = self
                .active
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            active
                .iter()
                .map(|(consumer_id, consumer)| ActiveConsumerIdentity {
                    consumer_id: consumer_id.clone(),
                    subscription: consumer.subscription.clone(),
                    generation: consumer.generation,
                })
                .collect::<Vec<_>>()
        };
        let mut first_error = None;
        for identity in consumers {
            if let Err(error) = self.retire_active_consumer(&identity, play_session_id) {
                first_error.get_or_insert(error);
            }
        }
        if let Some(error) = self.flush_pending_remote_cleanup() {
            first_error.get_or_insert(error);
        }
        if self.active_consumer_count() == 0 {
            *self
                .play_session_id
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
            self.quarantined_consumers
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear();
            self.user_disabled_consumers
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear();
        }
        first_error.map_or(Ok(()), Err)
    }

    pub fn active_consumer_count(&self) -> usize {
        self.active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub(crate) fn pending_remote_cleanup_count(&self) -> usize {
        self.pending_remote_cleanup
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub fn quarantined_consumer_count(&self) -> usize {
        self.quarantined_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub fn quarantined_consumer_reason(
        &self,
        consumer_id: &str,
    ) -> Option<EditorRuntimeEventConsumerQuarantineReason> {
        self.quarantined_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(consumer_id)
            .copied()
    }

    /// Re-enables one quarantined consumer and recreates its origin subscription for this play
    /// session. A failed reactivation retains its previous diagnostic state.
    pub fn retry_quarantined_consumer(
        &self,
        consumer_id: &str,
        enabled_capabilities: &[String],
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let _lifecycle_guard =
            LifecycleExecutionGuard::enter(&self.execution_state, "retry quarantined consumer")?;
        let previous_reason = self
            .quarantined_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(consumer_id)
            .ok_or_else(|| EditorRuntimeEventConsumerError::ConsumerNotQuarantined {
                consumer_id: consumer_id.to_string(),
            })?;
        self.user_disabled_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(consumer_id);
        match self.reconcile_enabled_capabilities_inner(enabled_capabilities) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.quarantined_consumers
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .entry(consumer_id.to_string())
                    .or_insert(previous_reason);
                Err(error)
            }
        }
    }

    /// Stops automatic reactivation for one quarantined consumer until the current play session
    /// ends. Project-persistent enablement remains owned by Plugin Manager.
    pub fn disable_quarantined_consumer(
        &self,
        consumer_id: &str,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let _lifecycle_guard =
            LifecycleExecutionGuard::enter(&self.execution_state, "disable quarantined consumer")?;
        if !self
            .quarantined_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(consumer_id)
        {
            return Err(EditorRuntimeEventConsumerError::ConsumerNotQuarantined {
                consumer_id: consumer_id.to_string(),
            });
        }
        self.user_disabled_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(consumer_id.to_string());
        Ok(())
    }

    pub fn consumer_is_user_disabled(&self, consumer_id: &str) -> bool {
        self.user_disabled_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains(consumer_id)
    }

    fn try_reserve_pending_bytes(&self, bytes: usize) -> bool {
        if !self.try_reserve_retained_bytes(bytes) {
            return false;
        }
        if self
            .pending_retained_bytes
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |retained| {
                retained
                    .checked_add(bytes)
                    .filter(|next| *next <= self.pending_delivery_budget.max_retained_bytes())
            })
            .is_ok()
        {
            true
        } else {
            self.release_retained_bytes(bytes);
            false
        }
    }

    pub(super) fn release_pending_bytes(&self, bytes: usize) {
        let _ = self.pending_retained_bytes.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            |retained| Some(retained.saturating_sub(bytes)),
        );
        self.release_retained_bytes(bytes);
    }

    pub fn active_play_session_id(&self) -> Option<u64> {
        *self
            .play_session_id
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn allocate_consumer_generation(&self) -> Result<u64, EditorRuntimeEventConsumerError> {
        let mut current = self.next_consumer_generation.load(Ordering::Relaxed);
        loop {
            let next = next_consumer_generation(current)?;
            match self.next_consumer_generation.compare_exchange_weak(
                current,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(next),
                Err(actual) => current = actual,
            }
        }
    }

    fn quarantine_consumer(
        &self,
        consumer_id: &str,
        reason: EditorRuntimeEventConsumerQuarantineReason,
    ) {
        self.quarantined_consumers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(consumer_id.to_string(), reason);
    }

    fn record_callback_health(
        &self,
        snapshot: &ActiveConsumerSnapshot,
        failed: bool,
        callback_elapsed: Duration,
        slow_callback_threshold: Duration,
    ) -> Option<EditorRuntimeEventConsumerQuarantineReason> {
        self.active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get_mut(&snapshot.consumer_id)
            .filter(|consumer| {
                consumer.generation == snapshot.generation
                    && consumer.subscription == snapshot.subscription
            })
            .and_then(|consumer| {
                consumer.health.record(
                    self.fault_policy,
                    failed,
                    callback_elapsed,
                    slow_callback_threshold,
                )
            })
    }

    fn record_callback_fault(
        &self,
        consumer_id: &str,
        play_session_id: u64,
        phase: EditorRuntimeEventConsumerCallbackPhase,
        delivery: Option<&ZrRuntimePluginEventDeliveryV1>,
        remote_cleanup_error: Option<&EditorRuntimeEventConsumerError>,
    ) {
        self.fault_receipts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .record_callback_panic(
                consumer_id,
                play_session_id,
                phase,
                delivery,
                remote_cleanup_error.map(ToString::to_string).as_deref(),
                |bytes| self.try_reserve_retained_bytes(bytes),
                |bytes| self.release_retained_bytes(bytes),
            );
    }

    fn try_reserve_retained_bytes(&self, bytes: usize) -> bool {
        self.retained_bytes
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |retained| {
                retained
                    .checked_add(bytes)
                    .filter(|next| *next <= self.retention_budget.max_retained_bytes())
            })
            .is_ok()
    }

    fn release_retained_bytes(&self, bytes: usize) {
        let _ =
            self.retained_bytes
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |retained| {
                    Some(retained.saturating_sub(bytes))
                });
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
                origin: consumer.origin.clone(),
                subscription: consumer.subscription.clone(),
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

    fn finish_pump_report(&self, report: &mut EditorRuntimeEventPumpReport) {
        let active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let queue_depth = active.values().map(|consumer| consumer.pending.len()).sum();
        let pending_sequence_span = active
            .values()
            .filter_map(|consumer| {
                let first = consumer.pending.front()?.delivery().sequence;
                let last = consumer.pending.back()?.delivery().sequence;
                Some(last.saturating_sub(first))
            })
            .max()
            .unwrap_or_default();
        let pending_encoded_bytes_upper_bound = self.pending_retained_bytes.load(Ordering::Relaxed);
        let pending_oldest_age = active
            .values()
            .filter_map(|consumer| {
                consumer
                    .pending
                    .front()
                    .map(|delivery| delivery.first_seen().elapsed())
            })
            .max()
            .unwrap_or_default();
        let mut runtime_backlog_observation = EditorRuntimeEventBacklogObservation::default();
        for consumer in active.values() {
            match (
                consumer.last_observed_runtime_remaining_deliveries,
                consumer.last_observed_runtime_oldest_pending_age_millis,
                consumer.runtime_backlog_observed_at,
            ) {
                (
                    Some(remaining_deliveries),
                    Some(oldest_pending_age_millis),
                    Some(observed_at),
                ) => {
                    runtime_backlog_observation.record_sample(
                        remaining_deliveries,
                        oldest_pending_age_millis,
                        observed_at.elapsed(),
                    );
                }
                _ => runtime_backlog_observation.record_unknown_consumer(),
            }
        }
        drop(active);
        report.set_queue_pressure(
            queue_depth,
            pending_sequence_span,
            pending_encoded_bytes_upper_bound,
            pending_oldest_age,
        );
        report.set_runtime_backlog_observation(runtime_backlog_observation);
        self.store_pump_report(*report);
    }

    fn store_pump_report(&self, report: EditorRuntimeEventPumpReport) {
        *self
            .last_pump_report
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = report;
    }
}

fn next_consumer_generation(current: u64) -> Result<u64, EditorRuntimeEventConsumerError> {
    current
        .checked_add(1)
        .ok_or(EditorRuntimeEventConsumerError::ConsumerGenerationExhausted)
}

#[cfg(test)]
mod tests {
    use super::{next_consumer_generation, EditorRuntimeEventConsumerHost, QualifiedSubscription};
    use crate::core::gateway::EditorRuntimeGatewayHandle;
    use crate::core::runtime_event_consumer::EditorRuntimeEventConsumerError;
    use zircon_runtime_interface::ZrRuntimePluginEventSubscriptionHandle;

    #[test]
    fn consumer_generation_exhaustion_is_typed() {
        assert!(matches!(
            next_consumer_generation(u64::MAX),
            Err(EditorRuntimeEventConsumerError::ConsumerGenerationExhausted)
        ));
    }

    #[test]
    fn shutdown_flushes_pending_remote_cleanup_without_an_active_play_session() {
        let gateway = EditorRuntimeGatewayHandle::detached();
        let host = EditorRuntimeEventConsumerHost::new(gateway.clone());
        let origin = gateway.current_lease().origin();
        host.defer_remote_cleanup(
            "deferred.consumer",
            QualifiedSubscription::new(
                ZrRuntimePluginEventSubscriptionHandle::new(11),
                origin.identity().clone(),
            ),
            origin,
        );

        assert_eq!(host.pending_remote_cleanup_count(), 1);
        assert!(host.shutdown().is_err());
        assert_eq!(host.pending_remote_cleanup_count(), 0);
    }
}
