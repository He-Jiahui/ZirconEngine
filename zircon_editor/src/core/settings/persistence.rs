use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::core::runtime::tasks::{
    BoundedKeyedIoAdmissionError, BoundedKeyedIoCancelAuthority, BoundedKeyedIoCancelError,
    BoundedKeyedIoDiagnostics, BoundedKeyedIoFence, BoundedKeyedIoLane, BoundedKeyedIoLimits,
    BoundedKeyedIoShutdownGuard, BoundedKeyedIoShutdownReport, BoundedKeyedIoTerminal,
    BoundedKeyedIoTicket, BoundedKeyedIoWaitResult, BoundedKeyedIoWorkDeadline, JobScheduler,
};

use super::{SettingChange, SettingsAuthority, SettingsKey, SettingsScope, SettingsStore};

const SETTINGS_PERSISTENCE_FAILURE_CODE: &str = "editor_settings_persistence_write_failed";
const PERSISTENCE_ENTRY_OVERHEAD_BYTES: usize = 128;
static NEXT_SETTINGS_FILE_GENERATION: AtomicU64 = AtomicU64::new(1);

type SettingsPersistenceTerminalObserver =
    Arc<dyn Fn(BoundedKeyedIoTerminal) + Send + Sync + 'static>;

/// Process-monotonic identity for one accepted physical settings-file state.
///
/// The physical target remains part of the identity. A process-wide allocator prevents a project
/// path rebound from regressing Runtime11's generation ordering for the same lane key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SettingsFileGeneration(u64);

impl SettingsFileGeneration {
    pub(crate) const fn from_raw(generation: u64) -> Self {
        Self(generation)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Hard bounds for settings save requests retained by the shared Runtime11 lane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsPersistenceLimits {
    pub max_entries: usize,
    pub max_retained_bytes: usize,
}

impl Default for SettingsPersistenceLimits {
    fn default() -> Self {
        Self {
            max_entries: 1024,
            max_retained_bytes: 8 * 1024 * 1024,
        }
    }
}

/// Immutable persistence work derived from one authority change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsPersistenceRequest {
    key: SettingsKey,
    scope: SettingsScope,
    target: Arc<str>,
    store: SettingsStore,
    file_generation: SettingsFileGeneration,
    authority_generation: u64,
}

impl SettingsPersistenceRequest {
    fn from_change(
        change: &SettingChange,
        store: SettingsStore,
        file_generation: SettingsFileGeneration,
    ) -> Result<Self, SettingsPersistenceSubmitError> {
        let target = persistence_target(change.scope, &store)?;
        Ok(Self {
            key: change.key.clone(),
            scope: change.scope,
            target,
            store,
            file_generation,
            authority_generation: change.revision,
        })
    }

    pub fn key(&self) -> &SettingsKey {
        &self.key
    }

    pub const fn scope(&self) -> SettingsScope {
        self.scope
    }

    pub fn target(&self) -> &str {
        self.target.as_ref()
    }

    pub const fn file_generation(&self) -> SettingsFileGeneration {
        self.file_generation
    }

    pub const fn authority_generation(&self) -> u64 {
        self.authority_generation
    }

    fn retained_bytes(&self) -> usize {
        PERSISTENCE_ENTRY_OVERHEAD_BYTES.saturating_add(self.target.len())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsPersistenceSubmitError {
    NonPersistentScope(SettingsScope),
    TargetUnavailable(SettingsScope),
    FileGenerationExhausted,
    LaneAdmission(BoundedKeyedIoAdmissionError),
}

impl fmt::Display for SettingsPersistenceSubmitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonPersistentScope(scope) => {
                write!(formatter, "{scope:?} settings cannot be persisted")
            }
            Self::TargetUnavailable(scope) => {
                write!(
                    formatter,
                    "{scope:?} settings do not have a physical target"
                )
            }
            Self::FileGenerationExhausted => {
                formatter.write_str("settings file generation is exhausted")
            }
            Self::LaneAdmission(error) => {
                write!(
                    formatter,
                    "settings persistence lane rejected work: {error:?}"
                )
            }
        }
    }
}

