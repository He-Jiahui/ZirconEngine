use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use crate::core::runtime::{
    BoundedKeyedIoCancelAuthority, BoundedKeyedIoCancelError, BoundedKeyedIoLane,
    BoundedKeyedIoLimits, BoundedKeyedIoTerminal, BoundedKeyedIoTicket, BoundedKeyedIoWaitResult,
    BoundedKeyedIoWorkDeadline, JobScheduler, TaskPool,
};
use crate::core::{CoreHandle, CoreWeak};
use crate::script::VmError;

use super::{
    discover_vm_plugin_packages_cancellable, DiscoveredVmPluginPackage, VmPluginDiscoveryLimits,
};

const MAX_PENDING_DISCOVERY_REQUESTS: usize = 4;
const DISCOVERY_RUNTIME_OWNER_UNAVAILABLE: &str =
    "VM plugin discovery runtime task owner is unavailable";

type DiscoveryResult = Result<Vec<DiscoveredVmPluginPackage>, VmError>;
type SharedDiscoveryResult = Arc<Mutex<Option<DiscoveryResult>>>;

pub struct VmPluginDiscoveryRequest {
    ticket: BoundedKeyedIoTicket,
    cancel_authority: BoundedKeyedIoCancelAuthority,
    result: SharedDiscoveryResult,
    cancellation: Arc<AtomicBool>,
    observer_deadline: Instant,
}

impl VmPluginDiscoveryRequest {
    pub fn generation(&self) -> u64 {
        self.ticket.generation()
    }

    pub fn is_terminal(&self) -> bool {
        self.ticket.terminal().is_some()
    }

    pub fn cancel_before_start(&self) -> Result<(), VmError> {
        self.ticket
            .cancel_before_start(&self.cancel_authority)
            .map_err(|error| {
                VmError::Operation(format!(
                    "plugin discovery request {} could not be cancelled: {error:?}",
                    self.ticket.id()
                ))
            })
    }

    pub fn cancel(&self) -> Result<(), VmError> {
        self.cancellation.store(true, Ordering::Release);
        match self.ticket.cancel_before_start(&self.cancel_authority) {
            Ok(()) | Err(BoundedKeyedIoCancelError::AlreadyStarted) => Ok(()),
            Err(error) => Err(VmError::Operation(format!(
                "plugin discovery request {} could not be cancelled: {error:?}",
                self.ticket.id()
            ))),
        }
    }

    pub fn wait(self) -> DiscoveryResult {
        let deadline = self.observer_deadline;
        self.wait_until(deadline)
    }

    pub fn wait_until(self, deadline: Instant) -> DiscoveryResult {
        let terminal = match self.ticket.wait_until(deadline) {
            BoundedKeyedIoWaitResult::Terminal(terminal) => terminal,
            BoundedKeyedIoWaitResult::ObserverTimedOut => {
                let _ = self.ticket.cancel_before_start(&self.cancel_authority);
                return Err(VmError::Operation(format!(
                    "plugin discovery request {} exceeded its observer deadline",
                    self.ticket.id()
                )));
            }
        };
        match terminal {
            BoundedKeyedIoTerminal::Succeeded => self.take_result().unwrap_or_else(|| {
                Err(VmError::Operation(format!(
                    "plugin discovery request {} completed without a result",
                    self.ticket.id()
                )))
            }),
            BoundedKeyedIoTerminal::Failed(failure) => Err(VmError::Operation(format!(
                "plugin discovery request {} failed in the I/O lane: {}",
                self.ticket.id(),
                failure.code
            ))),
            BoundedKeyedIoTerminal::DeadlineBeforeStart => Err(VmError::Operation(format!(
                "plugin discovery request {} reached its deadline before starting",
                self.ticket.id()
            ))),
            BoundedKeyedIoTerminal::CancelledBeforeStart => Err(VmError::Operation(format!(
                "plugin discovery request {} was cancelled before starting",
                self.ticket.id()
            ))),
            BoundedKeyedIoTerminal::Superseded { successor } => Err(VmError::Operation(format!(
                "plugin discovery request {} was superseded by generation {successor}",
                self.ticket.id()
            ))),
            BoundedKeyedIoTerminal::Shutdown => Err(VmError::Operation(format!(
                "plugin discovery request {} was stopped by shutdown",
                self.ticket.id()
            ))),
        }
    }

    fn take_result(&self) -> Option<DiscoveryResult> {
        self.result_lock().take()
    }

    fn result_lock(&self) -> MutexGuard<'_, Option<DiscoveryResult>> {
        self.result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl fmt::Debug for VmPluginDiscoveryRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VmPluginDiscoveryRequest")
            .field("generation", &self.generation())
            .field("terminal", &self.ticket.terminal())
            .finish()
    }
}

impl Drop for VmPluginDiscoveryRequest {
    fn drop(&mut self) {
        self.cancellation.store(true, Ordering::Release);
        let _ = self.ticket.cancel_before_start(&self.cancel_authority);
    }
}

pub(crate) struct VmPluginDiscoveryWorker {
    limits: VmPluginDiscoveryLimits,
    backend: VmPluginDiscoveryBackend,
    next_generation: AtomicU64,
    retained_bytes_per_request: usize,
}

