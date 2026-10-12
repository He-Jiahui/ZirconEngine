use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::asset::project::{ResolvedProjectPath, ResolvedProjectPathIdentity};
use crate::core::runtime::{
    BoundedKeyedIoAdmissionError, BoundedKeyedIoCancelAuthority, BoundedKeyedIoFailure,
    BoundedKeyedIoKey, BoundedKeyedIoLane, BoundedKeyedIoLimits, BoundedKeyedIoShutdownGuard,
    BoundedKeyedIoTicket, BoundedKeyedIoWorkDeadline, JobScheduler, RetainedByteBudget,
};
use crate::core::{CoreHandle, CoreWeak};

use super::super::load_save::load_from_path_with_limit;
use super::contract::{
    RuntimeSessionArchiveReadArtifact, RuntimeSessionArchiveReadOutcome,
    RuntimeSessionArchiveReadSubmission, RuntimeSessionArchiveReaderDiagnostics,
    RuntimeSessionArchiveReaderLimits, RuntimeSessionArchiveReaderSubmitError,
};

const ARCHIVE_READER_ENTRY_METADATA_BYTES: usize = 64 * 1024;
const ARCHIVE_READER_METADATA_BYTES: usize = 256;
const ARCHIVE_READ_FAILURE: &str = "runtime_session_archive_read_failed";

/// 由 Runtime 任务图驱动的有界会话读取服务；同一规范路径的未完成请求共享票据与结果预算。
/// 读取结果的内存保留期由返回的租约决定，关闭服务不强行撤销已交付的结果。
pub struct RuntimeSessionArchiveReader {
    lane: BoundedKeyedIoLane,
    result_budget: RetainedByteBudget,
    limits: RuntimeSessionArchiveReaderLimits,
    state: Arc<Mutex<RuntimeSessionArchiveReaderState>>,
    owner: RuntimeSessionArchiveReaderOwner,
}

struct RuntimeSessionArchiveReaderState {
    accepting: bool,
    next_generation: u64,
    requests: BTreeMap<ResolvedProjectPathIdentity, Weak<RuntimeSessionArchiveReadRequest>>,
}

enum RuntimeSessionArchiveReaderOwner {
    Runtime(CoreWeak),
    #[cfg(test)]
    Fixture,
}

pub(super) struct RuntimeSessionArchiveReadRequest {
    pub(super) ticket: BoundedKeyedIoTicket,
    pub(super) cancel_authority: BoundedKeyedIoCancelAuthority,
    outcome: Arc<Mutex<Option<RuntimeSessionArchiveReadOutcome>>>,
    reader_state: Weak<Mutex<RuntimeSessionArchiveReaderState>>,
    path_identity: ResolvedProjectPathIdentity,
}

impl RuntimeSessionArchiveReader {
    pub fn with_runtime(limits: RuntimeSessionArchiveReaderLimits, runtime: &CoreHandle) -> Self {
        let scheduler = runtime
            .task_graph()
            .scheduler(crate::core::TaskPoolKind::Compute);
        Self::with_owner(
            limits,
            scheduler,
            RuntimeSessionArchiveReaderOwner::Runtime(runtime.downgrade()),
        )
    }

    #[cfg(test)]
    pub(crate) fn with_scheduler(
        limits: RuntimeSessionArchiveReaderLimits,
        scheduler: JobScheduler,
    ) -> Self {
        Self::with_owner(limits, scheduler, RuntimeSessionArchiveReaderOwner::Fixture)
    }

    fn with_owner(
        limits: RuntimeSessionArchiveReaderLimits,
        scheduler: JobScheduler,
        owner: RuntimeSessionArchiveReaderOwner,
    ) -> Self {
        let metadata_capacity = limits
            .max_entries
            .saturating_mul(ARCHIVE_READER_ENTRY_METADATA_BYTES);
        Self {
            lane: BoundedKeyedIoLane::new(
                BoundedKeyedIoLimits::new(limits.max_entries, metadata_capacity),
                scheduler,
            ),
            result_budget: RetainedByteBudget::with_max_leases(
                limits.max_retained_result_bytes,
                limits.max_entries,
            ),
            limits,
            state: Arc::new(Mutex::new(RuntimeSessionArchiveReaderState {
                accepting: true,
                next_generation: 1,
                requests: BTreeMap::new(),
            })),
            owner,
        }
    }

