use std::cell::Cell;
use std::collections::BTreeMap;
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use crate::core::runtime::tasks::{JobScheduler, TaskPool, TaskPools};

use super::contract::{
    NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshBudget,
    NativePluginDiscoveryRefreshError, NativePluginDiscoveryRefreshInput,
    NativePluginDiscoveryRefreshRequest, NativePluginDiscoveryRefreshSink,
    NativePluginDiscoveryRoot, NativePluginDiscoverySnapshot,
};
use super::ticket::{NativePluginDiscoveryRefreshTerminal, NativePluginDiscoveryRefreshTicket};
use super::work::NativePluginDiscoveryRefreshWork;

#[cfg(test)]
#[path = "service/pending_work_move_tests.rs"]
mod pending_work_move_tests;

thread_local! {
    static NATIVE_PLUGIN_DISCOVERY_IO_LANE: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn is_native_plugin_discovery_io_lane() -> bool {
    NATIVE_PLUGIN_DISCOVERY_IO_LANE.with(Cell::get)
}

#[cfg(test)]
pub(crate) trait NativePluginDiscoveryTestCollector: Send + Sync + 'static {
    fn collect(
        &self,
        request: &NativePluginDiscoveryRefreshRequest,
        sink: &mut NativePluginDiscoveryRefreshSink,
    ) -> Result<NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError>;
}

/// Bounded, generation-aware discovery admission and immutable snapshot publication.
///
/// Frameworks04's native discovery authority owns collection; callers consume this service's
/// last-good snapshots without synchronously scanning a root or loading a dynamic library.
#[derive(Clone)]
pub struct NativePluginDiscoveryRefreshService {
    shared: Arc<RefreshShared>,
}

struct RefreshShared {
    scheduler: JobScheduler,
    collector: RefreshCollector,
    budget: NativePluginDiscoveryRefreshBudget,
    state: Mutex<RefreshState>,
}

enum RefreshCollector {
    NativePluginAuthority,
    #[cfg(test)]
    Fixture(Arc<dyn NativePluginDiscoveryTestCollector>),
}

impl RefreshCollector {
    fn collect(
        &self,
        request: &NativePluginDiscoveryRefreshRequest,
        sink: &mut NativePluginDiscoveryRefreshSink,
    ) -> Result<NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshError> {
        match self {
            Self::NativePluginAuthority => {
                super::super::discover::authority::collect_refresh(request, sink)
            }
            #[cfg(test)]
            Self::Fixture(collector) => collector.collect(request, sink),
        }
    }
}

#[derive(Default)]
struct RefreshState {
    shutting_down: bool,
    roots: BTreeMap<NativePluginDiscoveryRefreshKey, RootRefreshState>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct NativePluginDiscoveryRefreshKey {
    root: NativePluginDiscoveryRoot,
    input: NativePluginDiscoveryRefreshInput,
}

impl NativePluginDiscoveryRefreshKey {
    fn new(root: NativePluginDiscoveryRoot, input: NativePluginDiscoveryRefreshInput) -> Self {
        Self { root, input }
    }
}

#[derive(Default)]
struct RootRefreshState {
    newest_generation: u64,
    active: Option<ActiveRefresh>,
    pending: Option<PendingRefresh>,
    published: Option<Arc<NativePluginDiscoverySnapshot>>,
    last_failure: Option<RefreshFailure>,
}

struct RefreshFailure {
    error: Arc<NativePluginDiscoveryRefreshError>,
}

struct ActiveRefresh {
    generation: u64,
    ticket: NativePluginDiscoveryRefreshTicket,
    input: NativePluginDiscoveryRefreshInput,
    work: Option<NativePluginDiscoveryRefreshWork>,
}

struct PendingRefresh {
    generation: u64,
    ticket: NativePluginDiscoveryRefreshTicket,
    input: NativePluginDiscoveryRefreshInput,
    work: NativePluginDiscoveryRefreshWork,
    base_snapshot: Option<Arc<NativePluginDiscoverySnapshot>>,
}

fn take_active_refresh_work(
    work: &mut Option<NativePluginDiscoveryRefreshWork>,
) -> NativePluginDiscoveryRefreshWork {
    work.take()
        .expect("active refresh work is available until the first pending generation")
}

impl fmt::Debug for NativePluginDiscoveryRefreshService {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativePluginDiscoveryRefreshService")
            .field("root_count", &self.root_count())
            .field("is_shutdown", &self.is_shutdown())
            .finish_non_exhaustive()
    }
}

