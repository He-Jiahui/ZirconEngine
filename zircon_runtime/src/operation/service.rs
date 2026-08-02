use std::collections::{BTreeMap, HashMap};
use std::io::{self, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use zircon_runtime_interface::{
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle,
    ZrRuntimeOperationPhase, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
};

use crate::core::CoreHandle;
use crate::core::runtime::tasks::TaskTimerSubscription;
use crate::scene::World;

use super::maintenance::{
    expire_due_deadlines_in_state, expire_terminal_results_in_state, lock_operation_state,
    refresh_operation_maintenance_alarm,
};
use super::task::RuntimeOperationTask;
use super::{RuntimeOperationContext, RuntimeOperationHandler, RuntimeOperationServiceError};

mod admission;
mod completion;

const DEFAULT_MAX_TASKS: usize = 1_024;
const DEFAULT_MAX_IN_FLIGHT_PREPARES: usize = 32;
const DEFAULT_MAX_RETAINED_BYTES: usize = 4 * 1024 * 1024;
const DEFAULT_MAX_OWNER_APPLIES_PER_TICK: usize = 8;
const DEFAULT_TERMINAL_RESULT_TTL: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug)]
pub(super) struct RuntimeOperationLimits {
    pub(super) max_tasks: usize,
    pub(super) max_in_flight_prepares: usize,
    pub(super) max_retained_bytes: usize,
    pub(super) max_owner_applies_per_tick: usize,
    pub(super) terminal_result_ttl: Duration,
}

impl Default for RuntimeOperationLimits {
    fn default() -> Self {
        Self {
            max_tasks: DEFAULT_MAX_TASKS,
            max_in_flight_prepares: DEFAULT_MAX_IN_FLIGHT_PREPARES,
            max_retained_bytes: DEFAULT_MAX_RETAINED_BYTES,
            max_owner_applies_per_tick: DEFAULT_MAX_OWNER_APPLIES_PER_TICK,
            terminal_result_ttl: DEFAULT_TERMINAL_RESULT_TTL,
        }
    }
}

#[derive(Default)]
pub(super) struct RuntimeOperationTaskState {
    pub(super) next_handle: u64,
    pub(super) tasks: HashMap<ZrRuntimeOperationHandle, RuntimeOperationTask>,
    pub(super) retained_bytes: usize,
    pub(super) pending_admissions: usize,
    pub(super) pending_admission_bytes: usize,
    pub(super) in_flight_prepares: usize,
    pub(super) maintenance_subscription: Option<TaskTimerSubscription>,
    pub(super) maintenance_deadline: Option<Instant>,
    pub(super) maintenance_generation: u64,
}

impl RuntimeOperationTaskState {
    fn allocate_handle(
        &mut self,
    ) -> Result<ZrRuntimeOperationHandle, RuntimeOperationServiceError> {
        if self.next_handle == 0 {
            self.next_handle = 1;
        }
        let handle = ZrRuntimeOperationHandle::new(self.next_handle);
        self.next_handle = self
            .next_handle
            .checked_add(1)
            .ok_or(RuntimeOperationServiceError::HandleExhausted)?;
        Ok(handle)
    }
}

enum RuntimeOperationPrepareCompletion {
    Prepared {
        handle: ZrRuntimeOperationHandle,
        command: serde_json::Value,
        result: serde_json::Value,
        command_bytes: usize,
        result_bytes: usize,
    },
    Failed {
        handle: ZrRuntimeOperationHandle,
        error: String,
        detail_kind: ZrRuntimeOperationDetailKindV2,
    },
}

struct RuntimeOperationCompletionBatch {
    sender: SyncSender<RuntimeOperationPrepareCompletion>,
    receiver: Receiver<RuntimeOperationPrepareCompletion>,
    handles: Vec<ZrRuntimeOperationHandle>,
}

struct RuntimeOperationCompletionReceiver {
    receiver: Receiver<RuntimeOperationPrepareCompletion>,
    handles: Vec<ZrRuntimeOperationHandle>,
}

#[derive(Clone, Copy)]
struct RuntimeOperationAdmissionReservation {
    bytes: usize,
}

