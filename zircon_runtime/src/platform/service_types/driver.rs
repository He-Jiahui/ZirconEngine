use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Instant;

use crate::core::framework::platform::{
    EventLoopBackgroundPolicy, EventLoopControlFlow, EventLoopHostWakeReason, EventLoopWakeRequest,
    EventLoopWakeSource, PlatformHostBackend, PlatformHostEvidence, PlatformHostFailureReason,
    PlatformHostInstanceId, PlatformHostQuiesceRequest, PlatformHostSnapshot,
    PreferenceStorageBackendKind,
};
use crate::core::framework::window::{
    DisplayTopologyGeneration, DisplayTopologyReplacement, DisplayTopologyReplacementError,
    DisplayTopologySnapshot, SurfaceLeaseRegistry, WindowCommandAccepted, WindowCommandId,
    WindowCommandReceipt, WindowEffectiveSnapshot, WindowId, WindowObservedState,
    WindowRequestedState,
};
use crate::core::runtime::{BoundedKeyedIoShutdownReport, TaskPool};

use super::super::application_lifecycle::ApplicationLifecycleService;
use super::super::event_loop_scheduler::{
    EventLoopDueSources, EventLoopScheduler, EventLoopSchedulerSnapshot,
};
use super::super::host::{PlatformHostService, PlatformHostServiceError};
use super::super::preferences::{
    PreferencePersistenceAdapter, PreferenceStorageBackend, UnavailablePreferenceStorageBackend,
};
use super::super::{
    allocate_window_registry_id, HostCommandAdmissionError, HostCommandBroker,
    HostCommandBrokerAccessError, HostCommandBrokerError, HostCommandDispatch,
    HostWindowCommandCompletion, PlatformWindowCommandError, WindowCommandFailure, WindowRegistry,
    WindowRegistryError, WindowStateRegistry, WindowStateRegistryError,
};

mod surface_lifecycle;

/// Same-domain driver slot that lets the process host install one platform backend.
pub struct PlatformDriver {
    preference_storage: Arc<PreferencePersistenceAdapter>,
    install_state: Mutex<PreferenceStorageBackendInstallState>,
    application_lifecycle: ApplicationLifecycleService,
    platform_host: PlatformHostService,
    window_registry: Mutex<WindowRegistryState>,
    window_states: Mutex<WindowStateRegistry>,
    host_command_broker: Mutex<Option<HostCommandBroker>>,
    event_loop_scheduler: Mutex<EventLoopScheduler>,
    surface_lifecycle_gate: Mutex<()>,
    surface_leases: Mutex<SurfaceLeaseRegistry>,
    display_topology: RwLock<Arc<DisplayTopologySnapshot>>,
}

impl PlatformDriver {
    pub(crate) fn with_io_task_pool(preference_io_pool: TaskPool) -> Self {
        Self::from_preference_storage(PreferencePersistenceAdapter::with_default_limits_on_pool(
            Arc::new(UnavailablePreferenceStorageBackend),
            preference_io_pool,
        ))
    }

    fn from_preference_storage(preference_storage: PreferencePersistenceAdapter) -> Self {
        Self {
            preference_storage: Arc::new(preference_storage),
            install_state: Mutex::new(PreferenceStorageBackendInstallState::default()),
            application_lifecycle: ApplicationLifecycleService::default(),
            platform_host: PlatformHostService::default(),
            window_registry: Mutex::new(match allocate_window_registry_id() {
                Some(registry_id) => {
                    WindowRegistryState::Available(WindowRegistry::new(registry_id))
                }
                None => WindowRegistryState::IdentityExhausted,
            }),
            window_states: Mutex::new(WindowStateRegistry::default()),
            host_command_broker: Mutex::new(None),
            event_loop_scheduler: Mutex::new(EventLoopScheduler::default()),
            surface_lifecycle_gate: Mutex::new(()),
            surface_leases: Mutex::new(SurfaceLeaseRegistry::default()),
            display_topology: RwLock::new(Arc::new(DisplayTopologySnapshot::empty(
                DisplayTopologyGeneration::initial(),
            ))),
        }
    }

    pub fn with_preference_storage_backend(
        preference_io_pool: TaskPool,
        backend: Arc<dyn PreferenceStorageBackend>,
    ) -> Result<Self, PreferenceStorageBackendInstallError> {
        let driver = Self::with_io_task_pool(preference_io_pool);
        driver.install_preference_storage_backend(backend)?;
        Ok(driver)
    }

