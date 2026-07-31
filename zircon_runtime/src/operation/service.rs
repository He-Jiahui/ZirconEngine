use std::collections::{hash_map::Entry, BTreeMap, HashMap};
use std::io::{self, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationPhase, ZrRuntimeOperationProgressV1,
    ZrRuntimeOperationResultV1, ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use crate::core::{CoreHandle, JobScheduler};
use crate::scene::World;

use super::task::RuntimeOperationTask;
use super::{RuntimeOperationContext, RuntimeOperationHandler, RuntimeOperationServiceError};

const DEFAULT_MAX_TASKS: usize = 1_024;
const DEFAULT_MAX_IN_FLIGHT_PREPARES: usize = 32;
const DEFAULT_MAX_RETAINED_BYTES: usize = 4 * 1024 * 1024;
const DEFAULT_MAX_OWNER_APPLIES_PER_TICK: usize = 8;

#[derive(Clone, Copy, Debug)]
pub(super) struct RuntimeOperationLimits {
    pub(super) max_tasks: usize,
    pub(super) max_in_flight_prepares: usize,
    pub(super) max_retained_bytes: usize,
    pub(super) max_owner_applies_per_tick: usize,
}

impl Default for RuntimeOperationLimits {
    fn default() -> Self {
        Self {
            max_tasks: DEFAULT_MAX_TASKS,
            max_in_flight_prepares: DEFAULT_MAX_IN_FLIGHT_PREPARES,
            max_retained_bytes: DEFAULT_MAX_RETAINED_BYTES,
            max_owner_applies_per_tick: DEFAULT_MAX_OWNER_APPLIES_PER_TICK,
        }
    }
}

#[derive(Default)]
struct RuntimeOperationTaskState {
    next_handle: u64,
    tasks: HashMap<ZrRuntimeOperationHandle, RuntimeOperationTask>,
    retained_bytes: usize,
    in_flight_prepares: usize,
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
        prepared: serde_json::Value,
        prepared_bytes: usize,
    },
    Failed {
        handle: ZrRuntimeOperationHandle,
        error: String,
    },
}

/// Registry, bounded task store, and prepare completion queue for one runtime session.
pub struct RuntimeOperationService {
    handlers: BTreeMap<String, Arc<dyn RuntimeOperationHandler>>,
    limits: RuntimeOperationLimits,
    state: Mutex<RuntimeOperationTaskState>,
    completion_sender: SyncSender<RuntimeOperationPrepareCompletion>,
    completion_receiver: Mutex<Receiver<RuntimeOperationPrepareCompletion>>,
}

impl RuntimeOperationService {
    pub fn new() -> Self {
        Self::with_limits(RuntimeOperationLimits::default())
    }