impl NativePluginDiscoveryRefreshService {
    pub(super) fn native_plugin_authority(
        _capability: super::super::discover::authority::NativePluginDiscoveryAuthorityCapability,
        budget: NativePluginDiscoveryRefreshBudget,
    ) -> Self {
        Self::with_collector(
            RefreshCollector::NativePluginAuthority,
            TaskPools::process_default().io().clone(),
            budget,
        )
    }

    #[cfg(test)]
    pub(crate) fn new(
        collector: Arc<dyn NativePluginDiscoveryTestCollector>,
        budget: NativePluginDiscoveryRefreshBudget,
    ) -> Self {
        Self::with_pool(collector, TaskPools::process_default().io().clone(), budget)
    }

    #[cfg(test)]
    pub(crate) fn with_pool(
        collector: Arc<dyn NativePluginDiscoveryTestCollector>,
        pool: TaskPool,
        budget: NativePluginDiscoveryRefreshBudget,
    ) -> Self {
        Self::with_collector(RefreshCollector::Fixture(collector), pool, budget)
    }

    fn with_collector(
        collector: RefreshCollector,
        pool: TaskPool,
        budget: NativePluginDiscoveryRefreshBudget,
    ) -> Self {
        Self {
            shared: Arc::new(RefreshShared {
                scheduler: JobScheduler::from_pool(pool),
                collector,
                budget: budget.normalized(),
                state: Mutex::new(RefreshState::default()),
            }),
        }
    }

    /// Admits one latest-wins root generation without performing collector I/O on the caller.
    pub fn submit(&self, root: NativePluginDiscoveryRoot) -> NativePluginDiscoveryRefreshTicket {
        self.submit_with_input(root, NativePluginDiscoveryRefreshInput::root_scan())
    }

    /// Internal entry for the authority's non-scan selection input. The public service surface
    /// remains root scanning only; callers cannot inject a collector or bypass the authority.
    pub(in crate::plugin::native_plugin_loader) fn submit_with_input(
        &self,
        root: NativePluginDiscoveryRoot,
        input: NativePluginDiscoveryRefreshInput,
    ) -> NativePluginDiscoveryRefreshTicket {
        self.submit_with_work(root, input, NativePluginDiscoveryRefreshWork::root_scan())
    }

    /// Admits one root/input generation while retaining watcher work on the ticket rather than
    /// splitting `(root, input)` snapshot history by changed path.
    pub(in crate::plugin::native_plugin_loader) fn submit_with_work(
        &self,
        root: NativePluginDiscoveryRoot,
        input: NativePluginDiscoveryRefreshInput,
        work: NativePluginDiscoveryRefreshWork,
    ) -> NativePluginDiscoveryRefreshTicket {
        let key = NativePluginDiscoveryRefreshKey::new(root.clone(), input.clone());
        let now = Instant::now();
        let Some(deadline) = now.checked_add(self.shared.budget.deadline) else {
            let ticket = self.new_ticket(root, 0, now);
            ticket.finish(NativePluginDiscoveryRefreshTerminal::Rejected {
                reason: Arc::from("native plugin discovery deadline exceeds Instant range"),
            });
            return ticket;
        };
        let mut terminals = Vec::new();
        let mut launch = None;
        let ticket = {
            let mut state = self.lock_state();
            if state.shutting_down {
                let ticket = self.new_ticket(root, 0, deadline);
                terminals.push((
                    ticket.clone(),
                    NativePluginDiscoveryRefreshTerminal::Shutdown,
                ));
                ticket
            // A root with different collector inputs owns independent generations and
            // publications. Charge each key to the bounded state budget so input variation
            // cannot create unbounded refresh state behind one filesystem path.
            } else if !state.roots.contains_key(&key)
                && state.roots.len() >= self.shared.budget.max_roots
            {
                let ticket = self.new_ticket(root, 0, deadline);
                terminals.push((
                    ticket.clone(),
                    NativePluginDiscoveryRefreshTerminal::Rejected {
                        reason: Arc::from(
                            "native plugin discovery root admission budget exhausted",
                        ),
                    },
                ));
                ticket
            } else {
                let root_state = state.roots.entry(key).or_default();
                if root_state.active.is_some() {
                    if let Some(pending) = root_state.pending.as_mut() {
                        pending.work.merge(work);
                        pending.ticket.clone()
                    } else {
                        let (active_ticket, active_generation, active_work) = {
                            let active = root_state.active.as_mut().expect("active refresh");
                            (
                                active.ticket.clone(),
                                active.generation,
                                take_active_refresh_work(&mut active.work),
                            )
                        };
                        let mut pending_work = active_work;
                        pending_work.merge(work);
                        root_state.newest_generation =
                            root_state.newest_generation.saturating_add(1);
                        let generation = root_state.newest_generation;
                        let ticket = self.new_ticket(root.clone(), generation, deadline);
                        active_ticket.cancellation().cancel();
                        terminals.push((
                            active_ticket,
                            NativePluginDiscoveryRefreshTerminal::Superseded {
                                generation: active_generation,
                            },
                        ));
                        root_state.pending = Some(PendingRefresh {
                            generation,
                            ticket: ticket.clone(),
                            input: input.clone(),
                            work: pending_work,
                            base_snapshot: root_state.published.clone(),
                        });
                        ticket
                    }
                } else {
                    root_state.newest_generation = root_state.newest_generation.saturating_add(1);
                    let generation = root_state.newest_generation;
                    let ticket = self.new_ticket(root.clone(), generation, deadline);
                    let base_snapshot = root_state.published.clone();
                    root_state.active = Some(ActiveRefresh {
                        generation,
                        ticket: ticket.clone(),
                        input: input.clone(),
                        work: Some(work.clone()),
                    });
                    launch = Some((root, generation, ticket.clone(), input, work, base_snapshot));
                    ticket
                }
            }
        };

        for (ticket, terminal) in terminals {
            ticket.finish(terminal);
        }
        if let Some((root, generation, ticket, input, work, base_snapshot)) = launch {
            launch_generation(
                Arc::clone(&self.shared),
                root,
                input,
                work,
                base_snapshot,
                generation,
                ticket,
            );
        }
        ticket
    }

