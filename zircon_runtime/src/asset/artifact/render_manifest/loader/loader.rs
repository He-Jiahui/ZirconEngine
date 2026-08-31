use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::Instant;

use crate::core::runtime::{EngineTaskGraph, TaskDescriptor, TaskGraphScope, TaskId, TaskPoolKind};

use super::super::{RenderArtifactBlockDescriptor, RenderArtifactStore};
use super::contract::{
    RenderArtifactBlockAdmissionError, RenderArtifactBlockCancelReason, RenderArtifactBlockFailure,
    RenderArtifactBlockFailureCode, RenderArtifactBlockIoDispatchBudget,
    RenderArtifactBlockIoDispatchError, RenderArtifactBlockIoDispatchReport,
    RenderArtifactBlockLoaderCloseReport, RenderArtifactBlockLoaderDiagnostics,
    RenderArtifactBlockLoaderInitError, RenderArtifactBlockLoaderLimits,
    RenderArtifactBlockMaintenanceReport, RenderArtifactBlockPoll, RenderArtifactBlockRequest,
};
use super::entry::RenderArtifactBlockEntry;
use super::policy::validate_limits;
use super::registry::{
    RenderArtifactBlockRegistry, RenderArtifactDecodeKey, TICKET_ACTIVE, TICKET_CALLER_CANCELLED,
    TICKET_EXPIRED, TICKET_OWNER_CLOSED, TicketRegistration, remove_entry,
    remove_registered_ticket, take_task_id,
};
use super::worker::{RenderArtifactBlockLoaderMetrics, atomic_add, run_decode_task};

pub(super) struct RenderArtifactBlockLoaderInner {
    pub(super) store: RenderArtifactStore,
    pub(super) limits: RenderArtifactBlockLoaderLimits,
    pub(super) scope: TaskGraphScope,
    pub(super) registry: Mutex<RenderArtifactBlockRegistry>,
    pub(super) metrics: Arc<RenderArtifactBlockLoaderMetrics>,
}

#[derive(Clone)]
pub struct RenderArtifactBlockLoader {
    inner: Arc<RenderArtifactBlockLoaderInner>,
}

pub struct RenderArtifactBlockTicket {
    id: u64,
    entry: Arc<RenderArtifactBlockEntry>,
    descriptor: RenderArtifactBlockDescriptor,
    registration: Arc<TicketRegistration>,
    loader: Weak<RenderArtifactBlockLoaderInner>,
}

pub struct RenderArtifactBlockTicketBatch {
    tickets: Vec<RenderArtifactBlockTicket>,
}

impl RenderArtifactBlockLoader {
    pub fn new(
        store: RenderArtifactStore,
        limits: RenderArtifactBlockLoaderLimits,
        runtime: &EngineTaskGraph,
    ) -> Result<Self, RenderArtifactBlockLoaderInitError> {
        validate_limits(limits)?;
        let task_capacity = limits
            .max_entries()
            .checked_mul(2)
            .ok_or(RenderArtifactBlockLoaderInitError::ScopeCapacityOverflow)?;
        let scope = runtime.create_scope(
            crate::core::runtime::TaskGraphScopeDescriptor::new("render-artifact-block-loader")
                .with_task_capacity(task_capacity),
        )?;
        Ok(Self {
            inner: Arc::new(RenderArtifactBlockLoaderInner {
                store,
                limits,
                scope,
                registry: Mutex::new(RenderArtifactBlockRegistry::new()),
                metrics: Arc::default(),
            }),
        })
    }

    pub fn request(
        &self,
        request: RenderArtifactBlockRequest,
    ) -> Result<RenderArtifactBlockTicket, RenderArtifactBlockAdmissionError> {
        let mut batch = self.inner.request_batch(std::slice::from_ref(&request))?;
        batch
            .tickets
            .pop()
            .ok_or(RenderArtifactBlockAdmissionError::EmptyBatch)
    }

    pub fn request_batch(
        &self,
        requests: &[RenderArtifactBlockRequest],
    ) -> Result<RenderArtifactBlockTicketBatch, RenderArtifactBlockAdmissionError> {
        self.inner.request_batch(requests)
    }

    pub fn dispatch_io(
        &self,
        budget: RenderArtifactBlockIoDispatchBudget,
    ) -> Result<RenderArtifactBlockIoDispatchReport, RenderArtifactBlockIoDispatchError> {
        self.inner.dispatch_io(budget)
    }

    pub fn maintain_deadlines(&self, now: Instant) -> RenderArtifactBlockMaintenanceReport {
        self.inner.maintain_deadlines(now)
    }

    pub fn diagnostics(&self) -> RenderArtifactBlockLoaderDiagnostics {
        self.inner.diagnostics()
    }

    pub fn close(&self) -> RenderArtifactBlockLoaderCloseReport {
        self.inner.close()
    }
}

impl RenderArtifactBlockTicketBatch {
    pub(super) fn new(tickets: Vec<RenderArtifactBlockTicket>) -> Self {
        Self { tickets }
    }