    pub fn install_preference_storage_backend(
        &self,
        backend: Arc<dyn PreferenceStorageBackend>,
    ) -> Result<(), PreferenceStorageBackendInstallError> {
        let requested = backend.backend_kind();
        if requested == PreferenceStorageBackendKind::Unavailable {
            let current = self.preference_storage.backend_kind();
            return Err(PreferenceStorageBackendInstallError::new(
                PreferenceStorageBackendInstallErrorKind::UnavailableBackend,
                current,
                requested,
            ));
        }
        let mut state = self.lock_install_state();
        if state.installed {
            drop(state);
            let current = self.preference_storage.backend_kind();
            return Err(PreferenceStorageBackendInstallError::new(
                PreferenceStorageBackendInstallErrorKind::AlreadyInstalled,
                current,
                requested,
            ));
        }
        self.preference_storage.replace_backend(backend);
        state.installed = true;
        Ok(())
    }

    pub fn preference_storage_backend_kind(&self) -> PreferenceStorageBackendKind {
        self.preference_storage.backend_kind()
    }

    /// Returns the last host fact published by the app-owned platform thread.
    /// The snapshot never grants access to native event-loop or window objects.
    pub fn platform_host_snapshot(&self) -> PlatformHostSnapshot {
        self.platform_host.snapshot()
    }

    /// Installs the one process-host bridge for this driver. The bridge must
    /// dispatch onto its declared thread affinity instead of owning native
    /// objects in this shared driver.
    pub fn install_platform_host(
        &self,
        backend: Arc<dyn PlatformHostBackend>,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        self.platform_host.install(backend)
    }

    pub fn publish_platform_host_ready(
        &self,
        instance: PlatformHostInstanceId,
        evidence: PlatformHostEvidence,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        self.platform_host.publish_ready(instance, evidence)
    }

    pub fn publish_platform_host_degraded(
        &self,
        instance: PlatformHostInstanceId,
        evidence: PlatformHostEvidence,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        self.platform_host.publish_degraded(instance, evidence)
    }

    pub fn request_platform_host_quiesce(
        &self,
        deadline: Instant,
    ) -> Result<PlatformHostQuiesceRequest, PlatformHostServiceError> {
        self.platform_host.request_quiesce(deadline)
    }

    pub fn publish_platform_host_quiesced(
        &self,
        request: PlatformHostQuiesceRequest,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        self.platform_host.publish_quiesced(request)
    }

    pub fn publish_platform_host_failed(
        &self,
        instance: PlatformHostInstanceId,
        reason: PlatformHostFailureReason,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        self.platform_host.publish_failed(instance, reason)
    }

    pub fn publish_platform_host_stopped(
        &self,
        instance: PlatformHostInstanceId,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        self.platform_host.publish_stopped(instance)
    }

    /// Registers the latest deadline for one bounded platform wake source.
    /// Source owners coalesce payload independently, while the driver owns
    /// only the deadline selector that maps their requests to host control flow.
    pub(crate) fn schedule_event_loop_wake(&self, request: EventLoopWakeRequest) {
        self.lock_event_loop_scheduler().schedule(request);
    }

    /// Updates a source-owned backlog observation without giving the scheduler
    /// ownership of source payload queues or native event-loop objects.
    pub(crate) fn observe_event_loop_backlog(&self, source: EventLoopWakeSource, backlog: usize) {
        self.lock_event_loop_scheduler()
            .observe_backlog(source, backlog);
    }

    /// Retains the current host-owned background posture beside its scheduled
    /// deadline, without giving the driver control of App policy selection.
    pub(crate) fn observe_event_loop_background_policy(&self, policy: EventLoopBackgroundPolicy) {
        self.lock_event_loop_scheduler()
            .observe_background_policy(policy);
    }

    /// Records the native host's wake cause after its adapter has converted it
    /// to neutral runtime vocabulary.
    pub(crate) fn observe_event_loop_host_wake(
        &self,
        reason: EventLoopHostWakeReason,
        observed_at: Instant,
    ) {
        self.lock_event_loop_scheduler()
            .observe_host_wake(reason, observed_at);
    }