    pub fn snapshot(
        &self,
        root: &NativePluginDiscoveryRoot,
    ) -> Option<Arc<NativePluginDiscoverySnapshot>> {
        self.snapshot_for(root, &NativePluginDiscoveryRefreshInput::root_scan())
    }

    pub(in crate::plugin::native_plugin_loader) fn snapshot_for(
        &self,
        root: &NativePluginDiscoveryRoot,
        input: &NativePluginDiscoveryRefreshInput,
    ) -> Option<Arc<NativePluginDiscoverySnapshot>> {
        let key = NativePluginDiscoveryRefreshKey::new(root.clone(), input.clone());
        self.lock_state()
            .roots
            .get(&key)
            .and_then(|state| state.published.clone())
    }

    pub fn last_failure(
        &self,
        root: &NativePluginDiscoveryRoot,
    ) -> Option<Arc<NativePluginDiscoveryRefreshError>> {
        self.last_failure_for(root, &NativePluginDiscoveryRefreshInput::root_scan())
    }

    pub(in crate::plugin::native_plugin_loader) fn last_failure_for(
        &self,
        root: &NativePluginDiscoveryRoot,
        input: &NativePluginDiscoveryRefreshInput,
    ) -> Option<Arc<NativePluginDiscoveryRefreshError>> {
        let key = NativePluginDiscoveryRefreshKey::new(root.clone(), input.clone());
        self.lock_state()
            .roots
            .get(&key)
            .and_then(|state| state.last_failure.as_ref())
            .map(|failure| Arc::clone(&failure.error))
    }

