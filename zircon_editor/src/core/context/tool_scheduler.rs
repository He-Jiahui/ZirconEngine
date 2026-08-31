use std::collections::{BTreeSet, VecDeque};
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use zircon_runtime_interface::ui::dispatch::UiWindowId;

use crate::core::editor_message::{
    EditorMessage, EditorMessageDispatchReport, EditorMessagePayload, EditorTopic,
    SharedEditorMessageBus, ToolMessage,
};
use crate::core::tools::{
    AcquireOutcome, ReleaseOutcome, ToolAuthorityState, ToolDefinitionId,
    ToolInputCaptureDisposition, ToolInputCaptureEndOutcome, ToolInputCaptureHandle,
    ToolInputCaptureId, ToolInputCaptureOutcome, ToolInputCaptureOwner, ToolInputCaptureRequest,
    ToolInputSource, ToolInstanceId, ToolLeaseHandle, ToolLeaseId, ToolLifecycleEvent,
    ToolOwnerGeneration, ToolOwnerRevokeOutcome, ToolQueueLimits, ToolRequestId,
    ToolResourceCatalog, ToolResourceKey, ToolResourceKindDeclaration,
    ToolResourceKindRegistration, ToolResourceSet, ToolScheduleReport, ToolScheduler,
    ToolShutdownOutcome, ToolTransitionBatch, ToolTransitionRevision, WithdrawOutcome,
};

mod error;
mod observation;

pub use error::ToolSchedulerServiceError;
pub use observation::{
    DEFAULT_MAX_ACTIVE_TOOL_OWNER_GENERATIONS, DEFAULT_MAX_TOOL_TRANSITION_JOURNAL_BATCHES,
    ToolSchedulerLimits, ToolSchedulerLimitsError, ToolSchedulerSnapshot, ToolTransitionCursor,
    ToolTransitionRead, ToolTransitionReadError,
};

/// Thread-safe owner of the one editor-wide exclusive-resource scheduler.
#[derive(Clone, Debug)]
pub struct ToolSchedulerService {
    authority: Arc<Mutex<ToolSchedulerAuthority>>,
    dispatcher: Arc<Mutex<()>>,
    next_instance_ordinal: Arc<AtomicU64>,
    bus: SharedEditorMessageBus,
    topic: EditorTopic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToolSchedulerDeliveryHealth {
    committed_revision: ToolTransitionRevision,
    dispatched_revision: ToolTransitionRevision,
    delivered_batches: u64,
    unobserved_batches: u64,
    dropped_deliveries: u64,
    backpressured_deliveries: u64,
    dispatch_errors: u64,
}

impl ToolSchedulerDeliveryHealth {
    pub const fn committed_revision(self) -> ToolTransitionRevision {
        self.committed_revision
    }

    pub const fn dispatched_revision(self) -> ToolTransitionRevision {
        self.dispatched_revision
    }

    pub const fn delivered_batches(self) -> u64 {
        self.delivered_batches
    }

    pub const fn unobserved_batches(self) -> u64 {
        self.unobserved_batches
    }

    pub const fn dropped_deliveries(self) -> u64 {
        self.dropped_deliveries
    }

    pub const fn backpressured_deliveries(self) -> u64 {
        self.backpressured_deliveries
    }

    pub const fn dispatch_errors(self) -> u64 {
        self.dispatch_errors
    }