    pub(super) fn with_limits(limits: RuntimeOperationLimits) -> Self {
        let (completion_sender, completion_receiver) =
            mpsc::sync_channel(limits.max_in_flight_prepares);
        Self {
            handlers: BTreeMap::new(),
            limits,
            state: Mutex::new(RuntimeOperationTaskState::default()),
            completion_sender,
            completion_receiver: Mutex::new(completion_receiver),
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

    pub fn submit(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, RuntimeOperationServiceError> {
        if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return Err(RuntimeOperationServiceError::UnsupportedAbiVersion {
                actual: request.abi_version,
            });
        }
        if request.operation_id.trim().is_empty() {
            return Err(RuntimeOperationServiceError::EmptyOperationId);
        }
        let handler = self
            .handlers
            .get(&request.operation_id)
            .cloned()
            .ok_or_else(|| RuntimeOperationServiceError::UnknownOperation {
                operation_id: request.operation_id.clone(),
            })?;
        let payload_bytes = json_value_byte_len(&request.payload)?;
        let mut state = self.lock_state();
        if state.tasks.len() >= self.limits.max_tasks {
            return Err(RuntimeOperationServiceError::TaskCapacityReached {
                maximum: self.limits.max_tasks,
            });
        }
        let retained_bytes = state.retained_bytes.checked_add(payload_bytes).ok_or(
            RuntimeOperationServiceError::RetainedBytesCapacityReached {
                maximum: self.limits.max_retained_bytes,
            },
        )?;
        if retained_bytes > self.limits.max_retained_bytes {
            return Err(RuntimeOperationServiceError::RetainedBytesCapacityReached {
                maximum: self.limits.max_retained_bytes,
            });
        }
        let handle = state.allocate_handle()?;
        state.tasks.insert(
            handle,
            RuntimeOperationTask {
                handle,
                operation_id: request.operation_id,
                phase: ZrRuntimeOperationPhase::Queued,
                handler,
                payload: Some(request.payload),
                prepared: None,
                retained_bytes: payload_bytes,
                result: None,
            },
        );
        state.retained_bytes = retained_bytes;
        Ok(handle)
    }

    /// Upper bound for the raw V1 request accepted by this session.
    pub fn max_retained_bytes(&self) -> usize {
        self.limits.max_retained_bytes
    }

    /// Returns a snapshot only. Runtime work is dispatched exclusively by `tick`.
    pub fn poll(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationProgressV1, RuntimeOperationServiceError> {
        let state = self.lock_state();
        state
            .tasks
            .get(&handle)
            .map(RuntimeOperationTask::progress)
            .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })
    }

    /// Advances bounded prepare work and applies at most one owner budget per runtime frame.
    pub fn tick(&self, core: &CoreHandle, world: &mut World) {
        self.drain_prepare_completions();
        self.apply_prepared(core, world);
        self.dispatch_queued_prepares(core.scheduler());
    }

    pub fn harvest(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, RuntimeOperationServiceError> {
        let mut state = self.lock_state();
        let task = match state.tasks.entry(handle) {
            Entry::Occupied(entry) => {
                if !entry.get().phase.is_terminal() {
                    return Err(RuntimeOperationServiceError::NotTerminal { handle });
                }
                entry.remove()
            }
            Entry::Vacant(_) => return Err(RuntimeOperationServiceError::UnknownHandle { handle }),
        };
        state.retained_bytes = state.retained_bytes.saturating_sub(task.retained_bytes);
        task.result
            .ok_or(RuntimeOperationServiceError::NotTerminal { handle })
    }

    fn dispatch_queued_prepares(&self, scheduler: &JobScheduler) {
        loop {
            let prepare = {
                let mut state = self.lock_state();
                if state.in_flight_prepares >= self.limits.max_in_flight_prepares {
                    return;
                }
                let Some(handle) = state.tasks.iter().find_map(|(handle, task)| {
                    (task.phase == ZrRuntimeOperationPhase::Queued).then_some(*handle)
                }) else {
                    return;
                };
                let (handler, payload) = {
                    let Some(task) = state.tasks.get_mut(&handle) else {
                        continue;
                    };
                    let Some(payload) = task.payload.take() else {
                        self.finish_failed_task(
                            &mut state,
                            handle,
                            "queued runtime operation lost its payload",
                        );
                        continue;
                    };
                    task.phase = ZrRuntimeOperationPhase::Running;
                    (Arc::clone(&task.handler), payload)
                };
                state.in_flight_prepares += 1;
                (handle, handler, payload)
            };
            let completion_sender = self.completion_sender.clone();
            scheduler.spawn(move || {
                let (handle, handler, payload) = prepare;
                let completion = match catch_unwind(AssertUnwindSafe(|| handler.prepare(payload))) {
                    Ok(Ok(prepared)) => match json_value_byte_len(&prepared) {
                        Ok(prepared_bytes) => RuntimeOperationPrepareCompletion::Prepared {
                            handle,
                            prepared,
                            prepared_bytes,
                        },
                        Err(error) => RuntimeOperationPrepareCompletion::Failed {
                            handle,
                            error: error.to_string(),
                        },
                    },
                    Ok(Err(error)) => RuntimeOperationPrepareCompletion::Failed {
                        handle,
                        error: error.to_string(),
                    },
                    Err(_) => RuntimeOperationPrepareCompletion::Failed {
                        handle,
                        error: "runtime operation prepare panicked".to_owned(),
                    },
                };
                let _ = completion_sender.send(completion);
            });
        }
    }

    fn drain_prepare_completions(&self) {
        loop {
            let completion = match self.lock_completion_receiver().try_recv() {
                Ok(completion) => completion,
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return,
            };
            let mut state = self.lock_state();
            state.in_flight_prepares = state.in_flight_prepares.saturating_sub(1);
            match completion {
                RuntimeOperationPrepareCompletion::Prepared {
                    handle,
                    prepared,
                    prepared_bytes,
                } => {
                    let Some(previous_bytes) = state.tasks.get(&handle).and_then(|task| {
                        (task.phase == ZrRuntimeOperationPhase::Running)
                            .then_some(task.retained_bytes)
                    }) else {
                        continue;
                    };
                    let Some(retained_bytes) = state
                        .retained_bytes
                        .saturating_sub(previous_bytes)
                        .checked_add(prepared_bytes)
                    else {
                        self.finish_failed_task(
                            &mut state,
                            handle,
                            "runtime operation prepared payload exceeds retained byte budget",
                        );
                        continue;
                    };
                    if retained_bytes > self.limits.max_retained_bytes {
                        self.finish_failed_task(
                            &mut state,
                            handle,
                            "runtime operation prepared payload exceeds retained byte budget",
                        );
                        continue;
                    }
                    state.retained_bytes = retained_bytes;
                    let Some(task) = state.tasks.get_mut(&handle) else {
                        continue;
                    };
                    task.prepared = Some(prepared);
                    task.retained_bytes = prepared_bytes;
                }
                RuntimeOperationPrepareCompletion::Failed { handle, error } => {
                    self.finish_failed_task(&mut state, handle, error);
                }
            }
        }
    }

    fn apply_prepared(&self, core: &CoreHandle, world: &mut World) {
        for _ in 0..self.limits.max_owner_applies_per_tick {
            let Some((handle, operation_id, handler, prepared)) = self.take_prepared_task() else {
                return;
            };
            let result = catch_unwind(AssertUnwindSafe(|| {
                handler.apply(RuntimeOperationContext::new(core, world), prepared)
            }));
            let mut state = self.lock_state();
            match result {
                Ok(Ok(output)) => {
                    if let Err(error) =
                        self.finish_completed_task(&mut state, handle, operation_id, output)
                    {
                        self.finish_failed_task(&mut state, handle, error.to_string());
                    }
                }
                Ok(Err(error)) => self.finish_failed_task(&mut state, handle, error.to_string()),
                Err(_) => self.finish_failed_task(
                    &mut state,
                    handle,
                    "runtime operation owner apply panicked",
                ),
            }
        }
    }

    fn take_prepared_task(
        &self,
    ) -> Option<(
        ZrRuntimeOperationHandle,
        String,
        Arc<dyn RuntimeOperationHandler>,
        serde_json::Value,
    )> {
        let mut state = self.lock_state();
        let handle = state.tasks.iter().find_map(|(handle, task)| {
            (task.phase == ZrRuntimeOperationPhase::Running && task.prepared.is_some())
                .then_some(*handle)
        })?;
        let task = state.tasks.get_mut(&handle)?;
        Some((
            handle,
            task.operation_id.clone(),
            Arc::clone(&task.handler),
            task.prepared.take()?,
        ))
    }

    fn finish_completed_task(
        &self,
        state: &mut RuntimeOperationTaskState,
        handle: ZrRuntimeOperationHandle,
        operation_id: String,
        output: serde_json::Value,
    ) -> Result<(), RuntimeOperationServiceError> {
        let output_bytes = json_value_byte_len(&output)?;
        let previous_bytes = state
            .tasks
            .get(&handle)
            .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })?
            .retained_bytes;
        let retained_bytes = state
            .retained_bytes
            .saturating_sub(previous_bytes)
            .checked_add(output_bytes)
            .ok_or(RuntimeOperationServiceError::RetainedBytesCapacityReached {
                maximum: self.limits.max_retained_bytes,
            })?;
        if retained_bytes > self.limits.max_retained_bytes {
            return Err(RuntimeOperationServiceError::RetainedBytesCapacityReached {
                maximum: self.limits.max_retained_bytes,
            });
        }
        state.retained_bytes = retained_bytes;
        let task = state
            .tasks
            .get_mut(&handle)
            .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })?;
        task.phase = ZrRuntimeOperationPhase::Completed;
        task.retained_bytes = output_bytes;
        task.result = Some(ZrRuntimeOperationResultV1::succeeded(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            handle,
            operation_id,
            output,
        ));
        Ok(())
    }

    fn finish_failed_task(
        &self,
        state: &mut RuntimeOperationTaskState,
        handle: ZrRuntimeOperationHandle,
        error: impl Into<String>,
    ) {
        let error = error.into();
        let Some((previous_bytes, operation_id)) = state
            .tasks
            .get(&handle)
            .map(|task| (task.retained_bytes, task.operation_id.clone()))
        else {
            return;
        };
        let retained_without_task = state.retained_bytes.saturating_sub(previous_bytes);
        let error = truncate_utf8_to_bytes(
            error,
            self.limits
                .max_retained_bytes
                .saturating_sub(retained_without_task),
        );
        let error_bytes = error.len();
        state.retained_bytes = retained_without_task.saturating_add(error_bytes);
        let Some(task) = state.tasks.get_mut(&handle) else {
            return;
        };
        task.phase = ZrRuntimeOperationPhase::Failed;
        task.payload = None;
        task.prepared = None;
        task.retained_bytes = error_bytes;
        task.result = Some(ZrRuntimeOperationResultV1::failed(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            handle,
            operation_id,
            error,
        ));
    }

    fn lock_state(&self) -> MutexGuard<'_, RuntimeOperationTaskState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_completion_receiver(
        &self,
    ) -> MutexGuard<'_, Receiver<RuntimeOperationPrepareCompletion>> {
        self.completion_receiver
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for RuntimeOperationService {
    fn default() -> Self {
        Self::new()
    }
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