    /// Terminalizes pending and active generations while retaining immutable last-good snapshots.
    pub(super) fn shutdown(&self) {
        let mut terminals = Vec::new();
        {
            let mut state = self.lock_state();
            if state.shutting_down {
                return;
            }
            state.shutting_down = true;
            for root_state in state.roots.values_mut() {
                if let Some(active) = root_state.active.take() {
                    active.ticket.cancellation().cancel();
                    terminals.push((
                        active.ticket,
                        NativePluginDiscoveryRefreshTerminal::Shutdown,
                    ));
                }
                if let Some(pending) = root_state.pending.take() {
                    pending.ticket.cancellation().cancel();
                    terminals.push((
                        pending.ticket,
                        NativePluginDiscoveryRefreshTerminal::Shutdown,
                    ));
                }
            }
        }
        for (ticket, terminal) in terminals {
            ticket.finish(terminal);
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.lock_state().shutting_down
    }

    pub fn root_count(&self) -> usize {
        self.lock_state().roots.len()
    }

    fn new_ticket(
        &self,
        root: NativePluginDiscoveryRoot,
        generation: u64,
        deadline: Instant,
    ) -> NativePluginDiscoveryRefreshTicket {
        NativePluginDiscoveryRefreshTicket::new(
            root,
            generation,
            deadline,
            self.shared.budget.max_terminal_observers,
        )
    }

    fn lock_state(&self) -> MutexGuard<'_, RefreshState> {
        self.shared
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn launch_generation(
    shared: Arc<RefreshShared>,
    root: NativePluginDiscoveryRoot,
    input: NativePluginDiscoveryRefreshInput,
    work: NativePluginDiscoveryRefreshWork,
    base_snapshot: Option<Arc<NativePluginDiscoverySnapshot>>,
    generation: u64,
    ticket: NativePluginDiscoveryRefreshTicket,
) {
    let completed = Arc::new(AtomicBool::new(false));
    let task_shared = Arc::clone(&shared);
    let task_root = root.clone();
    let task_input = input.clone();
    let task_work = work.clone();
    let task_base_snapshot = base_snapshot.clone();
    let completion_input = input.clone();
    let task_ticket = ticket.clone();
    let task_completed = Arc::clone(&completed);
    let handle = shared.scheduler.schedule(move || {
        NATIVE_PLUGIN_DISCOVERY_IO_LANE.with(|in_lane| {
            let previous = in_lane.replace(true);
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                collect_generation(
                    &task_shared,
                    task_root.clone(),
                    task_input,
                    task_work,
                    task_base_snapshot,
                    generation,
                    &task_ticket,
                )
            }))
            .unwrap_or_else(|_| {
                Err(Arc::new(NativePluginDiscoveryRefreshError::collector(
                    "native plugin discovery collector panicked",
                )))
            });
            // `complete_generation` delivers terminal observers. Keep the lane marker set until
            // that delivery has returned so observer re-entry projects last-good state instead of
            // synchronously waiting on this same I/O pool.
            complete_generation(
                &task_shared,
                task_root,
                completion_input,
                generation,
                task_ticket,
                outcome,
            );
            task_completed.store(true, Ordering::Release);
            in_lane.set(previous);
        });
    });

    // The scheduler owns panic containment and executes this observer once after any terminal
    // task state. The fallback prevents a scheduler-level panic from leaving a root in-flight.
    let fallback_shared = Arc::clone(&shared);
    handle.on_terminal(move || {
        if !completed.load(Ordering::Acquire) {
            NATIVE_PLUGIN_DISCOVERY_IO_LANE.with(|in_lane| {
                let previous = in_lane.replace(true);
                complete_generation(
                    &fallback_shared,
                    root,
                    input,
                    generation,
                    ticket,
                    Err(Arc::new(NativePluginDiscoveryRefreshError::collector(
                        "native plugin discovery task terminated before publication",
                    ))),
                );
                in_lane.set(previous);
            });
        }
    });
}

fn collect_generation(
    shared: &RefreshShared,
    root: NativePluginDiscoveryRoot,
    input: NativePluginDiscoveryRefreshInput,
    work: NativePluginDiscoveryRefreshWork,
    base_snapshot: Option<Arc<NativePluginDiscoverySnapshot>>,
    generation: u64,
    ticket: &NativePluginDiscoveryRefreshTicket,
) -> Result<Arc<NativePluginDiscoverySnapshot>, Arc<NativePluginDiscoveryRefreshError>> {
    let request = NativePluginDiscoveryRefreshRequest::new(
        root.clone(),
        input.clone(),
        work.clone(),
        base_snapshot.clone(),
        generation,
        shared.budget.clone(),
        ticket.cancellation(),
    );
    request.check_active().map_err(Arc::new)?;
    let mut sink = NativePluginDiscoveryRefreshSink::new(shared.budget.clone());
    let input_identity = match shared.collector.collect(&request, &mut sink) {
        Ok(input_identity) => input_identity,
        Err(error) => {
            request.check_active().map_err(Arc::new)?;
            return Err(Arc::new(error));
        }
    };
    request.check_active().map_err(Arc::new)?;
    let payload = sink.into_payload(input_identity);
    request.check_active().map_err(Arc::new)?;
    let snapshot = match (&input, &work, base_snapshot.as_deref()) {
        (
            NativePluginDiscoveryRefreshInput::RootScan,
            NativePluginDiscoveryRefreshWork::ManifestBatch { .. },
            Some(base_snapshot),
        ) => NativePluginDiscoverySnapshot::from_incremental_payload(
            root,
            input,
            generation,
            base_snapshot,
            &work,
            payload,
            shared.budget.max_candidates,
        )
        .map_err(Arc::new)?,
        _ => NativePluginDiscoverySnapshot::from_payload(root, input, generation, payload),
    };
    Ok(Arc::new(snapshot))
}