    pub const fn requires_resync(self) -> bool {
        self.committed_revision != self.dispatched_revision
            || self.unobserved_batches > 0
            || self.dropped_deliveries > 0
            || self.backpressured_deliveries > 0
            || self.dispatch_errors > 0
    }
}

#[derive(Debug)]
struct ToolSchedulerAuthority {
    scheduler: ToolScheduler,
    resource_catalog: ToolResourceCatalog,
    state: ToolAuthorityState,
    active_owner_generations: BTreeSet<ToolOwnerGeneration>,
    next_owner_generation: Option<ToolOwnerGeneration>,
    revision: ToolTransitionRevision,
    outbox: VecDeque<ToolTransitionBatch>,
    journal: VecDeque<ToolTransitionBatch>,
    limits: ToolSchedulerLimits,
    delivery_health: ToolSchedulerDeliveryHealth,
}

impl ToolSchedulerAuthority {
    fn new(limits: ToolSchedulerLimits) -> Self {
        Self {
            scheduler: ToolScheduler::new(limits.queue_limits()),
            resource_catalog: ToolResourceCatalog::new(),
            state: ToolAuthorityState::Open,
            active_owner_generations: BTreeSet::from([ToolOwnerGeneration::BUILTIN]),
            next_owner_generation: ToolOwnerGeneration::BUILTIN.checked_next(),
            revision: ToolTransitionRevision::INITIAL,
            outbox: VecDeque::new(),
            journal: VecDeque::new(),
            limits,
            delivery_health: ToolSchedulerDeliveryHealth::default(),
        }
    }

    fn commit(&mut self, revision: ToolTransitionRevision, events: &[ToolLifecycleEvent]) {
        if events.is_empty() {
            return;
        }
        self.revision = revision;
        let batch = ToolTransitionBatch::new(self.revision, events.to_vec());
        self.outbox.push_back(batch.clone());
        self.journal.push_back(batch);
        while self.journal.len() > self.limits.max_transition_journal_batches() {
            self.journal.pop_front();
        }
        self.delivery_health.committed_revision = self.revision;
    }

    fn mark_faulted(&mut self) {
        if self.state == ToolAuthorityState::Faulted {
            return;
        }
        let previous = self.state;
        self.state = ToolAuthorityState::Faulted;
        let Some(revision) = self.revision.checked_next() else {
            return;
        };
        self.commit(
            revision,
            &[ToolLifecycleEvent::AuthorityStateChanged {
                previous,
                current: ToolAuthorityState::Faulted,
            }],
        );
    }

    fn snapshot(&self) -> ToolSchedulerSnapshot {
        ToolSchedulerSnapshot::new(
            self.revision,
            self.state,
            self.active_owner_generations.iter().copied().collect(),
            self.resource_catalog.registrations().cloned().collect(),
            self.scheduler.snapshot(),
        )
    }

    fn read_transitions(
        &self,
        cursor: ToolTransitionCursor,
    ) -> Result<ToolTransitionRead, ToolTransitionReadError> {
        let current = ToolTransitionCursor::from_revision(self.revision);
        if cursor > current {
            return Err(ToolTransitionReadError::FutureCursor {
                requested: cursor,
                current,
            });
        }
        if cursor == current {
            return Ok(ToolTransitionRead::Current { cursor: current });
        }

        let Some(oldest_available_revision) =
            self.journal.front().map(ToolTransitionBatch::revision)
        else {
            return Ok(ToolTransitionRead::ResyncRequired {
                requested: cursor,
                oldest_available_revision: self.revision,
                snapshot: self.snapshot(),
            });
        };
        if cursor.revision().value() < oldest_available_revision.value().saturating_sub(1) {
            return Ok(ToolTransitionRead::ResyncRequired {
                requested: cursor,
                oldest_available_revision,
                snapshot: self.snapshot(),
            });
        }

        let batches = self
            .journal
            .iter()
            .filter(|batch| batch.revision() > cursor.revision())
            .cloned()
            .collect();
        Ok(ToolTransitionRead::Available {
            from_exclusive: cursor,
            through: current,
            batches,
        })
    }

    fn record_dispatch(
        &mut self,
        revision: ToolTransitionRevision,
        report: &EditorMessageDispatchReport,
    ) {
        self.delivery_health.dispatched_revision = revision;
        if report.delivered().is_empty() && report.coalesced().is_empty() {
            self.delivery_health.unobserved_batches =
                self.delivery_health.unobserved_batches.saturating_add(1);
        } else {
            self.delivery_health.delivered_batches =
                self.delivery_health.delivered_batches.saturating_add(1);
        }
        self.delivery_health.dropped_deliveries = self
            .delivery_health
            .dropped_deliveries
            .saturating_add(report.dropped().len() as u64);
        self.delivery_health.backpressured_deliveries = self
            .delivery_health
            .backpressured_deliveries
            .saturating_add(report.backpressured().len() as u64);
        if report.error().is_some() {
            self.delivery_health.dispatch_errors =
                self.delivery_health.dispatch_errors.saturating_add(1);
        }
    }
}

impl ToolSchedulerService {
    pub fn new(bus: SharedEditorMessageBus) -> Self {
        Self::with_limits(bus, ToolSchedulerLimits::default())
    }