impl std::error::Error for SettingsPersistenceSubmitError {}

/// Rejects a retry that does not originate from a completed write failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsPersistenceRetryError {
    SourceTicketNotFailed {
        terminal: Option<BoundedKeyedIoTerminal>,
    },
    LaneAdmission(BoundedKeyedIoAdmissionError),
}

impl fmt::Display for SettingsPersistenceRetryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceTicketNotFailed { terminal } => {
                write!(
                    formatter,
                    "settings persistence ticket is not a failed write: {terminal:?}"
                )
            }
            Self::LaneAdmission(error) => {
                write!(
                    formatter,
                    "settings persistence retry was rejected by the lane: {error:?}"
                )
            }
        }
    }
}

impl std::error::Error for SettingsPersistenceRetryError {}

/// Terminal failure for the fence that closes settings persistence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsPersistenceShutdownError {
    FenceNotTerminal,
    FenceTerminal(BoundedKeyedIoTerminal),
}

impl fmt::Display for SettingsPersistenceShutdownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FenceNotTerminal => {
                write!(
                    formatter,
                    "settings persistence shutdown fence was not terminal"
                )
            }
            Self::FenceTerminal(terminal) => {
                write!(
                    formatter,
                    "settings persistence shutdown fence did not succeed: {terminal:?}"
                )
            }
        }
    }
}

impl std::error::Error for SettingsPersistenceShutdownError {}

/// A typed save ticket. Its terminal state, cancellation, and fences are owned by Runtime11.
#[derive(Clone, Debug)]
pub struct SettingsPersistenceTicket {
    request: SettingsPersistenceRequest,
    ticket: BoundedKeyedIoTicket,
    cancel_authority: BoundedKeyedIoCancelAuthority,
}

impl SettingsPersistenceTicket {
    pub fn key(&self) -> &SettingsKey {
        self.request.key()
    }

    pub const fn scope(&self) -> SettingsScope {
        self.request.scope()
    }

    pub fn target(&self) -> &str {
        self.request.target()
    }

    pub const fn file_generation(&self) -> SettingsFileGeneration {
        self.request.file_generation()
    }

    pub const fn authority_generation(&self) -> u64 {
        self.request.authority_generation()
    }

    pub fn terminal(&self) -> Option<BoundedKeyedIoTerminal> {
        self.ticket.terminal()
    }

    pub fn wait_until(&self, deadline: Instant) -> BoundedKeyedIoWaitResult {
        self.ticket.wait_until(deadline)
    }

    pub fn cancel_before_start(&self) -> Result<(), BoundedKeyedIoCancelError> {
        self.ticket.cancel_before_start(&self.cancel_authority)
    }
}

/// Owns the close fence and Runtime11 shutdown guard until their terminal result is observed.
pub struct SettingsPersistenceShutdown {
    fence: BoundedKeyedIoFence,
    guard: BoundedKeyedIoShutdownGuard,
}

impl SettingsPersistenceShutdown {
    /// Waits for the fence and returns only a successful persistence shutdown.
    pub fn finish(self) -> Result<BoundedKeyedIoShutdownReport, SettingsPersistenceShutdownError> {
        self.guard.wait();
        let report = self.guard.report();
        match self.fence.ticket().terminal() {
            Some(BoundedKeyedIoTerminal::Succeeded) => Ok(report),
            Some(terminal) => Err(SettingsPersistenceShutdownError::FenceTerminal(terminal)),
            None => Err(SettingsPersistenceShutdownError::FenceNotTerminal),
        }
    }
}

/// Bridges typed authority changes to Runtime11's bounded keyed I/O lane.
#[derive(Clone)]
pub struct SettingsPersistenceService {
    authority: Arc<SettingsAuthority>,
    lane: BoundedKeyedIoLane,
}

impl SettingsPersistenceService {
    pub fn new(authority: Arc<SettingsAuthority>, scheduler: JobScheduler) -> Self {
        Self::with_limits(authority, scheduler, SettingsPersistenceLimits::default())
    }