/// Registry, bounded task store, and prepare completion queue for one runtime session.
pub struct RuntimeOperationService {
    handlers: BTreeMap<String, Arc<dyn RuntimeOperationHandler>>,
    limits: RuntimeOperationLimits,
    state: Arc<Mutex<RuntimeOperationTaskState>>,
    maintenance_refresh: Arc<Mutex<()>>,
    completion_receivers: Mutex<Vec<RuntimeOperationCompletionReceiver>>,
}

impl RuntimeOperationService {
    pub fn new() -> Self {
        Self::with_limits(RuntimeOperationLimits::default())
    }

    pub(super) fn with_limits(limits: RuntimeOperationLimits) -> Self {
        Self {
            handlers: BTreeMap::new(),
            limits,
            state: Arc::new(Mutex::new(RuntimeOperationTaskState::default())),
            maintenance_refresh: Arc::new(Mutex::new(())),
            completion_receivers: Mutex::new(Vec::new()),
        }
    }

    pub fn register_handler(
        &mut self,
        operation_id: impl Into<String>,
        handler: Arc<dyn RuntimeOperationHandler>,
    ) -> Result<(), RuntimeOperationServiceError> {
        let operation_id = operation_id.into();
        if operation_id.trim().is_empty() {
            return Err(RuntimeOperationServiceError::EmptyOperationId);
        }
        if self.handlers.contains_key(&operation_id) {
            return Err(RuntimeOperationServiceError::DuplicateHandler { operation_id });
        }
        self.handlers.insert(operation_id, handler);
        Ok(())
    }

    /// Upper bound for the raw V1 request accepted by this session.
    pub fn max_retained_bytes(&self) -> usize {
        self.limits.max_retained_bytes
    }

