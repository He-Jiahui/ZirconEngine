use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::asset::artifact::{
    RenderArtifactBlockLoadStage, RenderArtifactBlockLoader, RenderArtifactIoPriority,
    RenderArtifactManifestLoader,
};
use crate::core::runtime::{
    EngineTaskGraph, TaskDescriptor, TaskGraphScope, TaskGraphScopeDescriptor, TaskHandle, TaskId,
    TaskPoolKind,
};

use super::{
    RenderAssetCpuArtifactLease, RenderAssetGpuUploadPlan, RenderAssetGpuUploadPlanError,
    RenderAssetResidencyManager, RenderAssetResidencyRoute, RenderAssetResidencyState,
    RenderAssetResidencyTicket, RenderAssetResidencyTransitionError, RenderAssetSemanticLoad,
    RenderAssetSemanticLoadAdvance, RenderAssetSemanticLoadStage,
};

mod contract;
mod owner;

pub(crate) use contract::{
    RenderAssetSemanticExecutorAdmissionError, RenderAssetSemanticExecutorCloseReport,
    RenderAssetSemanticExecutorDiagnostics, RenderAssetSemanticExecutorFailure,
    RenderAssetSemanticExecutorInitError, RenderAssetSemanticExecutorLimits,
    RenderAssetSemanticExecutorMaintenanceBudget, RenderAssetSemanticExecutorMaintenanceError,
    RenderAssetSemanticExecutorMaintenanceReport, RenderAssetSemanticExecutorWorkError,
};
pub(crate) use owner::{
    RenderAssetSemanticExecutorOwner, RenderAssetSemanticExecutorOwnerAdmissionError,
    RenderAssetSemanticExecutorOwnerConfig, RenderAssetSemanticExecutorOwnerInitError,
    RenderAssetSemanticExecutorOwnerMaintenanceError,
};

struct ActiveSemanticLoad {
    ticket: RenderAssetResidencyTicket,
    load: RenderAssetSemanticLoad,
}

struct PreparingSemanticUpload {
    _task: TaskHandle,
    ticket: RenderAssetResidencyTicket,
    cancelled: Arc<AtomicBool>,
}

struct PrepareCompletion {
    ticket: RenderAssetResidencyTicket,
    outcome: Option<Result<RenderAssetGpuUploadPlan, RenderAssetGpuUploadPlanError>>,
}

pub(crate) struct RenderAssetSemanticExecutor {
    manifest_loader: RenderArtifactManifestLoader,
    block_loader: RenderArtifactBlockLoader,
    target_platform: Arc<str>,
    limits: RenderAssetSemanticExecutorLimits,
    prepare_scope: TaskGraphScope,
    prepare_sender: crossbeam_channel::Sender<PrepareCompletion>,
    prepare_receiver: crossbeam_channel::Receiver<PrepareCompletion>,
    active_loads: BTreeMap<u64, ActiveSemanticLoad>,
    ready_cpu: BTreeMap<u64, RenderAssetCpuArtifactLease>,
    preparing_uploads: BTreeMap<u64, PreparingSemanticUpload>,
    ready_uploads: BTreeMap<u64, RenderAssetGpuUploadPlan>,
    failures: BTreeMap<u64, RenderAssetSemanticExecutorFailure>,
    load_poll_cursor: Option<u64>,
    next_prepare_task_id: u64,
    closed: bool,
}