    pub fn with_queue_limits(bus: SharedEditorMessageBus, queue_limits: ToolQueueLimits) -> Self {
        Self::with_limits(bus, ToolSchedulerLimits::with_default_journal(queue_limits))
    }

    pub fn with_limits(bus: SharedEditorMessageBus, limits: ToolSchedulerLimits) -> Self {
        Self {
            authority: Arc::new(Mutex::new(ToolSchedulerAuthority::new(limits))),
            dispatcher: Arc::new(Mutex::new(())),
            next_instance_ordinal: Arc::new(AtomicU64::new(1)),
            bus,
            topic: EditorTopic::tool(),
        }
    }

    pub fn acquire(
        &self,
        tool: ToolInstanceId,
        resources: ToolResourceSet,
    ) -> Result<ToolScheduleReport<AcquireOutcome>, ToolSchedulerServiceError> {
        let report = {
            let mut authority = self.lock_authority();
            Self::ensure_accepting_requests(&authority)?;
            let generation = tool.owner_generation();
            if !authority.active_owner_generations.contains(&generation) {
                return Err(ToolSchedulerServiceError::OwnerGenerationUnavailable { generation });
            }
            authority.resource_catalog.validate(&resources)?;
            Self::commit_locked_transition(&mut authority, move |scheduler| {
                scheduler.acquire(tool, resources)
            })?
        };
        self.dispatch_outbox();
        Ok(report)
    }

    pub fn release(
        &self,
        lease_id: ToolLeaseId,
    ) -> Result<ToolScheduleReport<ReleaseOutcome>, ToolSchedulerServiceError> {
        self.execute_transition(|scheduler| scheduler.release(lease_id))
    }

    pub fn withdraw(
        &self,
        request_id: ToolRequestId,
    ) -> Result<ToolScheduleReport<WithdrawOutcome>, ToolSchedulerServiceError> {
        self.execute_transition(|scheduler| scheduler.withdraw(request_id))
    }

    pub fn begin_input_capture(
        &self,
        request: ToolInputCaptureRequest,
    ) -> Result<ToolScheduleReport<ToolInputCaptureOutcome>, ToolSchedulerServiceError> {
        self.execute_owner_admission_transition(request.owner().generation(), move |scheduler| {
            scheduler.begin_input_capture(request)
        })
    }

    pub fn end_input_capture(
        &self,
        capture_id: ToolInputCaptureId,
        owner: &ToolInputCaptureOwner,
        disposition: ToolInputCaptureDisposition,
    ) -> Result<ToolScheduleReport<ToolInputCaptureEndOutcome>, ToolSchedulerServiceError> {
        self.execute_transition(|scheduler| {
            scheduler.end_input_capture(capture_id, owner, disposition)
        })
    }

    pub fn release_input_window_on_focus_loss(
        &self,
        window_id: &UiWindowId,
    ) -> Result<ToolScheduleReport<Box<[ToolInputCaptureHandle]>>, ToolSchedulerServiceError> {
        self.execute_transition(|scheduler| scheduler.release_input_window_on_focus_loss(window_id))
    }

    pub fn active_input_capture(&self, source: &ToolInputSource) -> Option<ToolInputCaptureHandle> {
        self.lock_authority()
            .scheduler
            .active_input_capture(source)
            .cloned()
    }

    pub fn holder(&self, resource: &ToolResourceKey) -> Option<ToolLeaseHandle> {
        self.lock_authority().scheduler.holder(resource).cloned()
    }