fn complete_generation(
    shared: &Arc<RefreshShared>,
    root: NativePluginDiscoveryRoot,
    input: NativePluginDiscoveryRefreshInput,
    generation: u64,
    ticket: NativePluginDiscoveryRefreshTicket,
    outcome: Result<Arc<NativePluginDiscoverySnapshot>, Arc<NativePluginDiscoveryRefreshError>>,
) {
    let (terminal, delivery, launch) = {
        let mut state = shared
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let shutting_down = state.shutting_down;
        let key = NativePluginDiscoveryRefreshKey::new(root.clone(), input.clone());
        let Some(root_state) = state.roots.get_mut(&key) else {
            return;
        };
        let Some(active) = root_state.active.as_ref() else {
            return;
        };
        if active.generation != generation {
            return;
        }
        debug_assert_eq!(active.input, input);
        root_state.active = None;

        let mut delivery = None;
        let terminal = if shutting_down {
            Some(NativePluginDiscoveryRefreshTerminal::Shutdown)
        } else if root_state.newest_generation != generation {
            Some(NativePluginDiscoveryRefreshTerminal::Superseded { generation })
        } else if let Some(existing_terminal) = ticket.terminal() {
            // `wait_terminal` can terminalize a queued ticket at its deadline before a worker
            // receives scheduler time. Preserve that reason when the worker later retires the
            // active generation; otherwise the authority would report a generic no-snapshot
            // failure instead of the actual deadline breach.
            match &existing_terminal {
                NativePluginDiscoveryRefreshTerminal::DeadlineExceeded => {
                    root_state.last_failure = Some(RefreshFailure {
                        error: Arc::new(NativePluginDiscoveryRefreshError::deadline_exceeded()),
                    });
                }
                NativePluginDiscoveryRefreshTerminal::Failed(error) => {
                    root_state.last_failure = Some(RefreshFailure {
                        error: Arc::clone(error),
                    });
                }
                _ => {}
            }
            Some(existing_terminal)
        } else if ticket.cancellation().is_explicitly_cancelled() {
            Some(NativePluginDiscoveryRefreshTerminal::Cancelled)
        } else {
            match outcome {
                Ok(snapshot) => {
                    let published =
                        NativePluginDiscoveryRefreshTerminal::Published(Arc::clone(&snapshot));
                    if let Some(reserved) = ticket.reserve_terminal(published) {
                        root_state.published = Some(snapshot);
                        root_state.last_failure = None;
                        delivery = Some(reserved);
                    }
                    None
                }
                Err(error) => {
                    let terminal = if matches!(
                        error.as_ref(),
                        NativePluginDiscoveryRefreshError::DeadlineExceeded
                    ) {
                        NativePluginDiscoveryRefreshTerminal::DeadlineExceeded
                    } else {
                        NativePluginDiscoveryRefreshTerminal::Failed(Arc::clone(&error))
                    };
                    if let Some(reserved) = ticket.reserve_terminal(terminal) {
                        root_state.last_failure = Some(RefreshFailure { error });
                        delivery = Some(reserved);
                    }
                    None
                }
            }
        };

        let launch = if shutting_down {
            None
        } else {
            root_state.pending.take().and_then(|pending| {
                if pending.ticket.is_complete() {
                    None
                } else {
                    root_state.active = Some(ActiveRefresh {
                        generation: pending.generation,
                        ticket: pending.ticket.clone(),
                        input: pending.input.clone(),
                        work: Some(pending.work.clone()),
                    });
                    Some((
                        root.clone(),
                        pending.input,
                        pending.work,
                        pending.base_snapshot,
                        pending.generation,
                        pending.ticket,
                    ))
                }
            })
        };
        (terminal, delivery, launch)
    };

    if let Some(delivery) = delivery {
        delivery.deliver();
    }
    if let Some(terminal) = terminal {
        ticket.finish(terminal);
    }
    if let Some((root, input, work, base_snapshot, generation, ticket)) = launch {
        launch_generation(
            Arc::clone(shared),
            root,
            input,
            work,
            base_snapshot,
            generation,
            ticket,
        );
    }
}