impl RenderAssetSemanticExecutor {
    pub(crate) fn try_new(
        manifest_loader: RenderArtifactManifestLoader,
        block_loader: RenderArtifactBlockLoader,
        target_platform: Arc<str>,
        limits: RenderAssetSemanticExecutorLimits,
        runtime: &EngineTaskGraph,
    ) -> Result<Self, RenderAssetSemanticExecutorInitError> {
        validate_limits(limits)?;
        if target_platform.trim().is_empty() {
            return Err(RenderAssetSemanticExecutorInitError::EmptyTargetPlatform);
        }
        let prepare_scope = runtime.create_scope(
            TaskGraphScopeDescriptor::new("render-asset-semantic-upload-prepare")
                .with_task_capacity(limits.max_in_flight()),
        )?;
        let (prepare_sender, prepare_receiver) = crossbeam_channel::bounded(limits.max_in_flight());
        Ok(Self {
            manifest_loader,
            block_loader,
            target_platform,
            limits,
            prepare_scope,
            prepare_sender,
            prepare_receiver,
            active_loads: BTreeMap::new(),
            ready_cpu: BTreeMap::new(),
            preparing_uploads: BTreeMap::new(),
            ready_uploads: BTreeMap::new(),
            failures: BTreeMap::new(),
            load_poll_cursor: None,
            next_prepare_task_id: 1,
            closed: false,
        })
    }

    pub(crate) fn admit(
        &mut self,
        ticket: RenderAssetResidencyTicket,
        priority: RenderArtifactIoPriority,
        deadline: Option<Instant>,
        residency: &mut RenderAssetResidencyManager,
    ) -> Result<(), RenderAssetSemanticExecutorAdmissionError> {
        if self.closed {
            return Err(RenderAssetSemanticExecutorAdmissionError::Closed);
        }
        if ticket.route() != RenderAssetResidencyRoute::SemanticBlocks {
            return Err(
                RenderAssetSemanticExecutorAdmissionError::UnsupportedRoute {
                    actual: ticket.route(),
                },
            );
        }
        let ticket_id = ticket.id().raw();
        if self.contains_ticket(ticket_id) {
            return Err(RenderAssetSemanticExecutorAdmissionError::DuplicateTicket {
                ticket: ticket.id(),
            });
        }
        if self.diagnostics().in_flight() >= self.limits.max_in_flight() {
            return Err(
                RenderAssetSemanticExecutorAdmissionError::CapacityExceeded {
                    capacity: self.limits.max_in_flight(),
                },
            );
        }
        let load = RenderAssetSemanticLoad::begin(
            ticket.clone(),
            Arc::clone(&self.target_platform),
            &self.manifest_loader,
            priority,
            deadline,
        )?;
        residency
            .advance(&ticket, RenderAssetResidencyState::Reading)
            .map_err(RenderAssetSemanticExecutorAdmissionError::Residency)?;
        self.active_loads
            .insert(ticket_id, ActiveSemanticLoad { ticket, load });
        Ok(())
    }

    /// Cancels executor-owned CPU work only. The residency owner must separately consume the
    /// matching release mutation so the ticket state and reference count remain authoritative.
    pub(crate) fn cancel(&mut self, ticket: &RenderAssetResidencyTicket) -> bool {
        let id = ticket.id().raw();
        if self
            .active_loads
            .get(&id)
            .is_some_and(|work| &work.ticket == ticket)
        {
            self.active_loads.remove(&id);
            return true;
        }
        if self
            .ready_cpu
            .get(&id)
            .is_some_and(|work| &work.ticket() == ticket)
        {
            self.ready_cpu.remove(&id);
            return true;
        }
        if self
            .ready_uploads
            .get(&id)
            .is_some_and(|work| &work.ticket() == ticket)
        {
            self.ready_uploads.remove(&id);
            return true;
        }
        if self
            .failures
            .get(&id)
            .is_some_and(|work| &work.ticket == ticket)
        {
            self.failures.remove(&id);
            return true;
        }
        let Some(preparing) = self.preparing_uploads.get(&id) else {
            return false;
        };
        if &preparing.ticket != ticket {
            return false;
        }
        preparing.cancelled.store(true, Ordering::Release);
        true
    }