    /// Returns the platform-loop control-flow directive derived from current,
    /// coalesced monotonic deadlines.
    pub(crate) fn event_loop_control_flow(&self, now: Instant) -> EventLoopControlFlow {
        self.lock_event_loop_scheduler().control_flow(now)
    }

    /// Drains all sources due for this host-loop pass as one bitset. Returning
    /// the complete due set prevents a ready source from starving another.
    pub(crate) fn take_due_event_loop_wakes(&self, now: Instant) -> EventLoopDueSources {
        self.lock_event_loop_scheduler().take_due(now)
    }

    pub(crate) fn event_loop_scheduler_snapshot(&self) -> EventLoopSchedulerSnapshot {
        self.lock_event_loop_scheduler().snapshot()
    }

    pub(crate) fn preference_persistence_adapter(&self) -> Arc<PreferencePersistenceAdapter> {
        Arc::clone(&self.preference_storage)
    }

    #[cfg(test)]
    pub(crate) fn preference_persistence_uses_task_pool(&self, pool: &TaskPool) -> bool {
        self.preference_storage.shares_execution_owner_with(pool)
    }

    pub(crate) fn shutdown_preference_persistence_until(
        &self,
        deadline: Instant,
    ) -> Result<BoundedKeyedIoShutdownReport, BoundedKeyedIoShutdownReport> {
        self.preference_storage.shutdown_until(deadline)
    }

    /// The platform driver remains the only owner of the live window identity
    /// table. Host commands acquire this narrow guard instead of retaining
    /// native windows or registry internals outside the driver lifecycle.
    pub(crate) fn with_window_registry<T>(
        &self,
        operation: impl FnOnce(&mut WindowRegistry) -> Result<T, WindowRegistryError>,
    ) -> Result<T, WindowRegistryError> {
        let mut state = self
            .window_registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let WindowRegistryState::Available(registry) = &mut *state else {
            return Err(WindowRegistryError::RegistryIdentityExhausted);
        };
        operation(registry)
    }

    /// State snapshots belong to the same driver lifetime as native window
    /// identities but never expose native objects. Window command transactions
    /// acquire the window registry, then this state registry, then the command
    /// broker, and finally surface leases in that fixed order.
    pub(crate) fn with_window_states<T>(
        &self,
        operation: impl FnOnce(&mut WindowStateRegistry) -> Result<T, WindowStateRegistryError>,
    ) -> Result<T, WindowStateRegistryError> {
        let mut window_states = self
            .window_states
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        operation(&mut window_states)
    }

    /// Installs the bounded command authority only after an actual platform
    /// host selects its runtime admission budget. An uninstalled host cannot
    /// admit commands through a hidden default queue.
    pub(crate) fn install_host_command_broker(
        &self,
        maximum_outstanding: NonZeroUsize,
    ) -> Result<(), HostCommandBrokerAccessError> {
        let mut broker = self
            .host_command_broker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if broker.is_some() {
            return Err(HostCommandBrokerAccessError::AlreadyInstalled);
        }
        *broker = Some(HostCommandBroker::new(maximum_outstanding));
        Ok(())
    }

    /// Exposes the raw broker only to module tests. Production code uses the
    /// ordered transaction entry points below, so it cannot invert the
    /// registry/state/broker lock order through a callback.
    #[cfg(test)]
    pub(crate) fn with_host_command_broker<T>(
        &self,
        operation: impl FnOnce(&mut HostCommandBroker) -> Result<T, HostCommandBrokerError>,
    ) -> Result<T, HostCommandBrokerAccessError> {
        let mut broker = self
            .host_command_broker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let broker = broker
            .as_mut()
            .ok_or(HostCommandBrokerAccessError::Uninstalled)?;
        operation(broker).map_err(HostCommandBrokerAccessError::from)
    }