    pub fn with_limits(
        authority: Arc<SettingsAuthority>,
        scheduler: JobScheduler,
        limits: SettingsPersistenceLimits,
    ) -> Self {
        Self {
            authority,
            lane: BoundedKeyedIoLane::new(
                BoundedKeyedIoLimits::new(limits.max_entries, limits.max_retained_bytes),
                scheduler,
            ),
        }
    }

    pub(super) fn allocate_file_generation(
        &self,
    ) -> Result<SettingsFileGeneration, SettingsPersistenceSubmitError> {
        NEXT_SETTINGS_FILE_GENERATION
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |generation| {
                generation.checked_add(1)
            })
            .map(SettingsFileGeneration)
            .map_err(|_| SettingsPersistenceSubmitError::FileGenerationExhausted)
    }

    /// Submits a changed persistent key. The caller performs no filesystem work.
    pub fn submit(
        &self,
        change: &SettingChange,
        store: SettingsStore,
    ) -> Result<SettingsPersistenceTicket, SettingsPersistenceSubmitError> {
        if !change.scope.is_persistent() {
            return Err(SettingsPersistenceSubmitError::NonPersistentScope(
                change.scope,
            ));
        }
        let generation = self.allocate_file_generation()?;
        let request = SettingsPersistenceRequest::from_change(change, store, generation)?;
        self.submit_request(request, None)
    }

    pub(super) fn submit_observed(
        &self,
        change: &SettingChange,
        file_generation: SettingsFileGeneration,
        store: SettingsStore,
        observer: impl Fn(BoundedKeyedIoTerminal) + Send + Sync + 'static,
    ) -> Result<SettingsPersistenceTicket, SettingsPersistenceSubmitError> {
        let request = SettingsPersistenceRequest::from_change(change, store, file_generation)?;
        self.submit_request(request, Some(Arc::new(observer)))
    }

    /// Re-enqueues the exact typed request after its prior worker write failed.
    ///
    /// A retry never fabricates a new authority generation. A newer setting
    /// change can therefore still supersede it through Runtime11's keyed lane.
    pub fn retry(
        &self,
        ticket: &SettingsPersistenceTicket,
    ) -> Result<SettingsPersistenceTicket, SettingsPersistenceRetryError> {
        if !matches!(ticket.terminal(), Some(BoundedKeyedIoTerminal::Failed(_))) {
            return Err(SettingsPersistenceRetryError::SourceTicketNotFailed {
                terminal: ticket.terminal(),
            });
        }

        self.submit_request(ticket.request.clone(), None)
            .map_err(|error| match error {
                SettingsPersistenceSubmitError::LaneAdmission(error) => {
                    SettingsPersistenceRetryError::LaneAdmission(error)
                }
                SettingsPersistenceSubmitError::NonPersistentScope(_) => {
                    unreachable!("a ticket retry preserves its validated persistent scope")
                }
                SettingsPersistenceSubmitError::TargetUnavailable(_)
                | SettingsPersistenceSubmitError::FileGenerationExhausted => {
                    unreachable!("a ticket retry preserves its allocated physical target")
                }
            })
    }

    pub(super) fn retry_observed(
        &self,
        ticket: &SettingsPersistenceTicket,
        observer: impl Fn(BoundedKeyedIoTerminal) + Send + Sync + 'static,
    ) -> Result<SettingsPersistenceTicket, SettingsPersistenceRetryError> {
        if !matches!(ticket.terminal(), Some(BoundedKeyedIoTerminal::Failed(_))) {
            return Err(SettingsPersistenceRetryError::SourceTicketNotFailed {
                terminal: ticket.terminal(),
            });
        }

        self.submit_request(ticket.request.clone(), Some(Arc::new(observer)))
            .map_err(|error| match error {
                SettingsPersistenceSubmitError::LaneAdmission(error) => {
                    SettingsPersistenceRetryError::LaneAdmission(error)
                }
                SettingsPersistenceSubmitError::NonPersistentScope(_) => {
                    unreachable!("a ticket retry preserves its validated persistent scope")
                }
                SettingsPersistenceSubmitError::TargetUnavailable(_)
                | SettingsPersistenceSubmitError::FileGenerationExhausted => {
                    unreachable!("a ticket retry preserves its allocated physical target")
                }
            })
    }

    fn submit_request(
        &self,
        request: SettingsPersistenceRequest,
        terminal_observer: Option<SettingsPersistenceTerminalObserver>,
    ) -> Result<SettingsPersistenceTicket, SettingsPersistenceSubmitError> {
        if !request.scope().is_persistent() {
            return Err(SettingsPersistenceSubmitError::NonPersistentScope(
                request.scope(),
            ));
        }

        let worker_request = request.clone();
        let worker_store = request.store.clone();
        let authority = Arc::clone(&self.authority);
        let lane_key = Arc::clone(&request.target);
        let admission = self
            .lane
            .try_admit(
                lane_key.clone(),
                request.file_generation().get(),
                request.retained_bytes(),
                BoundedKeyedIoWorkDeadline::none(),
                Box::new(move || {
                    worker_store
                        .save_authority_layer(worker_request.scope(), authority.as_ref())
                        .map_err(|error| {
                            tracing::warn!(
                                key = worker_request.key().as_str(),
                                scope = ?worker_request.scope(),
                                target = worker_request.target(),
                                file_generation = worker_request.file_generation().get(),
                                authority_generation = worker_request.authority_generation(),
                                error = %error,
                                "settings persistence worker could not write the authority layer"
                            );
                            zircon_runtime::core::runtime::tasks::BoundedKeyedIoFailure::new(
                                SETTINGS_PERSISTENCE_FAILURE_CODE,
                            )
                        })
                }),
            )
            .map_err(SettingsPersistenceSubmitError::LaneAdmission)?;
        if let Some(observer) = terminal_observer {
            admission.observe_terminal(move |terminal| observer(terminal));
        }
        let cancel_authority = admission.cancel_authority();
        let ticket = admission.activate();
        Ok(SettingsPersistenceTicket {
            request,
            ticket,
            cancel_authority,
        })
    }

    pub fn flush(
        &self,
        deadline: BoundedKeyedIoWorkDeadline,
    ) -> Result<BoundedKeyedIoFence, SettingsPersistenceSubmitError> {
        self.lane
            .submit_fence(0, deadline, Box::new(|| Ok(())))
            .map_err(SettingsPersistenceSubmitError::LaneAdmission)
    }

    pub fn diagnostics(&self) -> BoundedKeyedIoDiagnostics {
        self.lane.diagnostics()
    }

    pub fn shutdown(&self) -> BoundedKeyedIoShutdownGuard {
        self.lane.shutdown()
    }

    /// Fences admitted writes before closing the Runtime11 lane.
    ///
    /// The returned closeout must be finished so terminal write failures reach the host boundary.
    pub fn flush_then_shutdown(
        &self,
    ) -> Result<SettingsPersistenceShutdown, SettingsPersistenceSubmitError> {
        let fence = self.flush(BoundedKeyedIoWorkDeadline::none())?;
        Ok(SettingsPersistenceShutdown {
            fence,
            guard: self.shutdown(),
        })
    }
}

fn persistence_target(
    scope: SettingsScope,
    store: &SettingsStore,
) -> Result<Arc<str>, SettingsPersistenceSubmitError> {
    let target = match scope {
        SettingsScope::User => store.paths().user(),
        SettingsScope::Project => store
            .paths()
            .project()
            .ok_or(SettingsPersistenceSubmitError::TargetUnavailable(scope))?,
        SettingsScope::Session => {
            return Err(SettingsPersistenceSubmitError::NonPersistentScope(scope));
        }
    };
    let target_identity = blake3::hash(target.as_os_str().as_encoded_bytes()).to_hex();
    Ok(Arc::from(format!(
        "settings:{}:{}",
        scope_name(scope),
        target_identity
    )))
}

fn scope_name(scope: SettingsScope) -> &'static str {
    match scope {
        SettingsScope::User => "user",
        SettingsScope::Project => "project",
        SettingsScope::Session => "session",
    }
}