    /// 每轮先消费准备结果，再推进 I/O，最后准入新的 CPU 准备；各阶段有独立预算，就绪计划交由 RHI owner 提交。
    pub(crate) fn maintain(
        &mut self,
        budget: RenderAssetSemanticExecutorMaintenanceBudget,
        residency: &mut RenderAssetResidencyManager,
    ) -> Result<
        RenderAssetSemanticExecutorMaintenanceReport,
        RenderAssetSemanticExecutorMaintenanceError,
    > {
        if self.closed {
            return Err(RenderAssetSemanticExecutorMaintenanceError::Closed);
        }
        let mut report = RenderAssetSemanticExecutorMaintenanceReport::default();
        self.drain_prepare_completions(budget.max_prepare_completions, residency, &mut report);

        if !self.active_loads.is_empty() {
            report.manifest_io = self.manifest_loader.dispatch_io(budget.manifest_io)?;
            report.block_io = self.block_loader.dispatch_io(budget.block_io)?;
        }
        self.advance_loads(budget.max_load_advances, residency, &mut report);
        self.submit_cpu_preparations(budget.max_prepare_submissions, &mut report)?;
        Ok(report)
    }

    pub(crate) fn take_next_ready_upload(&mut self) -> Option<RenderAssetGpuUploadPlan> {
        self.ready_uploads.pop_first().map(|(_, plan)| plan)
    }

    pub(crate) fn take_next_failure(&mut self) -> Option<RenderAssetSemanticExecutorFailure> {
        self.failures.pop_first().map(|(_, failure)| failure)
    }

    pub(crate) fn diagnostics(&self) -> RenderAssetSemanticExecutorDiagnostics {
        RenderAssetSemanticExecutorDiagnostics {
            active_loads: self.active_loads.len(),
            ready_cpu: self.ready_cpu.len(),
            preparing_uploads: self.preparing_uploads.len(),
            ready_uploads: self.ready_uploads.len(),
            failures: self.failures.len(),
        }
    }

    pub(crate) fn close(&mut self) -> RenderAssetSemanticExecutorCloseReport {
        if self.closed {
            return RenderAssetSemanticExecutorCloseReport::default();
        }
        self.closed = true;
        self.prepare_scope.close_admission();
        let manifest_loader = self.manifest_loader.close();
        let block_loader = self.block_loader.close();

        let mut tickets = Vec::with_capacity(self.diagnostics().in_flight());
        tickets.extend(
            self.active_loads
                .values()
                .map(|active| active.ticket.clone()),
        );
        tickets.extend(
            self.ready_cpu
                .values()
                .map(RenderAssetCpuArtifactLease::ticket),
        );
        for preparing in self.preparing_uploads.values() {
            preparing.cancelled.store(true, Ordering::Release);
            preparing._task.request_cancellation();
            tickets.push(preparing.ticket.clone());
        }
        tickets.extend(
            self.ready_uploads
                .values()
                .map(RenderAssetGpuUploadPlan::ticket),
        );
        tickets.extend(
            self.failures
                .values()
                .map(RenderAssetSemanticExecutorFailure::ticket),
        );

        self.active_loads.clear();
        self.ready_cpu.clear();
        self.preparing_uploads.clear();
        self.ready_uploads.clear();
        self.failures.clear();
        self.load_poll_cursor = None;

        RenderAssetSemanticExecutorCloseReport {
            tickets,
            manifest_loader,
            block_loader,
        }
    }

    #[cfg(test)]
    pub(crate) fn close_prepare_admission_for_test(&self) {
        self.prepare_scope.close_admission();
    }

