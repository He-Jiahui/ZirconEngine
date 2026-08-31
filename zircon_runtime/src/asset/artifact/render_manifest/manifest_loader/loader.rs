use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::Instant;

use crate::core::runtime::{EngineTaskGraph, TaskGraphScope};

use super::super::RenderArtifactStore;
use super::contract::{
    RenderArtifactManifestAdmissionError, RenderArtifactManifestCancelReason,
    RenderArtifactManifestIoDispatchBudget, RenderArtifactManifestIoDispatchError,
    RenderArtifactManifestIoDispatchReport, RenderArtifactManifestLoaderCloseReport,
    RenderArtifactManifestLoaderDiagnostics, RenderArtifactManifestLoaderInitError,
    RenderArtifactManifestLoaderLimits, RenderArtifactManifestMaintenanceReport,
    RenderArtifactManifestPoll, RenderArtifactManifestRequest, RenderArtifactManifestRequestKey,
};
use super::state::{
    RenderArtifactManifestEntry, RenderArtifactManifestRegistry, TICKET_ACTIVE,
    TICKET_CALLER_CANCELLED, TICKET_EXPIRED, TICKET_OWNER_CLOSED, TicketRegistration, remove_entry,
    remove_registered_ticket,
};
use super::worker::{RenderArtifactManifestLoaderMetrics, atomic_add};

const ENTRY_METADATA_BYTES: usize = 512;

pub(super) struct RenderArtifactManifestLoaderInner {
    pub(super) store: RenderArtifactStore,
    pub(super) limits: RenderArtifactManifestLoaderLimits,
    pub(super) entry_retained_bytes: usize,
    pub(super) scope: TaskGraphScope,
    pub(super) registry: Mutex<RenderArtifactManifestRegistry>,
    pub(super) metrics: Arc<RenderArtifactManifestLoaderMetrics>,
}

#[derive(Clone)]
pub struct RenderArtifactManifestLoader {
    inner: Arc<RenderArtifactManifestLoaderInner>,
}

pub struct RenderArtifactManifestTicket {
    id: u64,
    entry: Arc<RenderArtifactManifestEntry>,
    registration: Arc<TicketRegistration>,
    loader: Weak<RenderArtifactManifestLoaderInner>,
}

pub struct RenderArtifactManifestTicketBatch {
    tickets: Vec<RenderArtifactManifestTicket>,
}

impl RenderArtifactManifestLoader {
    pub fn new(
        store: RenderArtifactStore,
        limits: RenderArtifactManifestLoaderLimits,
        runtime: &EngineTaskGraph,
    ) -> Result<Self, RenderArtifactManifestLoaderInitError> {
        let entry_retained_bytes = validate_limits(limits)?;
        let scope = runtime.create_scope(
            crate::core::runtime::TaskGraphScopeDescriptor::new("render-artifact-manifest-loader")
                .with_task_capacity(limits.max_entries()),
        )?;
        Ok(Self {
            inner: Arc::new(RenderArtifactManifestLoaderInner {
                store,
                limits,
                entry_retained_bytes,
                scope,
                registry: Mutex::new(RenderArtifactManifestRegistry::new()),
                metrics: Arc::default(),
            }),
        })
    }

    pub fn request(
        &self,
        request: RenderArtifactManifestRequest,
    ) -> Result<RenderArtifactManifestTicket, RenderArtifactManifestAdmissionError> {
        let mut batch = self.inner.request_batch(std::slice::from_ref(&request))?;
        batch
            .tickets
            .pop()
            .ok_or(RenderArtifactManifestAdmissionError::EmptyBatch)
    }

    pub fn request_batch(
        &self,
        requests: &[RenderArtifactManifestRequest],
    ) -> Result<RenderArtifactManifestTicketBatch, RenderArtifactManifestAdmissionError> {
        self.inner.request_batch(requests)
    }

    pub fn dispatch_io(
        &self,
        budget: RenderArtifactManifestIoDispatchBudget,
    ) -> Result<RenderArtifactManifestIoDispatchReport, RenderArtifactManifestIoDispatchError> {
        self.inner.dispatch_io(budget)
    }

