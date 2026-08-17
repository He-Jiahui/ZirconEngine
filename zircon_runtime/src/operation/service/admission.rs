use std::sync::Arc;
use std::time::Instant;

use zircon_runtime_interface::{
    ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle, ZrRuntimeOperationPhase,
    ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::{
    consume_raw_admission, json_value_byte_len, RuntimeOperationAdmissionReservation,
    RuntimeOperationService, RuntimeOperationServiceError, RuntimeOperationTask,
    RuntimeOperationTaskState,
};

impl RuntimeOperationService {
    pub fn submit(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, RuntimeOperationServiceError> {
        self.submit_with_deadline(request, None)
    }

    /// Reserves bounded admission before decoding an FFI-owned JSON request.
    pub fn submit_json(
        &self,
        request_json: &[u8],
    ) -> Result<ZrRuntimeOperationHandle, RuntimeOperationServiceError> {
        let reservation = self.reserve_raw_admission(request_json.len())?;
        let request = match serde_json::from_slice(request_json) {
            Ok(request) => request,
            Err(_) => {
                self.release_raw_admission(reservation);
                return Err(RuntimeOperationServiceError::InvalidRequest);
            }
        };
        let result = self.submit_with_reservation(request, None, Some(reservation));
        if result.is_err() {
            self.release_raw_admission(reservation);
        }
        result
    }

    /// Admits one operation with an owner-tick-enforced deadline.
    pub fn submit_with_deadline(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
        deadline: Option<Instant>,
    ) -> Result<ZrRuntimeOperationHandle, RuntimeOperationServiceError> {
        self.submit_with_reservation(request, deadline, None)
    }

    fn submit_with_reservation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
        deadline: Option<Instant>,
        reservation: Option<RuntimeOperationAdmissionReservation>,
    ) -> Result<ZrRuntimeOperationHandle, RuntimeOperationServiceError> {
        if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return Err(RuntimeOperationServiceError::UnsupportedAbiVersion {
                actual: request.abi_version,
            });
        }
        if request.operation_id.trim().is_empty() {
            return Err(RuntimeOperationServiceError::EmptyOperationId);
        }
        let (operation_id, handler) = self
            .handlers
            .get_key_value(&request.operation_id)
            .map(|(operation_id, handler)| (operation_id.clone(), Arc::clone(handler)))
            .ok_or_else(|| RuntimeOperationServiceError::UnknownOperation {
                operation_id: request.operation_id.clone(),
            })?;
        let payload_bytes = json_value_byte_len(&request.payload)?;
        let mut state = self.lock_state();
        let additional_slots = usize::from(reservation.is_none());
        self.evict_tombstones_until_admissible(&mut state, additional_slots);
        let pending_admissions = state.pending_admissions;
        let task_count = state
            .tasks
            .len()
            .checked_add(pending_admissions)
            .and_then(|count| count.checked_add(additional_slots))
            .unwrap_or(usize::MAX);
        if task_count > self.limits.max_tasks {
            return Err(RuntimeOperationServiceError::TaskCapacityReached {
                maximum: self.limits.max_tasks,
            });
        }
        let reserved_bytes = reservation.map_or(0, |reservation| reservation.bytes);
        let pending_bytes = state
            .pending_admission_bytes
            .checked_sub(reserved_bytes)
            .expect("raw operation admission reservation must remain accounted");
        let retained_bytes = state
            .retained_bytes
            .checked_add(pending_bytes)
            .and_then(|bytes| bytes.checked_add(payload_bytes))
            .ok_or(RuntimeOperationServiceError::RetainedBytesCapacityReached {
                maximum: self.limits.max_retained_bytes,
            })?;
        if retained_bytes > self.limits.max_retained_bytes {
            return Err(RuntimeOperationServiceError::RetainedBytesCapacityReached {
                maximum: self.limits.max_retained_bytes,
            });
        }
        let task_retained_bytes = state
            .retained_bytes
            .checked_add(payload_bytes)
            .expect("operation admission preflighted retained bytes");
        let handle = state.allocate_handle()?;
        if let Some(reservation) = reservation {
            consume_raw_admission(&mut state, reservation);
        }
        let queue_depth = u64::try_from(state.tasks.len().saturating_add(1)).unwrap_or(u64::MAX);
        state.tasks.insert(
            handle,
            RuntimeOperationTask {
                handle,
                operation_id,
                phase: ZrRuntimeOperationPhase::Queued,
                detail_kind: ZrRuntimeOperationDetailKindV2::QueueDepth,
                detail_value: queue_depth,
                handler,
                payload: Some(request.payload),
                prepared_command: None,
                prepared_result: None,
                prepared_command_bytes: 0,
                prepared_result_bytes: 0,
                retained_bytes: payload_bytes,
                result: None,
                deadline,
                deadline_armed: deadline.is_none(),
                terminal_at: None,
                harvest_in_flight: false,
                snapshot_claimed: false,
                prepare_in_flight: false,
                apply_claimed: false,
            },
        );
        state.retained_bytes = task_retained_bytes;
        drop(state);
        if let Some(deadline) = deadline {
            self.arm_deadline(handle, deadline)?;
        }
        Ok(handle)
    }

    fn reserve_raw_admission(
        &self,
        request_bytes: usize,
    ) -> Result<RuntimeOperationAdmissionReservation, RuntimeOperationServiceError> {
        let mut state = self.lock_state();
        self.evict_tombstones_until_admissible(&mut state, 1);
        let task_count = state
            .tasks
            .len()
            .checked_add(state.pending_admissions)
            .and_then(|count| count.checked_add(1))
            .unwrap_or(usize::MAX);
        if task_count > self.limits.max_tasks {
            return Err(RuntimeOperationServiceError::TaskCapacityReached {
                maximum: self.limits.max_tasks,
            });
        }
        let reserved_bytes = state
            .retained_bytes
            .checked_add(state.pending_admission_bytes)
            .and_then(|bytes| bytes.checked_add(request_bytes))
            .ok_or(RuntimeOperationServiceError::RetainedBytesCapacityReached {
                maximum: self.limits.max_retained_bytes,
            })?;
        if reserved_bytes > self.limits.max_retained_bytes {
            return Err(RuntimeOperationServiceError::RetainedBytesCapacityReached {
                maximum: self.limits.max_retained_bytes,
            });
        }
        state.pending_admissions = state
            .pending_admissions
            .checked_add(1)
            .expect("raw operation admission count was preflighted");
        state.pending_admission_bytes = state
            .pending_admission_bytes
            .checked_add(request_bytes)
            .expect("raw operation admission bytes were preflighted");
        Ok(RuntimeOperationAdmissionReservation {
            bytes: request_bytes,
        })
    }

    fn release_raw_admission(&self, reservation: RuntimeOperationAdmissionReservation) {
        let mut state = self.lock_state();
        consume_raw_admission(&mut state, reservation);
    }

    fn evict_tombstones_until_admissible(
        &self,
        state: &mut RuntimeOperationTaskState,
        additional_slots: usize,
    ) {
        while state
            .tasks
            .len()
            .saturating_add(state.pending_admissions)
            .saturating_add(additional_slots)
            > self.limits.max_tasks
        {
            let Some(handle) = state
                .tasks
                .iter()
                .filter(|(_, task)| {
                    matches!(
                        task.phase,
                        ZrRuntimeOperationPhase::Cancelled
                            | ZrRuntimeOperationPhase::Expired
                            | ZrRuntimeOperationPhase::Harvested
                    )
                })
                .min_by_key(|(handle, task)| (task.terminal_at, handle.raw()))
                .map(|(handle, _)| *handle)
            else {
                return;
            };
            state.tasks.remove(&handle);
        }
    }
}