    fn advance_loads(
        &mut self,
        max_advances: usize,
        residency: &mut RenderAssetResidencyManager,
        report: &mut RenderAssetSemanticExecutorMaintenanceReport,
    ) {
        let advance_count = max_advances.min(self.active_loads.len());
        for _ in 0..advance_count {
            let Some(id) = self.next_active_load_id() else {
                break;
            };
            self.load_poll_cursor = Some(id);
            let Some(active) = self.active_loads.remove(&id) else {
                continue;
            };
            report.load_advances = report.load_advances.saturating_add(1);
            if residency.pending_ticket(active.ticket.resource()).as_ref() != Some(&active.ticket) {
                report.stale_discards = report.stale_discards.saturating_add(1);
                continue;
            }
            match active.load.advance(&self.block_loader) {
                Ok(RenderAssetSemanticLoadAdvance::Pending(load, stage)) => {
                    match align_pending_stage(residency, &active.ticket, stage) {
                        Ok(()) => {
                            self.active_loads.insert(
                                id,
                                ActiveSemanticLoad {
                                    ticket: active.ticket,
                                    load,
                                },
                            );
                        }
                        Err(error) => {
                            self.record_failure(
                                active.ticket,
                                RenderAssetSemanticExecutorWorkError::Residency(error),
                                residency,
                            );
                            report.failed = report.failed.saturating_add(1);
                        }
                    }
                }
                Ok(RenderAssetSemanticLoadAdvance::Deferred(load, _)) => {
                    self.active_loads.insert(
                        id,
                        ActiveSemanticLoad {
                            ticket: active.ticket,
                            load,
                        },
                    );
                    report.deferred_loads = report.deferred_loads.saturating_add(1);
                }
                Ok(RenderAssetSemanticLoadAdvance::Ready(cpu_lease)) => {
                    match advance_to_ready_cpu(residency, &active.ticket) {
                        Ok(()) => {
                            self.ready_cpu.insert(id, cpu_lease);
                            report.ready_cpu = report.ready_cpu.saturating_add(1);
                        }
                        Err(error) => {
                            self.record_failure(
                                active.ticket,
                                RenderAssetSemanticExecutorWorkError::Residency(error),
                                residency,
                            );
                            report.failed = report.failed.saturating_add(1);
                        }
                    }
                }
                Err(error) => {
                    self.record_failure(
                        active.ticket,
                        RenderAssetSemanticExecutorWorkError::Semantic(error),
                        residency,
                    );
                    report.failed = report.failed.saturating_add(1);
                }
            }
        }
    }

    fn submit_cpu_preparations(
        &mut self,
        max_submissions: usize,
        report: &mut RenderAssetSemanticExecutorMaintenanceReport,
    ) -> Result<(), RenderAssetSemanticExecutorMaintenanceError> {
        for _ in 0..max_submissions.min(self.ready_cpu.len()) {
            let Some((id, cpu_lease)) = self.ready_cpu.pop_first() else {
                break;
            };
            let task_id = self.next_prepare_task_id;
            let Some(next_task_id) = task_id.checked_add(1) else {
                self.ready_cpu.insert(id, cpu_lease);
                return Err(RenderAssetSemanticExecutorMaintenanceError::PrepareTaskIdExhausted);
            };
            let ticket = cpu_lease.ticket();
            let completion_ticket = ticket.clone();
            let cancelled = Arc::new(AtomicBool::new(false));
            let cancelled_for_task = Arc::clone(&cancelled);
            let sender = self.prepare_sender.clone();
            let upload_limits = self.limits.upload();
            let lease_slot = Arc::new(Mutex::new(Some(cpu_lease)));
            let lease_slot_for_task = Arc::clone(&lease_slot);
            // TODO: [CR-R02-runtime_wave12_graphics_resource_residency-0004] 待确认取消或 panic 跳过准备体后的业务完成转交；scope 终态只退休其任务，owner 关闭会清表，但关停先后未证，需验证关停顺序及未启动任务回收。
            let task = self.prepare_scope.submit(
                TaskDescriptor::new(
                    TaskId::new(task_id),
                    TaskPoolKind::Compute,
                    "render-asset-semantic-upload-plan",
                ),
                move |_| {
                    let cpu_lease = lease_slot_for_task
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .take();
                    let outcome = match cpu_lease {
                        Some(cpu_lease) if !cancelled_for_task.load(Ordering::Acquire) => {
                            let outcome =
                                RenderAssetGpuUploadPlan::prepare(cpu_lease, upload_limits);
                            if cancelled_for_task.load(Ordering::Acquire) {
                                None
                            } else {
                                Some(outcome)
                            }
                        }
                        Some(_) | None => None,
                    };
                    let _ = sender.try_send(PrepareCompletion {
                        ticket: completion_ticket,
                        outcome,
                    });
                },
            );
            let task = match task {
                Ok(task) => task,
                Err(error) => {
                    if let Some(cpu_lease) = lease_slot
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .take()
                    {
                        self.ready_cpu.insert(id, cpu_lease);
                    }
                    return Err(
                        RenderAssetSemanticExecutorMaintenanceError::PrepareExecution(error),
                    );
                }
            };
            self.next_prepare_task_id = next_task_id;
            self.preparing_uploads.insert(
                id,
                PreparingSemanticUpload {
                    _task: task,
                    ticket,
                    cancelled,
                },
            );
            report.prepare_submissions = report.prepare_submissions.saturating_add(1);
        }
        Ok(())
    }