    /// Admits a desired window state through the only transaction allowed to
    /// change requested state: live native identity validation, current state
    /// snapshot, bounded broker reservation, then requested-state publication
    /// and queue insertion under one driver-owned lock order.
    pub(crate) fn submit_window_command(
        &self,
        target: WindowId,
        desired: WindowRequestedState,
        deadline: Instant,
        accepted_at: Instant,
    ) -> Result<WindowCommandAccepted, PlatformWindowCommandError> {
        let mut registry_state = self
            .window_registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let WindowRegistryState::Available(registry) = &mut *registry_state else {
            return Err(PlatformWindowCommandError::Registry(
                WindowRegistryError::RegistryIdentityExhausted,
            ));
        };
        registry
            .native_for(target)
            .map_err(PlatformWindowCommandError::Registry)?;

        let mut states = self
            .window_states
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let current = states
            .snapshot(target)
            .map_err(PlatformWindowCommandError::State)?;
        let expected_requested_generation = current.requested().generation();

        let mut broker_state = self
            .host_command_broker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let broker = broker_state
            .as_mut()
            .ok_or(PlatformWindowCommandError::Broker(
                HostCommandBrokerAccessError::Uninstalled,
            ))?;
        let command_desired = desired.clone();
        match broker.submit_after_requested_state(
            target,
            deadline,
            command_desired,
            &current,
            accepted_at,
            || {
                states
                    .replace_requested(target, expected_requested_generation, desired)
                    .map(|_| ())
            },
        ) {
            Ok(accepted) => Ok(accepted),
            Err(HostCommandAdmissionError::Broker(error)) => Err(
                PlatformWindowCommandError::Broker(HostCommandBrokerAccessError::from(error)),
            ),
            Err(HostCommandAdmissionError::RequestedState(error)) => {
                Err(PlatformWindowCommandError::State(error))
            }
        }
    }

    /// Acquires the next command for the platform host thread. The driver
    /// validates the live native identity and state snapshot immediately
    /// before dispatch, so a queued command cannot cross a recycled slot.
    pub(crate) fn dispatch_next_window_command(
        &self,
        now: Instant,
    ) -> Result<Option<HostCommandDispatch>, PlatformWindowCommandError> {
        let mut registry_state = self
            .window_registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let WindowRegistryState::Available(registry) = &mut *registry_state else {
            return Err(PlatformWindowCommandError::Registry(
                WindowRegistryError::RegistryIdentityExhausted,
            ));
        };
        let mut states = self
            .window_states
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut broker_state = self
            .host_command_broker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let broker = broker_state
            .as_mut()
            .ok_or(PlatformWindowCommandError::Broker(
                HostCommandBrokerAccessError::Uninstalled,
            ))?;
        let Some(target) = broker.next_target() else {
            return Ok(None);
        };
        registry
            .native_for(target)
            .map_err(PlatformWindowCommandError::Registry)?;
        let current = states
            .snapshot(target)
            .map_err(PlatformWindowCommandError::State)?;
        broker.dispatch_next(now, &current).map_err(|error| {
            PlatformWindowCommandError::Broker(HostCommandBrokerAccessError::from(error))
        })
    }

    /// Receives one platform-thread terminal result. Observed and effective
    /// state remain host facts: an applied completion records its source
    /// requested generation even when a newer desired state is pending. The
    /// broker's serial lane and state registry prevent source regression.
    pub(crate) fn complete_window_command(
        &self,
        target: WindowId,
        request_id: WindowCommandId,
        completion: HostWindowCommandCompletion,
    ) -> Result<
        WindowCommandReceipt<WindowEffectiveSnapshot, WindowCommandFailure>,
        PlatformWindowCommandError,
    > {
        let mut registry_state = self
            .window_registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let WindowRegistryState::Available(registry) = &mut *registry_state else {
            return Err(PlatformWindowCommandError::Registry(
                WindowRegistryError::RegistryIdentityExhausted,
            ));
        };
        registry
            .native_for(target)
            .map_err(PlatformWindowCommandError::Registry)?;

        let mut states = self
            .window_states
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut broker_state = self
            .host_command_broker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let broker = broker_state
            .as_mut()
            .ok_or(PlatformWindowCommandError::Broker(
                HostCommandBrokerAccessError::Uninstalled,
            ))?;
        let execution = broker.in_flight_execution(request_id).map_err(|error| {
            PlatformWindowCommandError::Broker(HostCommandBrokerAccessError::from(error))
        })?;
        if execution.target() != target {
            return Err(PlatformWindowCommandError::Broker(
                HostCommandBrokerAccessError::Broker(
                    HostCommandBrokerError::SnapshotTargetMismatch {
                        expected: execution.target(),
                        actual: target,
                    },
                ),
            ));
        }

        states
            .preflight_command_completion(
                target,
                completion
                    .applies_effective_state()
                    .then_some(execution.requested_generation()),
            )
            .map_err(PlatformWindowCommandError::State)?;
        let (observed, effective, terminal) = completion.into_parts();
        let current = states
            .publish_observed(target, observed)
            .map_err(PlatformWindowCommandError::State)?;
        let current = match effective {
            Some(effective) => states
                .publish_effective(target, execution.requested_generation(), effective)
                .map_err(PlatformWindowCommandError::State)?,
            None => current,
        };
        broker
            .complete(request_id, &current, terminal)
            .map_err(|error| {
                PlatformWindowCommandError::Broker(HostCommandBrokerAccessError::from(error))
            })
    }