enum VmPluginDiscoveryBackend {
    Runtime {
        lane: BoundedKeyedIoLane,
        worker_pool: TaskPool,
        runtime_owner: CoreWeak,
    },
    Unavailable,
}

impl VmPluginDiscoveryWorker {
    pub(crate) fn with_runtime(limits: VmPluginDiscoveryLimits, runtime: &CoreHandle) -> Self {
        let worker_pool = runtime.task_graph().worker_pool().clone();
        let scheduler = JobScheduler::from_pool(worker_pool.clone());
        let retained_bytes_per_request = limits
            .max_total_manifest_bytes
            .saturating_add(limits.max_total_path_bytes);
        Self {
            limits,
            backend: VmPluginDiscoveryBackend::Runtime {
                lane: BoundedKeyedIoLane::new(
                    BoundedKeyedIoLimits::new(
                        MAX_PENDING_DISCOVERY_REQUESTS,
                        retained_bytes_per_request.saturating_mul(MAX_PENDING_DISCOVERY_REQUESTS),
                    ),
                    scheduler,
                ),
                worker_pool,
                runtime_owner: runtime.downgrade(),
            },
            next_generation: AtomicU64::new(0),
            retained_bytes_per_request,
        }
    }

    pub(crate) fn unavailable(limits: VmPluginDiscoveryLimits) -> Self {
        let retained_bytes_per_request = limits
            .max_total_manifest_bytes
            .saturating_add(limits.max_total_path_bytes);
        Self {
            limits,
            backend: VmPluginDiscoveryBackend::Unavailable,
            next_generation: AtomicU64::new(0),
            retained_bytes_per_request,
        }
    }

    pub(crate) fn is_current_io_worker(&self) -> bool {
        match &self.backend {
            VmPluginDiscoveryBackend::Runtime { worker_pool, .. } => {
                worker_pool.is_current_worker()
            }
            VmPluginDiscoveryBackend::Unavailable => false,
        }
    }

    pub(crate) fn submit(&self, root: PathBuf) -> Result<VmPluginDiscoveryRequest, VmError> {
        let (lane, runtime_admission_lease) = match &self.backend {
            VmPluginDiscoveryBackend::Runtime {
                lane,
                runtime_owner,
                ..
            } => {
                let runtime_lease = runtime_owner.upgrade().ok_or_else(|| {
                    VmError::Operation(DISCOVERY_RUNTIME_OWNER_UNAVAILABLE.to_string())
                })?;
                (lane, runtime_lease)
            }
            VmPluginDiscoveryBackend::Unavailable => {
                return Err(VmError::Operation(
                    DISCOVERY_RUNTIME_OWNER_UNAVAILABLE.to_string(),
                ));
            }
        };
        let generation = self
            .next_generation
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| VmError::Operation("plugin discovery generation exhausted".to_string()))?
            .saturating_add(1);
        let now = Instant::now();
        let start_deadline = now.checked_add(self.limits.max_wall_time).ok_or_else(|| {
            VmError::Operation("plugin discovery deadline overflowed".to_string())
        })?;
        let observer_deadline = start_deadline
            .checked_add(self.limits.max_wall_time)
            .ok_or_else(|| {
                VmError::Operation("plugin discovery observer deadline overflowed".to_string())
            })?;
        let key = format!("script.vm.discovery:{}", root.display());
        let result: SharedDiscoveryResult = Arc::new(Mutex::new(None));
        let worker_result = Arc::clone(&result);
        let cancellation = Arc::new(AtomicBool::new(false));
        let worker_cancellation = Arc::clone(&cancellation);
        let limits = self.limits;
        let admission = lane
            .try_admit(
                key,
                generation,
                self.retained_bytes_per_request,
                BoundedKeyedIoWorkDeadline::at(start_deadline),
                Box::new(move || {
                    let discovered =
                        discover_vm_plugin_packages_cancellable(root, limits, worker_cancellation);
                    *worker_result
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(discovered);
                    Ok(())
                }),
            )
            .map_err(|error| {
                VmError::Operation(format!("plugin discovery I/O admission failed: {error:?}"))
            })?;
        let cancel_authority = admission.cancel_authority();
        let ticket = admission.activate();
        drop(runtime_admission_lease);
        Ok(VmPluginDiscoveryRequest {
            ticket,
            cancel_authority,
            result,
            cancellation,
            observer_deadline,
        })
    }
}

impl fmt::Debug for VmPluginDiscoveryWorker {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("VmPluginDiscoveryWorker");
        debug.field("limits", &self.limits);
        match &self.backend {
            VmPluginDiscoveryBackend::Runtime { lane, .. } => {
                debug.field("diagnostics", &lane.diagnostics());
            }
            VmPluginDiscoveryBackend::Unavailable => {
                debug.field("backend", &"unavailable");
            }
        }
        debug.finish()
    }
}

impl Drop for VmPluginDiscoveryWorker {
    fn drop(&mut self) {
        if let VmPluginDiscoveryBackend::Runtime { lane, .. } = &self.backend {
            drop(lane.shutdown());
        }
    }
}