    pub fn tickets(&self) -> &[RenderArtifactBlockTicket] {
        &self.tickets
    }

    pub fn into_tickets(self) -> Vec<RenderArtifactBlockTicket> {
        self.tickets
    }
}

impl RenderArtifactBlockTicket {
    pub(super) fn from_parts(
        id: u64,
        entry: Arc<RenderArtifactBlockEntry>,
        descriptor: RenderArtifactBlockDescriptor,
        registration: Arc<TicketRegistration>,
        loader: Weak<RenderArtifactBlockLoaderInner>,
    ) -> Self {
        Self {
            id,
            entry,
            descriptor,
            registration,
            loader,
        }
    }

    pub const fn id(&self) -> u64 {
        self.id
    }

    pub fn poll(&self) -> RenderArtifactBlockPoll {
        match self.registration.status() {
            TICKET_ACTIVE => self.entry.poll(&self.descriptor),
            TICKET_EXPIRED => {
                RenderArtifactBlockPoll::Cancelled(RenderArtifactBlockCancelReason::Deadline)
            }
            TICKET_OWNER_CLOSED => {
                RenderArtifactBlockPoll::Cancelled(RenderArtifactBlockCancelReason::OwnerClosed)
            }
            _ => RenderArtifactBlockPoll::Cancelled(RenderArtifactBlockCancelReason::Caller),
        }
    }

    pub fn cancel(self) {
        if self
            .registration
            .transition_from_active(TICKET_CALLER_CANCELLED)
        {
            if let Some(loader) = self.loader.upgrade() {
                loader.release_ticket(
                    self.id,
                    &self.entry,
                    RenderArtifactBlockCancelReason::Caller,
                );
            }
        }
    }
}

impl Drop for RenderArtifactBlockTicket {
    fn drop(&mut self) {
        if self
            .registration
            .transition_from_active(TICKET_CALLER_CANCELLED)
        {
            if let Some(loader) = self.loader.upgrade() {
                loader.release_ticket(
                    self.id,
                    &self.entry,
                    RenderArtifactBlockCancelReason::Caller,
                );
            }
        }
    }
}

impl RenderArtifactBlockLoaderInner {
    pub(super) fn schedule_decode(
        self: &Arc<Self>,
        entry: Arc<RenderArtifactBlockEntry>,
        encoded: Arc<[u8]>,
    ) {
        let key = RenderArtifactDecodeKey::from_descriptor(entry.descriptor());
        let task_id = {
            let mut registry = self.lock_registry();
            if !registry.accepting
                || !registry
                    .entries
                    .get(&key)
                    .is_some_and(|current| Arc::ptr_eq(current, &entry))
            {
                drop(registry);
                self.cancel_entry(&entry, RenderArtifactBlockCancelReason::OwnerClosed);
                return;
            }
            match take_task_id(&mut registry) {
                Some(task_id) => task_id,
                None => {
                    drop(registry);
                    self.fail_entry(
                        &entry,
                        RenderArtifactBlockFailure::new(
                            RenderArtifactBlockFailureCode::DecodeAdmissionFailed,
                            "render artifact block task identifier space exhausted",
                        ),
                    );
                    return;
                }
            }
        };
        let entry_for_work = Arc::clone(&entry);
        let metrics = Arc::clone(&self.metrics);
        let task = self.scope.submit(
            TaskDescriptor::new(
                TaskId::new(task_id),
                TaskPoolKind::AsyncCompute,
                "render-artifact-block-decode",
            ),
            move |cancellation| {
                run_decode_task(entry_for_work, encoded, metrics, cancellation);
            },
        );
        match task {
            Ok(task) => {
                self.metrics
                    .submitted_decode_tasks
                    .fetch_add(1, Ordering::Relaxed);
                entry.install_task(task);
            }
            Err(error) => self.fail_entry(
                &entry,
                RenderArtifactBlockFailure::new(
                    RenderArtifactBlockFailureCode::DecodeAdmissionFailed,
                    error.to_string(),
                ),
            ),
        }
    }

    pub(super) fn maintain_deadlines(&self, now: Instant) -> RenderArtifactBlockMaintenanceReport {
        let mut registry = self.lock_registry();
        let mut expired_tickets = 0_usize;
        let mut cancelled_entries = Vec::new();
        loop {
            let Some(&(deadline, ticket_id)) = registry.deadlines.first() else {
                break;
            };
            if deadline > now {
                break;
            }
            let Some(ticket) = remove_registered_ticket(&mut registry, ticket_id) else {
                registry.deadlines.remove(&(deadline, ticket_id));
                continue;
            };
            if !ticket.registration.transition_from_active(TICKET_EXPIRED) {
                continue;
            }
            expired_tickets += 1;
            if let Some(entry) = ticket.entry.upgrade() {
                if entry.remove_ticket() {
                    remove_entry(&mut registry, &entry);
                    cancelled_entries.push(entry);
                }
            }
        }
        drop(registry);
        let mut cancelled_count = 0_usize;
        for entry in cancelled_entries {
            if self.cancel_entry(&entry, RenderArtifactBlockCancelReason::Deadline) {
                cancelled_count += 1;
            }
        }
        atomic_add(&self.metrics.expired_tickets, expired_tickets as u64);
        RenderArtifactBlockMaintenanceReport {
            expired_tickets,
            cancelled_entries: cancelled_count,
        }
    }