    /// Returns a snapshot only. Runtime work is dispatched exclusively by `tick`.
    pub fn poll(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, RuntimeOperationServiceError> {
        let state = self.lock_state();
        state
            .tasks
            .get(&handle)
            .map(RuntimeOperationTask::status)
            .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })
    }

    /// Advances bounded owner snapshots, worker preparation, and owner applies per runtime frame.
    pub fn tick(&self, core: &CoreHandle, world: &mut World) {
        self.expire_due_deadlines(Instant::now());
        self.expire_terminal_results(Instant::now());
        self.drain_prepare_completions();
        self.expire_due_deadlines(Instant::now());
        self.apply_prepared(core, world);
        self.expire_due_deadlines(Instant::now());
        self.snapshot_and_dispatch_queued_prepares(core, world);
    }

    /// Cancels an operation before its owner-thread apply phase is claimed.
    pub fn cancel(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<(), RuntimeOperationServiceError> {
        let now = Instant::now();
        let mut state = self.lock_state();
        let released_bytes = {
            let task = state
                .tasks
                .get_mut(&handle)
                .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })?;
            if task.apply_claimed
                || !matches!(
                    task.phase,
                    ZrRuntimeOperationPhase::Queued
                        | ZrRuntimeOperationPhase::Preparing
                        | ZrRuntimeOperationPhase::ReadyToApply
                )
            {
                return Err(RuntimeOperationServiceError::NotCancellable { handle });
            }
            let released_bytes = std::mem::replace(&mut task.retained_bytes, 0);
            task.payload = None;
            task.prepared_command = None;
            task.prepared_result = None;
            task.prepared_command_bytes = 0;
            task.prepared_result_bytes = 0;
            task.result = None;
            task.deadline_armed = false;
            task.snapshot_claimed = false;
            task.phase = ZrRuntimeOperationPhase::Cancelled;
            task.detail_kind = ZrRuntimeOperationDetailKindV2::Cancelled;
            task.detail_value = 0;
            task.terminal_at = Some(now);
            released_bytes
        };
        state.retained_bytes = state
            .retained_bytes
            .checked_sub(released_bytes)
            .expect("cancelled operation bytes must remain accounted");
        drop(state);
        self.refresh_maintenance_after_transition();
        Ok(())
    }

    pub fn harvest(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, RuntimeOperationServiceError> {
        let mut state = self.lock_state();
        let (result, released_bytes, terminal_phase) = {
            let task = state
                .tasks
                .get_mut(&handle)
                .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })?;
            if task.phase == ZrRuntimeOperationPhase::Harvested {
                return Err(RuntimeOperationServiceError::AlreadyHarvested { handle });
            }
            if task.phase == ZrRuntimeOperationPhase::Cancelled {
                return Err(RuntimeOperationServiceError::OperationCancelled { handle });
            }
            if task.phase == ZrRuntimeOperationPhase::Expired {
                return Err(RuntimeOperationServiceError::OperationExpired { handle });
            }
            if !matches!(
                task.phase,
                ZrRuntimeOperationPhase::Completed | ZrRuntimeOperationPhase::Failed
            ) {
                return Err(RuntimeOperationServiceError::NotTerminal { handle });
            }
            let terminal_phase = task.phase;
            let result = task
                .result
                .take()
                .ok_or(RuntimeOperationServiceError::NotTerminal { handle })?;
            let released_bytes = std::mem::replace(&mut task.retained_bytes, 0);
            task.payload = None;
            task.prepared_command = None;
            task.prepared_result = None;
            task.prepared_command_bytes = 0;
            task.prepared_result_bytes = 0;
            task.deadline_armed = false;
            task.phase = ZrRuntimeOperationPhase::Harvested;
            task.detail_kind = ZrRuntimeOperationDetailKindV2::Harvested;
            task.detail_value = u64::from(terminal_phase.raw());
            (result, released_bytes, terminal_phase)
        };
        debug_assert!(terminal_phase.is_terminal());
        state.retained_bytes = state
            .retained_bytes
            .checked_sub(released_bytes)
            .expect("harvested operation bytes must remain accounted");
        drop(state);
        self.refresh_maintenance_after_transition();
        Ok(result)
    }

    fn arm_deadline(
        &self,
        handle: ZrRuntimeOperationHandle,
        deadline: Instant,
    ) -> Result<(), RuntimeOperationServiceError> {
        self.refresh_maintenance().map_err(|error| {
            self.rollback_unarmed_admission(handle);
            error
        })?;
        let mut state = self.lock_state();
        let Some(task) = state.tasks.get_mut(&handle) else {
            return Ok(());
        };
        if matches!(
            task.phase,
            ZrRuntimeOperationPhase::Queued
                | ZrRuntimeOperationPhase::Preparing
                | ZrRuntimeOperationPhase::ReadyToApply
        ) && task.deadline == Some(deadline)
        {
            task.deadline_armed = true;
        }
        Ok(())
    }

    fn rollback_unarmed_admission(&self, handle: ZrRuntimeOperationHandle) {
        let mut state = self.lock_state();
        let Some(task) = state.tasks.remove(&handle) else {
            return;
        };
        debug_assert!(!task.deadline_armed);
        state.retained_bytes = state
            .retained_bytes
            .checked_sub(task.retained_bytes)
            .expect("rolled back operation bytes must remain accounted");
    }

    fn snapshot_and_dispatch_queued_prepares(&self, core: &CoreHandle, world: &mut World) {
        let mut completion_batch = None;
        for _ in 0..self.limits.max_owner_applies_per_tick {
            let Some((handle, handler, payload)) = self.take_queued_snapshot_task() else {
                break;
            };
            let snapshot = catch_unwind(AssertUnwindSafe(|| {
                handler.snapshot(RuntimeOperationContext::new(core, world), payload)
            }));
            let snapshot = match snapshot {
                Ok(Ok(snapshot)) => snapshot,
                Ok(Err(error)) => {
                    self.finish_snapshot_failed_task(handle, error.to_string());
                    continue;
                }
                Err(_) => {
                    self.finish_snapshot_failed_task(
                        handle,
                        "runtime operation owner snapshot panicked".to_owned(),
                    );
                    continue;
                }
            };
            let should_dispatch = {
                let mut state = self.lock_state();
                if state.in_flight_prepares >= self.limits.max_in_flight_prepares {
                    false
                } else if let Some(task) = state.tasks.get_mut(&handle) {
                    if task.phase == ZrRuntimeOperationPhase::Queued
                        && task.snapshot_claimed
                        && task.deadline_armed
                        && !task.apply_claimed
                        && task
                            .deadline
                            .is_none_or(|deadline| deadline > Instant::now())
                    {
                        task.phase = ZrRuntimeOperationPhase::Preparing;
                        task.detail_kind = ZrRuntimeOperationDetailKindV2::None;
                        task.detail_value = 0;
                        task.snapshot_claimed = false;
                        task.prepare_in_flight = true;
                        state.in_flight_prepares = state
                            .in_flight_prepares
                            .checked_add(1)
                            .expect("operation prepare capacity was preflighted");
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            };
            if !should_dispatch {
                continue;
            }
            let batch = completion_batch.get_or_insert_with(|| {
                let (sender, receiver) = mpsc::sync_channel(self.limits.max_in_flight_prepares);
                RuntimeOperationCompletionBatch {
                    sender,
                    receiver,
                    handles: Vec::new(),
                }
            });
            batch.handles.push(handle);
            let completion_sender = batch.sender.clone();
            core.scheduler().spawn(move || {
                let completion = match catch_unwind(AssertUnwindSafe(|| handler.prepare(snapshot)))
                {
                    Ok(Ok(prepared)) => {
                        let (command, result) = prepared.into_parts();
                        match (json_value_byte_len(&command), json_value_byte_len(&result)) {
                            (Ok(command_bytes), Ok(result_bytes)) => {
                                RuntimeOperationPrepareCompletion::Prepared {
                                    handle,
                                    command,
                                    result,
                                    command_bytes,
                                    result_bytes,
                                }
                            }
                            (Err(error), _) | (_, Err(error)) => {
                                RuntimeOperationPrepareCompletion::Failed {
                                    handle,
                                    error: error.to_string(),
                                    detail_kind: ZrRuntimeOperationDetailKindV2::None,
                                }
                            }
                        }
                    }
                    Ok(Err(error)) => RuntimeOperationPrepareCompletion::Failed {
                        handle,
                        error: error.to_string(),
                        detail_kind: ZrRuntimeOperationDetailKindV2::None,
                    },
                    Err(_) => RuntimeOperationPrepareCompletion::Failed {
                        handle,
                        error: "runtime operation prepare panicked".to_owned(),
                        detail_kind: ZrRuntimeOperationDetailKindV2::WorkerPanic,
                    },
                };
                let _ = completion_sender.send(completion);
            });
        }
        if let Some(RuntimeOperationCompletionBatch {
            sender,
            receiver,
            handles,
        }) = completion_batch
        {
            drop(sender);
            self.lock_completion_receivers()
                .push(RuntimeOperationCompletionReceiver { receiver, handles });
        }
    }

    fn take_queued_snapshot_task(
        &self,
    ) -> Option<(
        ZrRuntimeOperationHandle,
        Arc<dyn RuntimeOperationHandler>,
        serde_json::Value,
    )> {
        let mut state = self.lock_state();
        if state.in_flight_prepares >= self.limits.max_in_flight_prepares {
            return None;
        }
        let handle = state.tasks.iter().find_map(|(handle, task)| {
            (task.phase == ZrRuntimeOperationPhase::Queued
                && !task.snapshot_claimed
                && task.deadline_armed
                && task
                    .deadline
                    .is_none_or(|deadline| deadline > Instant::now()))
            .then_some(*handle)
        })?;
        let (handler, payload) = {
            let task = state.tasks.get_mut(&handle)?;
            task.snapshot_claimed = true;
            (Arc::clone(&task.handler), task.payload.take())
        };
        let Some(payload) = payload else {
            self.finish_failed_task(
                &mut state,
                handle,
                "queued runtime operation lost its payload",
                ZrRuntimeOperationDetailKindV2::None,
                0,
            );
            return None;
        };
        Some((handle, handler, payload))
    }

    fn finish_snapshot_failed_task(&self, handle: ZrRuntimeOperationHandle, error: String) {
        let mut state = self.lock_state();
        let should_finish = state.tasks.get(&handle).is_some_and(|task| {
            task.phase == ZrRuntimeOperationPhase::Queued
                && task.snapshot_claimed
                && !task.apply_claimed
        });
        if should_finish {
            self.finish_failed_task(
                &mut state,
                handle,
                error,
                ZrRuntimeOperationDetailKindV2::None,
                0,
            );
        }
        drop(state);
        if should_finish {
            self.refresh_maintenance_after_transition();
        }
    }

    fn apply_prepared(&self, core: &CoreHandle, world: &mut World) {
        let mut terminal_transition = false;
        for _ in 0..self.limits.max_owner_applies_per_tick {
            let Some((handle, handler, command)) = self.take_prepared_task() else {
                break;
            };
            let result = catch_unwind(AssertUnwindSafe(|| {
                handler.apply(RuntimeOperationContext::new(core, world), command)
            }));
            let mut state = self.lock_state();
            match result {
                Ok(Ok(())) => {
                    if let Err(error) = self.finish_completed_task(&mut state, handle) {
                        self.finish_failed_task(
                            &mut state,
                            handle,
                            error.to_string(),
                            ZrRuntimeOperationDetailKindV2::AdmissionByteLimit,
                            self.limits.max_retained_bytes as u64,
                        );
                    }
                }
                Ok(Err(error)) => self.finish_failed_task(
                    &mut state,
                    handle,
                    error.to_string(),
                    ZrRuntimeOperationDetailKindV2::OwnerApplyFailed,
                    0,
                ),
                Err(_) => self.finish_failed_task(
                    &mut state,
                    handle,
                    "runtime operation owner apply panicked",
                    ZrRuntimeOperationDetailKindV2::OwnerApplyFailed,
                    0,
                ),
            }
            terminal_transition = true;
        }
        if terminal_transition {
            self.refresh_maintenance_after_transition();
        }
    }

    fn take_prepared_task(
        &self,
    ) -> Option<(
        ZrRuntimeOperationHandle,
        Arc<dyn RuntimeOperationHandler>,
        serde_json::Value,
    )> {
        let mut state = self.lock_state();
        let now = Instant::now();
        let handle = state.tasks.iter().find_map(|(handle, task)| {
            (task.phase == ZrRuntimeOperationPhase::ReadyToApply
                && task.prepared_command.is_some()
                && task.prepared_result.is_some()
                && !task.apply_claimed
                && task.deadline_armed
                && task.deadline.is_none_or(|deadline| deadline > now))
            .then_some(*handle)
        })?;
        let task = state.tasks.get_mut(&handle)?;
        task.apply_claimed = true;
        Some((
            handle,
            Arc::clone(&task.handler),
            task.prepared_command.take()?,
        ))
    }

    fn finish_completed_task(
        &self,
        state: &mut RuntimeOperationTaskState,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<(), RuntimeOperationServiceError> {
        let (operation_id, command_bytes, result_bytes) = {
            let task = state
                .tasks
                .get(&handle)
                .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })?;
            (
                task.operation_id.clone(),
                task.prepared_command_bytes,
                task.prepared_result_bytes,
            )
        };
        let retained_bytes = state
            .retained_bytes
            .checked_sub(command_bytes)
            .expect("prepared command bytes must remain reserved before owner apply");
        let task = state
            .tasks
            .get_mut(&handle)
            .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })?;
        let result = task
            .prepared_result
            .take()
            .ok_or(RuntimeOperationServiceError::NotTerminal { handle })?;
        task.prepared_command = None;
        task.prepared_command_bytes = 0;
        task.prepared_result_bytes = 0;
        task.phase = ZrRuntimeOperationPhase::Completed;
        task.detail_kind = ZrRuntimeOperationDetailKindV2::None;
        task.detail_value = 0;
        task.retained_bytes = result_bytes;
        task.deadline_armed = false;
        task.terminal_at = Some(Instant::now());
        task.apply_claimed = false;
        task.result = Some(ZrRuntimeOperationResultV1::succeeded(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            handle,
            operation_id,
            result,
        ));
        state.retained_bytes = retained_bytes;
        Ok(())
    }

    fn finish_failed_task(
        &self,
        state: &mut RuntimeOperationTaskState,
        handle: ZrRuntimeOperationHandle,
        error: impl Into<String>,
        detail_kind: ZrRuntimeOperationDetailKindV2,
        detail_value: u64,
    ) {
        let error = error.into();
        let Some((previous_bytes, operation_id)) = state
            .tasks
            .get(&handle)
            .map(|task| (task.retained_bytes, task.operation_id.clone()))
        else {
            return;
        };
        let retained_without_task = state
            .retained_bytes
            .checked_sub(previous_bytes)
            .expect("failed operation bytes must remain accounted");
        let error = truncate_utf8_to_bytes(
            error,
            self.limits
                .max_retained_bytes
                .checked_sub(retained_without_task)
                .expect("other retained operation bytes must remain within capacity"),
        );
        let error_bytes = error.len();
        state.retained_bytes = retained_without_task
            .checked_add(error_bytes)
            .expect("failed operation error bytes must fit retained capacity");
        let Some(task) = state.tasks.get_mut(&handle) else {
            return;
        };
        task.phase = ZrRuntimeOperationPhase::Failed;
        task.detail_kind = detail_kind;
        task.detail_value = detail_value;
        task.payload = None;
        task.prepared_command = None;
        task.prepared_result = None;
        task.prepared_command_bytes = 0;
        task.prepared_result_bytes = 0;
        task.retained_bytes = error_bytes;
        task.deadline_armed = false;
        task.result = Some(ZrRuntimeOperationResultV1::failed(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            handle,
            operation_id,
            error,
        ));
        task.terminal_at = Some(Instant::now());
        task.snapshot_claimed = false;
        task.apply_claimed = false;
    }

    fn lock_state(&self) -> MutexGuard<'_, RuntimeOperationTaskState> {
        lock_operation_state(&self.state)
    }

    fn expire_due_deadlines(&self, now: Instant) {
        let mut state = self.lock_state();
        expire_due_deadlines_in_state(&mut state, now);
    }

    fn expire_terminal_results(&self, now: Instant) {
        let mut state = self.lock_state();
        expire_terminal_results_in_state(&mut state, self.limits.terminal_result_ttl, now);
    }

    fn refresh_maintenance(&self) -> Result<(), RuntimeOperationServiceError> {
        refresh_operation_maintenance_alarm(
            &self.state,
            &self.maintenance_refresh,
            self.limits.terminal_result_ttl,
        )
        .map_err(|_| RuntimeOperationServiceError::DeadlineTimerUnavailable)
    }

    fn refresh_maintenance_after_transition(&self) {
        let _ = self.refresh_maintenance();
    }

    fn lock_completion_receivers(&self) -> MutexGuard<'_, Vec<RuntimeOperationCompletionReceiver>> {
        self.completion_receivers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for RuntimeOperationService {
    fn default() -> Self {
        Self::new()
    }
}

fn consume_raw_admission(
    state: &mut RuntimeOperationTaskState,
    reservation: RuntimeOperationAdmissionReservation,
) {
    state.pending_admissions = state
        .pending_admissions
        .checked_sub(1)
        .expect("raw operation admission count must be released exactly once");
    state.pending_admission_bytes = state
        .pending_admission_bytes
        .checked_sub(reservation.bytes)
        .expect("raw operation admission bytes must be released exactly once");
}

struct JsonByteCounter(usize);

impl Write for JsonByteCounter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self.0.saturating_add(bytes.len());
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn json_value_byte_len(value: &serde_json::Value) -> Result<usize, RuntimeOperationServiceError> {
    let mut counter = JsonByteCounter(0);
    serde_json::to_writer(&mut counter, value).map_err(|error| {
        RuntimeOperationServiceError::PayloadEncoding {
            message: error.to_string(),
        }
    })?;
    Ok(counter.0)
}

fn truncate_utf8_to_bytes(message: String, maximum: usize) -> String {
    if message.len() <= maximum {
        return message;
    }
    let mut end = maximum;
    while end > 0 && !message.is_char_boundary(end) {
        end -= 1;
    }
    message[..end].to_owned()
}