    pub fn delivery_health(&self) -> ToolSchedulerDeliveryHealth {
        self.lock_authority().delivery_health
    }

    pub fn snapshot(&self) -> ToolSchedulerSnapshot {
        self.lock_authority().snapshot()
    }

    pub fn authority_state(&self) -> ToolAuthorityState {
        self.lock_authority().state
    }

    pub fn quiesce(&self) -> Result<ToolAuthorityState, ToolSchedulerServiceError> {
        let changed = {
            let mut authority = self.lock_authority();
            if authority.state != ToolAuthorityState::Open {
                return Ok(authority.state);
            }
            let revision = authority
                .revision
                .checked_next()
                .ok_or(ToolSchedulerServiceError::TransitionRevisionExhausted)?;
            let previous = authority.state;
            let current = ToolAuthorityState::Quiescing;
            authority.state = current;
            authority.commit(
                revision,
                &[ToolLifecycleEvent::AuthorityStateChanged { previous, current }],
            );
            current
        };
        self.dispatch_outbox();
        Ok(changed)
    }

    pub fn close(
        &self,
    ) -> Result<ToolScheduleReport<ToolShutdownOutcome>, ToolSchedulerServiceError> {
        let report = {
            let mut authority = self.lock_authority();
            if authority.state == ToolAuthorityState::Closed {
                return Ok(ToolScheduleReport::new(
                    ToolShutdownOutcome::default(),
                    Vec::new(),
                ));
            }
            let revision = authority
                .revision
                .checked_next()
                .ok_or(ToolSchedulerServiceError::TransitionRevisionExhausted)?;
            let mut events = Vec::new();
            if authority.state == ToolAuthorityState::Open {
                events.push(ToolLifecycleEvent::AuthorityStateChanged {
                    previous: ToolAuthorityState::Open,
                    current: ToolAuthorityState::Quiescing,
                });
                authority.state = ToolAuthorityState::Quiescing;
            }
            if authority.state != ToolAuthorityState::Draining {
                events.push(ToolLifecycleEvent::AuthorityStateChanged {
                    previous: authority.state,
                    current: ToolAuthorityState::Draining,
                });
                authority.state = ToolAuthorityState::Draining;
            }
            let (outcome, shutdown_events) = authority.scheduler.shutdown().into_parts();
            events.extend(shutdown_events);
            authority.active_owner_generations.clear();
            authority.resource_catalog.remove_extension_registrations();
            events.push(ToolLifecycleEvent::AuthorityStateChanged {
                previous: authority.state,
                current: ToolAuthorityState::Closed,
            });
            authority.state = ToolAuthorityState::Closed;
            authority.commit(revision, &events);
            ToolScheduleReport::new(outcome, events)
        };
        self.dispatch_outbox();
        Ok(report)
    }

    pub fn read_transitions(
        &self,
        cursor: ToolTransitionCursor,
    ) -> Result<ToolTransitionRead, ToolTransitionReadError> {
        self.lock_authority().read_transitions(cursor)
    }

