use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationPhase, ZrRuntimeOperationProgressV1,
    ZrRuntimeOperationResultV1, ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::task::RuntimeOperationTask;
use super::{RuntimeOperationContext, RuntimeOperationHandler, RuntimeOperationServiceError};

#[derive(Default)]
struct RuntimeOperationTaskState {
    next_handle: u64,
    tasks: HashMap<ZrRuntimeOperationHandle, RuntimeOperationTask>,
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

/// Registry and task store shared by one runtime session.
pub struct RuntimeOperationService {
    handlers: BTreeMap<String, Arc<dyn RuntimeOperationHandler>>,
    state: Mutex<RuntimeOperationTaskState>,
}

impl RuntimeOperationService {
    pub fn new() -> Self {
        Self {
            handlers: BTreeMap::new(),
            state: Mutex::new(RuntimeOperationTaskState::default()),
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
        let mut state = self.lock_state();
        let handle = state.allocate_handle()?;
        state.tasks.insert(
            handle,
            RuntimeOperationTask {
                handle,
                operation_id: request.operation_id,
                phase: ZrRuntimeOperationPhase::Queued,
                handler,
                payload: Some(request.payload),
                execution_started: false,
                result: None,
            },
        );
        Ok(handle)
    }

    pub fn poll(
        &self,
        context: RuntimeOperationContext<'_>,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationProgressV1, RuntimeOperationServiceError> {
        let execution = {
            let mut state = self.lock_state();
            let task = state
                .tasks
                .get_mut(&handle)
                .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })?;
            match task.phase {
                ZrRuntimeOperationPhase::Queued => {
                    task.phase = ZrRuntimeOperationPhase::Running;
                    return Ok(task.progress());
                }
                ZrRuntimeOperationPhase::Running if !task.execution_started => {
                    task.execution_started = true;
                    task.payload
                        .take()
                        .map(|payload| (task.operation_id.clone(), task.handler.clone(), payload))
                }
                ZrRuntimeOperationPhase::Running
                | ZrRuntimeOperationPhase::Completed
                | ZrRuntimeOperationPhase::Failed => None,
            }
        };

        let Some((operation_id, handler, payload)) = execution else {
            let state = self.lock_state();
            let task = state
                .tasks
                .get(&handle)
                .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })?;
            return Ok(task.progress());
        };
        let result = handler.execute(context, payload);
        let mut state = self.lock_state();
        let task = state
            .tasks
            .get_mut(&handle)
            .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })?;
        match result {
            Ok(output) => {
                task.phase = ZrRuntimeOperationPhase::Completed;
                task.result = Some(ZrRuntimeOperationResultV1::succeeded(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    handle,
                    operation_id,
                    output,
                ));
            }
            Err(error) => {
                task.phase = ZrRuntimeOperationPhase::Failed;
                task.result = Some(ZrRuntimeOperationResultV1::failed(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    handle,
                    operation_id,
                    error.to_string(),
                ));
            }
        }
        Ok(task.progress())
    }

    pub fn harvest(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, RuntimeOperationServiceError> {
        let mut state = self.lock_state();
        let task = state
            .tasks
            .get(&handle)
            .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })?;
        if !task.phase.is_terminal() {
            return Err(RuntimeOperationServiceError::NotTerminal { handle });
        }
        let task = state
            .tasks
            .remove(&handle)
            .ok_or(RuntimeOperationServiceError::UnknownHandle { handle })?;
        task.result
            .ok_or(RuntimeOperationServiceError::NotTerminal { handle })
    }

    fn lock_state(&self) -> MutexGuard<'_, RuntimeOperationTaskState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for RuntimeOperationService {
    fn default() -> Self {
        Self::new()
    }
}