    pub fn maintain_deadlines(&self, now: Instant) -> RenderArtifactManifestMaintenanceReport {
        self.inner.maintain_deadlines(now)
    }

    pub fn diagnostics(&self) -> RenderArtifactManifestLoaderDiagnostics {
        self.inner.diagnostics()
    }

    pub fn close(&self) -> RenderArtifactManifestLoaderCloseReport {
        self.inner.close()
    }
}

impl RenderArtifactManifestTicketBatch {
    pub(super) fn new(tickets: Vec<RenderArtifactManifestTicket>) -> Self {
        Self { tickets }
    }

    pub fn tickets(&self) -> &[RenderArtifactManifestTicket] {
        &self.tickets
    }

    pub fn into_tickets(self) -> Vec<RenderArtifactManifestTicket> {
        self.tickets
    }
}

impl RenderArtifactManifestTicket {
    pub(super) fn from_parts(
        id: u64,
        entry: Arc<RenderArtifactManifestEntry>,
        registration: Arc<TicketRegistration>,
        loader: Weak<RenderArtifactManifestLoaderInner>,
    ) -> Self {
        Self {
            id,
            entry,
            registration,
            loader,
        }
    }

    pub const fn id(&self) -> u64 {
        self.id
    }

    pub fn key(&self) -> &RenderArtifactManifestRequestKey {
        self.entry.key()
    }

    pub fn poll(&self) -> RenderArtifactManifestPoll {
        match self.registration.status() {
            TICKET_ACTIVE => self.entry.poll(),
            TICKET_EXPIRED => {
                RenderArtifactManifestPoll::Cancelled(RenderArtifactManifestCancelReason::Deadline)
            }
            TICKET_OWNER_CLOSED => RenderArtifactManifestPoll::Cancelled(
                RenderArtifactManifestCancelReason::OwnerClosed,
            ),
            _ => RenderArtifactManifestPoll::Cancelled(RenderArtifactManifestCancelReason::Caller),
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
                    RenderArtifactManifestCancelReason::Caller,
                );
            }
        }
    }
}

impl Drop for RenderArtifactManifestTicket {
    fn drop(&mut self) {
        if self
            .registration
            .transition_from_active(TICKET_CALLER_CANCELLED)
        {
            if let Some(loader) = self.loader.upgrade() {
                loader.release_ticket(
                    self.id,
                    &self.entry,
                    RenderArtifactManifestCancelReason::Caller,
                );
            }
        }
    }
}