    pub(crate) fn register_owner_generation(
        &self,
        resource_kinds: impl IntoIterator<Item = ToolResourceKindDeclaration>,
    ) -> Result<ToolScheduleReport<ToolOwnerGeneration>, ToolSchedulerServiceError> {
        let mut resource_kinds = resource_kinds.into_iter().collect::<Vec<_>>();
        resource_kinds.sort_unstable_by(|left, right| left.kind().cmp(right.kind()));
        let report = {
            let mut authority = self.lock_authority();
            Self::ensure_accepting_requests(&authority)?;
            let revision = authority
                .revision
                .checked_next()
                .ok_or(ToolSchedulerServiceError::TransitionRevisionExhausted)?;
            if authority.active_owner_generations.len()
                >= authority.limits.max_active_owner_generations()
            {
                return Err(ToolSchedulerServiceError::OwnerGenerationCapacityReached {
                    max_active_owner_generations: authority.limits.max_active_owner_generations(),
                });
            }
            let generation = authority
                .next_owner_generation
                .ok_or(ToolSchedulerServiceError::OwnerGenerationIdentityExhausted)?;
            let mut resource_catalog = authority.resource_catalog.clone();
            let mut registrations = Vec::with_capacity(resource_kinds.len());
            for declaration in resource_kinds {
                let registration =
                    ToolResourceKindRegistration::from_declaration(declaration, generation);
                resource_catalog.register(registration.clone())?;
                registrations.push(registration);
            }
            authority.next_owner_generation = generation.checked_next();
            authority.active_owner_generations.insert(generation);
            authority.resource_catalog = resource_catalog;
            let mut events = Vec::with_capacity(1 + registrations.len());
            events.push(ToolLifecycleEvent::OwnerGenerationRegistered { generation });
            events.extend(
                registrations.into_iter().map(|registration| {
                    ToolLifecycleEvent::ResourceKindRegistered { registration }
                }),
            );
            authority.commit(revision, &events);
            ToolScheduleReport::new(generation, events)
        };
        self.dispatch_outbox();
        Ok(report)
    }

    pub(crate) fn revoke_owner_generation(
        &self,
        generation: ToolOwnerGeneration,
    ) -> Result<ToolScheduleReport<ToolOwnerRevokeOutcome>, ToolSchedulerServiceError> {
        let report = {
            let mut authority = self.lock_authority();
            Self::ensure_cleanup_mutation_allowed(&authority)?;
            if generation == ToolOwnerGeneration::BUILTIN {
                return Ok(ToolScheduleReport::new(
                    ToolOwnerRevokeOutcome::BuiltinProtected,
                    Vec::new(),
                ));
            }
            if !authority.active_owner_generations.contains(&generation) {
                return Ok(ToolScheduleReport::new(
                    ToolOwnerRevokeOutcome::NotRegistered { generation },
                    Vec::new(),
                ));
            }
            let revision = authority
                .revision
                .checked_next()
                .ok_or(ToolSchedulerServiceError::TransitionRevisionExhausted)?;
            authority.active_owner_generations.remove(&generation);
            let revoked_resource_kinds = authority.resource_catalog.kinds_for_owner(generation);
            let (outcome, mut events) = authority
                .scheduler
                .revoke_owner_generation(generation, &revoked_resource_kinds)
                .into_parts();
            authority.resource_catalog.remove_owner(generation);
            if !revoked_resource_kinds.is_empty() {
                events.push(ToolLifecycleEvent::ResourceKindsRevoked {
                    owner_generation: generation,
                    kinds: revoked_resource_kinds,
                });
            }
            events.push(ToolLifecycleEvent::OwnerGenerationRevoked { generation });
            let report = ToolScheduleReport::new(outcome, events);
            authority.commit(revision, report.events());
            report
        };
        self.dispatch_outbox();
        Ok(report)
    }

    pub(crate) fn allocate_instance_id(
        &self,
        definition_id: &ToolDefinitionId,
        owner_generation: ToolOwnerGeneration,
    ) -> Result<ToolInstanceId, ToolSchedulerServiceError> {
        let authority = self.lock_authority();
        Self::ensure_accepting_requests(&authority)?;
        if !authority
            .active_owner_generations
            .contains(&owner_generation)
        {
            return Err(ToolSchedulerServiceError::OwnerGenerationUnavailable {
                generation: owner_generation,
            });
        }
        let ordinal = self
            .next_instance_ordinal
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| ToolSchedulerServiceError::ToolInstanceIdentityExhausted)?;
        let ordinal = NonZeroU64::new(ordinal)
            .ok_or(ToolSchedulerServiceError::ToolInstanceIdentityExhausted)?;
        drop(authority);
        Ok(ToolInstanceId::new(
            definition_id.clone(),
            owner_generation,
            ordinal,
        ))
    }