    fn release_ticket(
        &self,
        ticket_id: u64,
        entry: &Arc<RenderArtifactBlockEntry>,
        reason: RenderArtifactBlockCancelReason,
    ) {
        let should_cancel = {
            let mut registry = self.lock_registry();
            let Some(_ticket) = remove_registered_ticket(&mut registry, ticket_id) else {
                return;
            };
            let last_ticket = entry.remove_ticket();
            if last_ticket {
                remove_entry(&mut registry, entry);
            }
            last_ticket
        };
        if should_cancel {
            self.cancel_entry(entry, reason);
        }
    }

    fn close(&self) -> RenderArtifactBlockLoaderCloseReport {
        let (entries, cancelled_tickets, released_retained_bytes) = {
            let mut registry = self.lock_registry();
            if !registry.accepting {
                return RenderArtifactBlockLoaderCloseReport::default();
            }
            registry.accepting = false;
            self.scope.close_admission();
            for ticket in registry.tickets.values() {
                ticket
                    .registration
                    .transition_from_active(TICKET_OWNER_CLOSED);
            }
            let cancelled_tickets = registry.tickets.len();
            let released_retained_bytes = registry.retained_bytes;
            let entries: Vec<Arc<RenderArtifactBlockEntry>> =
                registry.entries.drain().map(|(_, entry)| entry).collect();
            registry.tickets.clear();
            registry.deadlines.clear();
            registry.io_frontier.clear();
            registry.retained_bytes = 0;
            (entries, cancelled_tickets, released_retained_bytes)
        };
        let mut cancelled_entries = 0_usize;
        for entry in entries {
            if self.cancel_entry(&entry, RenderArtifactBlockCancelReason::OwnerClosed) {
                cancelled_entries += 1;
            }
        }
        RenderArtifactBlockLoaderCloseReport {
            cancelled_entries,
            cancelled_tickets,
            released_retained_bytes,
        }
    }

    fn diagnostics(&self) -> RenderArtifactBlockLoaderDiagnostics {
        let registry = self.lock_registry();
        RenderArtifactBlockLoaderDiagnostics {
            live_entries: registry.entries.len(),
            live_tickets: registry.tickets.len(),
            queued_io_entries: registry.io_frontier.queued_len(),
            retained_bytes: registry.retained_bytes,
            submitted_io_tasks: self.metrics.submitted_io_tasks.load(Ordering::Relaxed),
            submitted_decode_tasks: self.metrics.submitted_decode_tasks.load(Ordering::Relaxed),
            merged_requests: self.metrics.merged_requests.load(Ordering::Relaxed),
            ready_entries: self.metrics.ready_entries.load(Ordering::Relaxed),
            failed_entries: self.metrics.failed_entries.load(Ordering::Relaxed),
            cancelled_entries: self.metrics.cancelled_entries.load(Ordering::Relaxed),
            expired_tickets: self.metrics.expired_tickets.load(Ordering::Relaxed),
            encoded_bytes_read: self.metrics.encoded_bytes_read.load(Ordering::Relaxed),
            decoded_bytes: self.metrics.decoded_bytes.load(Ordering::Relaxed),
            io_worker_wall: std::time::Duration::from_nanos(
                self.metrics.io_worker_wall_ns.load(Ordering::Relaxed),
            ),
            decode_worker_wall: std::time::Duration::from_nanos(
                self.metrics.decode_worker_wall_ns.load(Ordering::Relaxed),
            ),
        }
    }

    fn cancel_entry(
        &self,
        entry: &RenderArtifactBlockEntry,
        reason: RenderArtifactBlockCancelReason,
    ) -> bool {
        let cancelled = entry.cancel(reason);
        if cancelled {
            self.metrics
                .cancelled_entries
                .fetch_add(1, Ordering::Relaxed);
        }
        cancelled
    }

    fn fail_entry(&self, entry: &RenderArtifactBlockEntry, failure: RenderArtifactBlockFailure) {
        if entry.fail(failure) {
            self.metrics.failed_entries.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub(super) fn lock_registry(&self) -> MutexGuard<'_, RenderArtifactBlockRegistry> {
        self.registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for RenderArtifactBlockLoaderInner {
    fn drop(&mut self) {
        self.scope.close_admission();
        let registry = self
            .registry
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        registry.accepting = false;
        for ticket in registry.tickets.values() {
            ticket
                .registration
                .transition_from_active(TICKET_OWNER_CLOSED);
        }
        registry.tickets.clear();
        registry.deadlines.clear();
        registry.io_frontier.clear();
        registry.retained_bytes = 0;
        for entry in registry.entries.drain().map(|(_, entry)| entry) {
            entry.cancel(RenderArtifactBlockCancelReason::OwnerClosed);
        }
    }
}