impl RenderArtifactManifestLoaderInner {
    pub(super) fn maintain_deadlines(
        &self,
        now: Instant,
    ) -> RenderArtifactManifestMaintenanceReport {
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
            expired_tickets = expired_tickets.saturating_add(1);
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
            if self.cancel_entry(&entry, RenderArtifactManifestCancelReason::Deadline) {
                cancelled_count = cancelled_count.saturating_add(1);
            }
        }
        atomic_add(&self.metrics.expired_tickets, expired_tickets as u64);
        RenderArtifactManifestMaintenanceReport {
            expired_tickets,
            cancelled_entries: cancelled_count,
        }
    }

    fn release_ticket(
        &self,
        ticket_id: u64,
        entry: &Arc<RenderArtifactManifestEntry>,
        reason: RenderArtifactManifestCancelReason,
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

    fn close(&self) -> RenderArtifactManifestLoaderCloseReport {
        let (entries, cancelled_tickets, released_reserved_bytes) = {
            let mut registry = self.lock_registry();
            if !registry.accepting {
                return RenderArtifactManifestLoaderCloseReport::default();
            }
            registry.accepting = false;
            self.scope.close_admission();
            for ticket in registry.tickets.values() {
                ticket
                    .registration
                    .transition_from_active(TICKET_OWNER_CLOSED);
            }
            let cancelled_tickets = registry.tickets.len();
            let released_reserved_bytes = registry.reserved_retained_bytes;
            let entries: Vec<Arc<RenderArtifactManifestEntry>> =
                registry.entries.drain().map(|(_, entry)| entry).collect();
            registry.tickets.clear();
            registry.deadlines.clear();
            registry.io_frontier.clear();
            registry.reserved_retained_bytes = 0;
            (entries, cancelled_tickets, released_reserved_bytes)
        };
        let mut cancelled_entries = 0_usize;
        for entry in entries {
            if self.cancel_entry(&entry, RenderArtifactManifestCancelReason::OwnerClosed) {
                cancelled_entries = cancelled_entries.saturating_add(1);
            }
        }
        RenderArtifactManifestLoaderCloseReport {
            cancelled_entries,
            cancelled_tickets,
            released_reserved_bytes,
        }
    }

    fn diagnostics(&self) -> RenderArtifactManifestLoaderDiagnostics {
        let registry = self.lock_registry();
        RenderArtifactManifestLoaderDiagnostics {
            live_entries: registry.entries.len(),
            live_tickets: registry.tickets.len(),
            queued_io_entries: registry.io_frontier.queued_len(),
            reserved_retained_bytes: registry.reserved_retained_bytes,
            submitted_io_tasks: self.metrics.submitted_io_tasks.load(Ordering::Relaxed),
            merged_requests: self.metrics.merged_requests.load(Ordering::Relaxed),
            ready_entries: self.metrics.ready_entries.load(Ordering::Relaxed),
            failed_entries: self.metrics.failed_entries.load(Ordering::Relaxed),
            cancelled_entries: self.metrics.cancelled_entries.load(Ordering::Relaxed),
            expired_tickets: self.metrics.expired_tickets.load(Ordering::Relaxed),
            io_worker_wall: std::time::Duration::from_nanos(
                self.metrics.io_worker_wall_ns.load(Ordering::Relaxed),
            ),
        }
    }

    fn cancel_entry(
        &self,
        entry: &RenderArtifactManifestEntry,
        reason: RenderArtifactManifestCancelReason,
    ) -> bool {
        let cancelled = entry.cancel(reason);
        if cancelled {
            self.metrics
                .cancelled_entries
                .fetch_add(1, Ordering::Relaxed);
        }
        cancelled
    }

    pub(super) fn lock_registry(&self) -> MutexGuard<'_, RenderArtifactManifestRegistry> {
        self.registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for RenderArtifactManifestLoaderInner {
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
        registry.reserved_retained_bytes = 0;
        for entry in registry.entries.drain().map(|(_, entry)| entry) {
            entry.cancel(RenderArtifactManifestCancelReason::OwnerClosed);
        }
    }
}

fn validate_limits(
    limits: RenderArtifactManifestLoaderLimits,
) -> Result<usize, RenderArtifactManifestLoaderInitError> {
    for (name, value) in [
        ("max_entries", limits.max_entries()),
        ("max_total_tickets", limits.max_total_tickets()),
        ("max_tickets_per_entry", limits.max_tickets_per_entry()),
        ("max_retained_bytes", limits.max_retained_bytes()),
    ] {
        if value == 0 {
            return Err(RenderArtifactManifestLoaderInitError::ZeroLimit { limit: name });
        }
    }
    for (name, value) in [
        (
            "max_manifest_bytes",
            limits.store_limits().max_manifest_bytes(),
        ),
        (
            "max_encoded_block_bytes",
            limits.store_limits().max_encoded_block_bytes(),
        ),
    ] {
        if value == 0 {
            return Err(RenderArtifactManifestLoaderInitError::ZeroLimit { limit: name });
        }
    }
    let manifest_bytes = usize::try_from(limits.store_limits().max_manifest_bytes())
        .map_err(|_| RenderArtifactManifestLoaderInitError::RetainedBytesQuoteOverflow)?;
    let required = manifest_bytes
        .checked_add(ENTRY_METADATA_BYTES)
        .ok_or(RenderArtifactManifestLoaderInitError::RetainedBytesQuoteOverflow)?;
    if required > limits.max_retained_bytes() {
        return Err(
            RenderArtifactManifestLoaderInitError::RetainedBytesCapacityTooSmall {
                required,
                capacity: limits.max_retained_bytes(),
            },
        );
    }
    Ok(required)
}