    fn execute_owner_admission_transition<O>(
        &self,
        generation: ToolOwnerGeneration,
        transition: impl FnOnce(&mut ToolScheduler) -> ToolScheduleReport<O>,
    ) -> Result<ToolScheduleReport<O>, ToolSchedulerServiceError> {
        let report = {
            let mut authority = self.lock_authority();
            Self::ensure_accepting_requests(&authority)?;
            if !authority.active_owner_generations.contains(&generation) {
                return Err(ToolSchedulerServiceError::OwnerGenerationUnavailable { generation });
            }
            Self::commit_locked_transition(&mut authority, transition)?
        };
        self.dispatch_outbox();
        Ok(report)
    }

    fn execute_transition<O>(
        &self,
        transition: impl FnOnce(&mut ToolScheduler) -> ToolScheduleReport<O>,
    ) -> Result<ToolScheduleReport<O>, ToolSchedulerServiceError> {
        let report = {
            let mut authority = self.lock_authority();
            Self::ensure_cleanup_mutation_allowed(&authority)?;
            Self::commit_locked_transition(&mut authority, transition)?
        };
        self.dispatch_outbox();
        Ok(report)
    }

    #[cfg(test)]
    fn commit_transition<O>(
        &self,
        transition: impl FnOnce(&mut ToolScheduler) -> ToolScheduleReport<O>,
    ) -> Result<ToolScheduleReport<O>, ToolSchedulerServiceError> {
        let mut authority = self.lock_authority();
        Self::commit_locked_transition(&mut authority, transition)
    }

    fn commit_locked_transition<O>(
        authority: &mut ToolSchedulerAuthority,
        transition: impl FnOnce(&mut ToolScheduler) -> ToolScheduleReport<O>,
    ) -> Result<ToolScheduleReport<O>, ToolSchedulerServiceError> {
        let revision = authority
            .revision
            .checked_next()
            .ok_or(ToolSchedulerServiceError::TransitionRevisionExhausted)?;
        let report = transition(&mut authority.scheduler);
        authority.commit(revision, report.events());
        Ok(report)
    }

    fn ensure_accepting_requests(
        authority: &ToolSchedulerAuthority,
    ) -> Result<(), ToolSchedulerServiceError> {
        if authority.state.accepts_requests() {
            Ok(())
        } else {
            Err(ToolSchedulerServiceError::AuthorityUnavailable {
                state: authority.state,
            })
        }
    }

    fn ensure_cleanup_mutation_allowed(
        authority: &ToolSchedulerAuthority,
    ) -> Result<(), ToolSchedulerServiceError> {
        if matches!(
            authority.state,
            ToolAuthorityState::Open | ToolAuthorityState::Quiescing
        ) {
            Ok(())
        } else {
            Err(ToolSchedulerServiceError::AuthorityUnavailable {
                state: authority.state,
            })
        }
    }

    fn dispatch_outbox(&self) {
        let _dispatcher = match self.dispatcher.lock() {
            Ok(dispatcher) => dispatcher,
            Err(poisoned) => {
                let mut authority = self.lock_authority();
                authority.delivery_health.dispatch_errors =
                    authority.delivery_health.dispatch_errors.saturating_add(1);
                if authority.state != ToolAuthorityState::Closed {
                    authority.mark_faulted();
                }
                drop(authority);
                self.dispatcher.clear_poison();
                poisoned.into_inner()
            }
        };
        loop {
            let batch = {
                let mut authority = self.lock_authority();
                authority.outbox.pop_front()
            };
            let Some(batch) = batch else {
                break;
            };
            let revision = batch.revision();
            let report = self.bus.publish(
                self.topic.clone(),
                EditorMessage::new(EditorMessagePayload::Tool(ToolMessage::Transition(batch))),
            );
            let mut authority = self.lock_authority();
            authority.record_dispatch(revision, &report);
        }
    }

    fn lock_authority(&self) -> MutexGuard<'_, ToolSchedulerAuthority> {
        match self.authority.lock() {
            Ok(authority) => authority,
            Err(poisoned) => {
                let mut authority = poisoned.into_inner();
                authority.mark_faulted();
                self.authority.clear_poison();
                authority
            }
        }
    }
}

#[cfg(test)]
mod tests;