    /// 提交已解析的项目路径；路径身份用于去重，实际文件读取仅在获准的 I/O 任务中发生。
    /// 同路径未完成请求会返回同一票据；调用方应理解共享取消和截止时间语义。
    pub fn try_submit(
        &self,
        path: ResolvedProjectPath,
        deadline: BoundedKeyedIoWorkDeadline,
    ) -> Result<RuntimeSessionArchiveReadSubmission, RuntimeSessionArchiveReaderSubmitError> {
        let runtime_admission_lease = match &self.owner {
            RuntimeSessionArchiveReaderOwner::Runtime(runtime) => Some(
                runtime
                    .upgrade()
                    .ok_or(RuntimeSessionArchiveReaderSubmitError::RuntimeUnavailable)?,
            ),
            #[cfg(test)]
            RuntimeSessionArchiveReaderOwner::Fixture => None,
        };
        let mut state = lock(&self.state);
        if !state.accepting {
            return Err(RuntimeSessionArchiveReaderSubmitError::Closed);
        }
        let path_identity = ResolvedProjectPathIdentity::from(path.clone());
        if let Some(request) = state.requests.get(&path_identity).and_then(Weak::upgrade) {
            // TODO: [CR-DYNAMIC-SESSION-FACADE-0002] 确认后续同路径调用所给更早 deadline 是否应影响共享请求；当前直接返回旧票据，缺少跨调用方截止时间契约和覆盖测试。
            if request.ticket.terminal().is_none() {
                return Ok(RuntimeSessionArchiveReadSubmission { request });
            }
            state.requests.remove(&path_identity);
        }

        let result_reservation = self
            .result_budget
            .try_reserve(self.limits.max_archive_bytes)?;
        let generation = state.next_generation;
        state.next_generation = generation
            .checked_add(1)
            .ok_or(RuntimeSessionArchiveReaderSubmitError::GenerationExhausted)?;
        let retained_metadata_bytes = retained_metadata_bytes(path.operation_path()).ok_or(
            RuntimeSessionArchiveReaderSubmitError::Admission(
                BoundedKeyedIoAdmissionError::RetainedBytesOverflow,
            ),
        )?;
        let outcome = Arc::new(Mutex::new(None));
        let outcome_for_work = Arc::clone(&outcome);
        let archive_byte_limit = self.limits.max_archive_bytes;
        let admission = self
            .lane
            .try_admit(
                BoundedKeyedIoKey::from_value(path_identity.clone()),
                generation,
                retained_metadata_bytes,
                deadline,
                Box::new(move || {
                    match load_from_path_with_limit(path.operation_path(), archive_byte_limit) {
                        Ok(archive) => {
                            *lock(&outcome_for_work) =
                                Some(RuntimeSessionArchiveReadOutcome::Succeeded(
                                    RuntimeSessionArchiveReadArtifact::new(
                                        archive,
                                        result_reservation,
                                    ),
                                ));
                            Ok(())
                        }
                        Err(error) => {
                            *lock(&outcome_for_work) =
                                Some(RuntimeSessionArchiveReadOutcome::Failed(Arc::new(error)));
                            Err(BoundedKeyedIoFailure::new(ARCHIVE_READ_FAILURE))
                        }
                    }
                }),
            )
            .map_err(RuntimeSessionArchiveReaderSubmitError::Admission)?;
        let request = Arc::new(RuntimeSessionArchiveReadRequest {
            ticket: admission.ticket(),
            cancel_authority: admission.cancel_authority(),
            outcome,
            reader_state: Arc::downgrade(&self.state),
            path_identity: path_identity.clone(),
        });
        state
            .requests
            .insert(path_identity, Arc::downgrade(&request));
        admission.activate();
        drop(state);
        drop(runtime_admission_lease);
        Ok(RuntimeSessionArchiveReadSubmission { request })
    }

    pub fn diagnostics(&self) -> RuntimeSessionArchiveReaderDiagnostics {
        let result_budget = self.result_budget.diagnostics();
        RuntimeSessionArchiveReaderDiagnostics {
            io: self.lane.diagnostics(),
            retained_result_bytes: result_budget.retained_bytes,
            retained_result_leases: result_budget.active_leases,
        }
    }

    pub fn shutdown(&self) -> BoundedKeyedIoShutdownGuard {
        lock(&self.state).accepting = false;
        self.result_budget.close();
        self.lane.shutdown()
    }
}

impl RuntimeSessionArchiveReadRequest {
    pub(super) fn outcome(&self) -> Option<RuntimeSessionArchiveReadOutcome> {
        lock(&self.outcome).clone()
    }
}

impl Drop for RuntimeSessionArchiveReadRequest {
    fn drop(&mut self) {
        let _ = self.ticket.cancel_before_start(&self.cancel_authority);
        let Some(reader_state) = self.reader_state.upgrade() else {
            return;
        };
        let mut state = lock(&reader_state);
        if state
            .requests
            .get(&self.path_identity)
            .is_some_and(|request| std::ptr::eq(request.as_ptr(), std::ptr::from_ref(self)))
        {
            state.requests.remove(&self.path_identity);
        }
    }
}

fn retained_metadata_bytes(path: &Path) -> Option<usize> {
    path.as_os_str()
        .len()
        .checked_add(ARCHIVE_READER_METADATA_BYTES)
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