    fn drain_prepare_completions(
        &mut self,
        max_completions: usize,
        residency: &mut RenderAssetResidencyManager,
        report: &mut RenderAssetSemanticExecutorMaintenanceReport,
    ) {
        for _ in 0..max_completions {
            let completion = match self.prepare_receiver.try_recv() {
                Ok(completion) => completion,
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => break,
            };
            let id = completion.ticket.id().raw();
            let Some(preparing) = self.preparing_uploads.remove(&id) else {
                report.stale_discards = report.stale_discards.saturating_add(1);
                continue;
            };
            report.prepare_completions = report.prepare_completions.saturating_add(1);
            if preparing.cancelled.load(Ordering::Acquire)
                || preparing.ticket != completion.ticket
                || completion.outcome.is_none()
                || residency
                    .pending_ticket(completion.ticket.resource())
                    .as_ref()
                    != Some(&completion.ticket)
            {
                report.stale_discards = report.stale_discards.saturating_add(1);
                continue;
            }
            let Some(outcome) = completion.outcome else {
                report.stale_discards = report.stale_discards.saturating_add(1);
                continue;
            };
            match outcome {
                Ok(plan) => match residency
                    .advance(&completion.ticket, RenderAssetResidencyState::QueuedUpload)
                {
                    Ok(()) => {
                        self.ready_uploads.insert(id, plan);
                        report.ready_uploads = report.ready_uploads.saturating_add(1);
                    }
                    Err(error) => {
                        self.record_failure(
                            completion.ticket,
                            RenderAssetSemanticExecutorWorkError::Residency(error),
                            residency,
                        );
                        report.failed = report.failed.saturating_add(1);
                    }
                },
                Err(error) => {
                    self.record_failure(
                        completion.ticket,
                        RenderAssetSemanticExecutorWorkError::UploadPlan(error),
                        residency,
                    );
                    report.failed = report.failed.saturating_add(1);
                }
            }
        }
    }

    fn record_failure(
        &mut self,
        ticket: RenderAssetResidencyTicket,
        error: RenderAssetSemanticExecutorWorkError,
        residency: &mut RenderAssetResidencyManager,
    ) {
        let terminal_transition_error = residency.fail_pending(&ticket).err();
        self.failures.insert(
            ticket.id().raw(),
            RenderAssetSemanticExecutorFailure {
                ticket,
                error,
                terminal_transition_error,
            },
        );
    }

    fn next_active_load_id(&self) -> Option<u64> {
        match self.load_poll_cursor {
            Some(cursor) => self
                .active_loads
                .range((Excluded(cursor), Unbounded))
                .next()
                .map(|(id, _)| *id)
                .or_else(|| self.active_loads.first_key_value().map(|(id, _)| *id)),
            None => self.active_loads.first_key_value().map(|(id, _)| *id),
        }
    }

