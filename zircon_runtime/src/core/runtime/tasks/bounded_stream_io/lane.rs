use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use super::super::{
    EngineTaskGraph, TaskCancellationPolicy, TaskDescriptor, TaskGraphScope, TaskGraphScopeCensus,
    TaskGraphScopeDescriptor, TaskHandle, TaskId, TaskPoolKind,
};
use super::capture::BoundedStreamIoCapture;
use super::model::{
    BoundedStreamIoAdmissionError, BoundedStreamIoLaneDiagnostics, BoundedStreamIoLimits,
    BoundedStreamIoReader,
};
use super::state::CaptureState;
use super::worker::{run_reader, ReaderStartGate};

pub struct BoundedStreamIoLane {
    scope: TaskGraphScope,
    limits: BoundedStreamIoLimits,
    next_task_handle: AtomicU64,
    reader_budget: Arc<ReaderBudget>,
}

impl BoundedStreamIoLane {
    pub fn try_new(
        runtime: &EngineTaskGraph,
        owner: impl Into<String>,
        limits: BoundedStreamIoLimits,
    ) -> Result<Self, BoundedStreamIoAdmissionError> {
        let limits = limits.validate()?;
        let owner = owner.into();
        if owner.is_empty() {
            return Err(BoundedStreamIoAdmissionError::EmptyOwner);
        }
        let reader_capacity = limits
            .max_concurrent_readers
            .min(runtime.worker_pool().parallelism());
        let scope = runtime.create_scope(
            TaskGraphScopeDescriptor::new(owner).with_task_capacity(reader_capacity),
        )?;
        Ok(Self {
            scope,
            limits,
            next_task_handle: AtomicU64::new(1),
            reader_budget: Arc::new(ReaderBudget::new(reader_capacity)),
        })
    }

    pub fn limits(&self) -> BoundedStreamIoLimits {
        self.limits
    }

    pub fn reader_capacity(&self) -> usize {
        self.reader_budget.limit
    }

    pub fn diagnostics(&self) -> BoundedStreamIoLaneDiagnostics {
        self.reader_budget.diagnostics()
    }

    pub fn scope_census(&self) -> TaskGraphScopeCensus {
        self.scope.census()
    }

    pub fn capture(
        &self,
        readers: Vec<BoundedStreamIoReader>,
    ) -> Result<BoundedStreamIoCapture, BoundedStreamIoAdmissionError> {
        if readers.is_empty() {
            return Err(BoundedStreamIoAdmissionError::EmptyReaders);
        }
        if readers.iter().any(|reader| reader.stream.is_empty()) {
            return Err(BoundedStreamIoAdmissionError::EmptyStreamId);
        }

        let reader_count = readers.len();
        let permits = self.reader_budget.try_reserve(reader_count)?;
        let handles = match self.allocate_task_handles(reader_count) {
            Ok(handles) => handles,
            Err(error) => {
                self.reader_budget.record_rejected(reader_count);
                return Err(error);
            }
        };
        let state = Arc::new(CaptureState::new(reader_count));
        let gate = Arc::new(ReaderStartGate::new());
        let mut start_transaction = ReaderStartTransaction::new(Arc::clone(&gate));
        let mut tasks = Vec::<TaskHandle>::with_capacity(reader_count);
        let limits = self.limits;

        for ((input, permit), handle) in readers.into_iter().zip(permits).zip(handles) {
            let label = format!("bounded-stream-io.{}", input.stream);
            let descriptor = TaskDescriptor::new(handle, TaskPoolKind::Io, label)
                .with_cancellation_policy(TaskCancellationPolicy::FinishOnShutdown);
            let worker_state = Arc::clone(&state);
            let worker_gate = Arc::clone(&gate);
            match self.scope.submit(descriptor, move |token| {
                run_reader(input, worker_state, limits, worker_gate, token, permit);
            }) {
                Ok(task) => tasks.push(task),
                Err(error) => {
                    self.reader_budget.record_rejected(reader_count);
                    return Err(error.into());
                }
            }
        }

        self.reader_budget.record_admitted(reader_count);
        start_transaction.commit();
        Ok(BoundedStreamIoCapture::new(state, tasks))
    }

    fn allocate_task_handles(
        &self,
        count: usize,
    ) -> Result<Vec<TaskId>, BoundedStreamIoAdmissionError> {
        let count = u64::try_from(count)
            .map_err(|_| BoundedStreamIoAdmissionError::TaskHandleSpaceExhausted)?;
        let start = self
            .next_task_handle
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(count)
            })
            .map_err(|_| BoundedStreamIoAdmissionError::TaskHandleSpaceExhausted)?;
        Ok((start..start + count).map(TaskId::new).collect())
    }
}

struct ReaderStartTransaction {
    gate: Arc<ReaderStartGate>,
    committed: bool,
}

impl ReaderStartTransaction {
    fn new(gate: Arc<ReaderStartGate>) -> Self {
        Self {
            gate,
            committed: false,
        }
    }

    fn commit(&mut self) {
        self.gate.start();
        self.committed = true;
    }
}

impl Drop for ReaderStartTransaction {
    fn drop(&mut self) {
        if !self.committed {
            self.gate.abort();
        }
    }
}

struct ReaderBudget {
    limit: usize,
    active: AtomicUsize,
    peak: AtomicUsize,
    admitted: AtomicU64,
    rejected: AtomicU64,
}

impl ReaderBudget {
    fn new(limit: usize) -> Self {
        Self {
            limit,
            active: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            admitted: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
        }
    }

    fn try_reserve(
        self: &Arc<Self>,
        count: usize,
    ) -> Result<Vec<ReaderPermit>, BoundedStreamIoAdmissionError> {
        let mut active = self.active.load(Ordering::Acquire);
        loop {
            let available = self.limit.saturating_sub(active);
            if count > available {
                self.record_rejected(count);
                return Err(BoundedStreamIoAdmissionError::ReaderCapacityReached {
                    requested: count,
                    available,
                });
            }
            let next = active
                .checked_add(count)
                .ok_or(BoundedStreamIoAdmissionError::TaskHandleSpaceExhausted)?;
            match self.active.compare_exchange_weak(
                active,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.peak.fetch_max(next, Ordering::Relaxed);
                    return Ok((0..count)
                        .map(|_| ReaderPermit {
                            budget: Arc::clone(self),
                        })
                        .collect());
                }
                Err(current) => active = current,
            }
        }
    }

    fn record_admitted(&self, count: usize) {
        self.admitted.fetch_add(count as u64, Ordering::Relaxed);
    }

    fn record_rejected(&self, count: usize) {
        self.rejected.fetch_add(count as u64, Ordering::Relaxed);
    }

    fn diagnostics(&self) -> BoundedStreamIoLaneDiagnostics {
        BoundedStreamIoLaneDiagnostics {
            active_readers: self.active.load(Ordering::Acquire),
            peak_active_readers: self.peak.load(Ordering::Relaxed),
            admitted_readers: self.admitted.load(Ordering::Relaxed),
            rejected_readers: self.rejected.load(Ordering::Relaxed),
        }
    }
}

pub(super) struct ReaderPermit {
    budget: Arc<ReaderBudget>,
}

impl Drop for ReaderPermit {
    fn drop(&mut self) {
        let previous = self.budget.active.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0, "bounded stream reader budget underflow");
    }
}