    /// Removes one terminal receipt after its consumer has persisted or
    /// forwarded it. This is also the explicit backpressure release for the
    /// broker's bounded outstanding-command budget.
    pub(crate) fn take_window_command_receipt(
        &self,
        request_id: WindowCommandId,
    ) -> Result<
        Option<WindowCommandReceipt<WindowEffectiveSnapshot, WindowCommandFailure>>,
        HostCommandBrokerAccessError,
    > {
        let mut broker_state = self
            .host_command_broker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let broker = broker_state
            .as_mut()
            .ok_or(HostCommandBrokerAccessError::Uninstalled)?;
        Ok(broker.take_terminal_receipt(request_id))
    }

    /// Returns the last atomically published display topology without exposing
    /// backend monitor objects beyond the platform-host owner.
    pub(crate) fn display_topology_snapshot(&self) -> Arc<DisplayTopologySnapshot> {
        Arc::clone(&self.read_display_topology())
    }

    /// Publishes a fully validated replacement topology. The generation and
    /// diff are checked before the shared snapshot pointer is changed.
    pub(crate) fn publish_display_topology(
        &self,
        topology: DisplayTopologySnapshot,
    ) -> Result<DisplayTopologyReplacement, DisplayTopologyReplacementError> {
        let mut current = self.write_display_topology();
        let replacement = topology.replacement_from(current.as_ref())?;
        *current = Arc::new(topology);
        Ok(replacement)
    }

    fn lock_install_state(&self) -> MutexGuard<'_, PreferenceStorageBackendInstallState> {
        self.install_state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_event_loop_scheduler(&self) -> MutexGuard<'_, EventLoopScheduler> {
        self.event_loop_scheduler
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn read_display_topology(&self) -> RwLockReadGuard<'_, Arc<DisplayTopologySnapshot>> {
        self.display_topology
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write_display_topology(&self) -> RwLockWriteGuard<'_, Arc<DisplayTopologySnapshot>> {
        self.display_topology
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl fmt::Debug for PlatformDriver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PlatformDriver")
            .field(
                "preference_storage_backend_kind",
                &self.preference_storage_backend_kind(),
            )
            .field(
                "application_lifecycle",
                &self.application_lifecycle_snapshot(),
            )
            .field("platform_host", &self.platform_host_snapshot())
            .finish()
    }
}

#[derive(Default)]
struct PreferenceStorageBackendInstallState {
    installed: bool,
}

enum WindowRegistryState {
    Available(WindowRegistry),
    IdentityExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferenceStorageBackendInstallErrorKind {
    UnavailableBackend,
    AlreadyInstalled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreferenceStorageBackendInstallError {
    kind: PreferenceStorageBackendInstallErrorKind,
    current_backend: PreferenceStorageBackendKind,
    requested_backend: PreferenceStorageBackendKind,
}

impl PreferenceStorageBackendInstallError {
    const fn new(
        kind: PreferenceStorageBackendInstallErrorKind,
        current_backend: PreferenceStorageBackendKind,
        requested_backend: PreferenceStorageBackendKind,
    ) -> Self {
        Self {
            kind,
            current_backend,
            requested_backend,
        }
    }

    pub const fn kind(self) -> PreferenceStorageBackendInstallErrorKind {
        self.kind
    }

    pub const fn current_backend(self) -> PreferenceStorageBackendKind {
        self.current_backend
    }

    pub const fn requested_backend(self) -> PreferenceStorageBackendKind {
        self.requested_backend
    }
}

impl fmt::Display for PreferenceStorageBackendInstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cannot install preference storage backend {} over {}: {:?}",
            self.requested_backend.as_str(),
            self.current_backend.as_str(),
            self.kind
        )
    }
}

impl Error for PreferenceStorageBackendInstallError {}

#[cfg(test)]
mod tests;