    fn contains_ticket(&self, id: u64) -> bool {
        self.active_loads.contains_key(&id)
            || self.ready_cpu.contains_key(&id)
            || self.preparing_uploads.contains_key(&id)
            || self.ready_uploads.contains_key(&id)
            || self.failures.contains_key(&id)
    }
}

impl Drop for RenderAssetSemanticExecutor {
    fn drop(&mut self) {
        self.close();
    }
}

fn align_pending_stage(
    residency: &mut RenderAssetResidencyManager,
    ticket: &RenderAssetResidencyTicket,
    stage: RenderAssetSemanticLoadStage,
) -> Result<(), RenderAssetResidencyTransitionError> {
    let mut current =
        residency
            .state(ticket)
            .ok_or(RenderAssetResidencyTransitionError::UnknownTicket {
                presented: ticket.id(),
            })?;
    if current == RenderAssetResidencyState::QueuedIo {
        residency.advance(ticket, RenderAssetResidencyState::Reading)?;
        current = RenderAssetResidencyState::Reading;
    }
    let is_decode = matches!(
        stage,
        RenderAssetSemanticLoadStage::Blocks(
            RenderArtifactBlockLoadStage::QueuedDecode | RenderArtifactBlockLoadStage::Decoding
        )
    );
    if is_decode && current == RenderAssetResidencyState::Reading {
        residency.advance(ticket, RenderAssetResidencyState::Decoding)?;
        current = RenderAssetResidencyState::Decoding;
    }
    if matches!(
        current,
        RenderAssetResidencyState::Reading | RenderAssetResidencyState::Decoding
    ) {
        return Ok(());
    }
    Err(RenderAssetResidencyTransitionError::InvalidTransition {
        from: current,
        to: if is_decode {
            RenderAssetResidencyState::Decoding
        } else {
            RenderAssetResidencyState::Reading
        },
    })
}

fn advance_to_ready_cpu(
    residency: &mut RenderAssetResidencyManager,
    ticket: &RenderAssetResidencyTicket,
) -> Result<(), RenderAssetResidencyTransitionError> {
    let current =
        residency
            .state(ticket)
            .ok_or(RenderAssetResidencyTransitionError::UnknownTicket {
                presented: ticket.id(),
            })?;
    match current {
        RenderAssetResidencyState::QueuedIo => {
            residency.advance(ticket, RenderAssetResidencyState::Reading)?;
            residency.advance(ticket, RenderAssetResidencyState::Decoding)?;
        }
        RenderAssetResidencyState::Reading => {
            residency.advance(ticket, RenderAssetResidencyState::Decoding)?;
        }
        RenderAssetResidencyState::Decoding => {}
        current => {
            return Err(RenderAssetResidencyTransitionError::InvalidTransition {
                from: current,
                to: RenderAssetResidencyState::ReadyCpu,
            });
        }
    }
    residency.advance(ticket, RenderAssetResidencyState::ReadyCpu)
}

fn validate_limits(
    limits: RenderAssetSemanticExecutorLimits,
) -> Result<(), RenderAssetSemanticExecutorInitError> {
    for (limit, value) in [
        ("max_in_flight", limits.max_in_flight()),
        (
            "max_upload_subresources",
            limits.upload().max_subresources(),
        ),
    ] {
        if value == 0 {
            return Err(RenderAssetSemanticExecutorInitError::ZeroLimit { limit });
        }
    }
    for (limit, value) in [
        (
            "max_upload_staging_bytes",
            limits.upload().max_staging_bytes(),
        ),
        (
            "max_upload_destination_bytes",
            limits.upload().max_destination_bytes(),
        ),
    ] {
        if value == 0 {
            return Err(RenderAssetSemanticExecutorInitError::ZeroLimit { limit });
        }
    }
    Ok(())
}
