use std::collections::{HashMap, HashSet};

use crate::core::resource::{
    ResourceKind, ResourceManagementGeneration, ResourceReadinessGeneration, UntypedResourceHandle,
};
use crate::graphics::scene::render_scene::RenderSceneResourceReferenceDelta;
use zr_rhi::{SubmissionStatus, SubmissionTicket};

use super::gpu_residency::{
    RenderAssetGpuResidencyLimits, RenderAssetGpuResidencyState,
    RenderAssetGpuRetirementBackpressure,
};
use super::gpu_upload::{RenderAssetGpuArtifact, RenderAssetGpuUploadLease};
use super::{
    RenderAssetDemandGeneration, RenderAssetDeviceEpoch, RenderAssetResidencyAdmissionError,
    RenderAssetResidencyMutation, RenderAssetResidencyMutationStats, RenderAssetResidencyRelease,
    RenderAssetResidencyReleaseKind, RenderAssetResidencyRoute, RenderAssetResidencyScope,
    RenderAssetResidencyState, RenderAssetResidencyTicket, RenderAssetResidencyTicketId,
    RenderAssetResidencyTransitionError,
};

pub(super) mod device_recovery;
mod ticket_issuance;

#[derive(Clone, Copy, Debug)]
struct RenderAssetResidencyTicketSeed {
    resource: UntypedResourceHandle,
    asset_revision: u64,
    readiness_generation: u64,
    dependency_revision: u64,
    demand_generation: RenderAssetDemandGeneration,
    device: RenderAssetDeviceEpoch,
    scope: RenderAssetResidencyScope,
    route: RenderAssetResidencyRoute,
}

impl RenderAssetResidencyTicketSeed {
    fn issue(self, id: RenderAssetResidencyTicketId) -> RenderAssetResidencyTicket {
        RenderAssetResidencyTicket::from_parts(
            id,
            self.resource,
            self.asset_revision,
            self.readiness_generation,
            self.dependency_revision,
            self.demand_generation,
            self.device,
            self.scope,
            self.route,
        )
    }

    fn matches(self, ticket: RenderAssetResidencyTicket) -> bool {
        self.resource == ticket.resource()
            && self.asset_revision == ticket.asset_revision()
            && self.readiness_generation == ticket.readiness_generation()
            && self.dependency_revision == ticket.dependency_revision()
            && self.demand_generation == ticket.demand_generation()
            && self.device == ticket.device()
            && self.scope == ticket.scope()
            && self.route == ticket.route()
    }
}

#[derive(Clone, Copy, Debug)]
struct PendingResidency {
    ticket: RenderAssetResidencyTicket,
    state: RenderAssetResidencyState,
    submission: Option<SubmissionTicket>,
}

#[derive(Clone, Copy, Debug)]
struct ActiveResidency {
    ticket: RenderAssetResidencyTicket,
    submission: SubmissionTicket,
}

pub(super) struct RenderAssetResidencyEntry {
    reference_count: usize,
    pending: Option<PendingResidency>,
    pub(super) pending_upload: Option<RenderAssetGpuUploadLease>,
    active: Option<ActiveResidency>,
    pub(super) active_artifact: Option<RenderAssetGpuArtifact>,
}

#[derive(Clone, Copy, Debug)]
struct PreparedReferenceChange {
    resource: UntypedResourceHandle,
    next_reference_count: usize,
    request_seed: Option<RenderAssetResidencyTicketSeed>,
    request: Option<RenderAssetResidencyTicket>,
}

#[derive(Clone, Copy, Debug)]
struct PreparedReconciliation {
    resource: UntypedResourceHandle,
    cancel_pending: bool,
    request_seed: Option<RenderAssetResidencyTicketSeed>,
    request: Option<RenderAssetResidencyTicket>,
}

pub(crate) struct RenderAssetResidencyManager {
    pub(super) entries: HashMap<UntypedResourceHandle, RenderAssetResidencyEntry>,
    pub(super) gpu: RenderAssetGpuResidencyState,
    next_ticket_id: u64,
}

impl Default for RenderAssetResidencyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderAssetResidencyManager {
    pub(crate) fn new() -> Self {
        Self::new_with_gpu_limits(RenderAssetGpuResidencyLimits::default())
    }

    pub(crate) fn new_with_gpu_limits(limits: RenderAssetGpuResidencyLimits) -> Self {
        Self {
            entries: HashMap::new(),
            gpu: RenderAssetGpuResidencyState::new(limits),
            next_ticket_id: 1,
        }
    }

    pub(crate) fn apply_scene_reference_deltas(
        &mut self,
        deltas: &[RenderSceneResourceReferenceDelta],
        management: &ResourceManagementGeneration,
        readiness: &ResourceReadinessGeneration,
        device: RenderAssetDeviceEpoch,
        demand_generation: RenderAssetDemandGeneration,
    ) -> Result<RenderAssetResidencyMutation, RenderAssetResidencyAdmissionError> {
        self.ensure_device_epoch_admission(device)?;
        let mut prepared = Vec::with_capacity(deltas.len());
        let mut seen = HashSet::with_capacity(deltas.len());
        let mut stats = RenderAssetResidencyMutationStats {
            input_delta_count: deltas.len(),
            ..RenderAssetResidencyMutationStats::default()
        };
        for delta in deltas {
            let resource = delta.resource();
            let acquired_count = delta.acquired_count();
            let released_count = delta.released_count();
            if (acquired_count == 0) == (released_count == 0) {
                return Err(
                    RenderAssetResidencyAdmissionError::MalformedReferenceDelta {
                        resource,
                        acquired_count,
                        released_count,
                    },
                );
            }
            if !seen.insert(resource) {
                return Err(
                    RenderAssetResidencyAdmissionError::DuplicateReferenceDelta { resource },
                );
            }
            stats.preflight_entry_lookup_count =
                stats.preflight_entry_lookup_count.saturating_add(1);
            let current_count = self.reference_count(resource);
            let next_reference_count = if acquired_count > 0 {
                current_count.checked_add(acquired_count).ok_or(
                    RenderAssetResidencyAdmissionError::ReferenceCountOverflow { resource },
                )?
            } else {
                current_count.checked_sub(released_count).ok_or(
                    RenderAssetResidencyAdmissionError::ReferenceCountUnderflow {
                        resource,
                        current_count,
                        released_count,
                    },
                )?
            };
            let request_seed = if current_count == 0 && next_reference_count > 0 {
                stats.catalog_lookup_count = stats.catalog_lookup_count.saturating_add(1);
                Some(resolve_ticket_seed(
                    resource,
                    management,
                    readiness,
                    device,
                    demand_generation,
                )?)
            } else {
                None
            };
            prepared.push(PreparedReferenceChange {
                resource,
                next_reference_count,
                request_seed,
                request: None,
            });
        }

        let requested_retirements = prepared
            .iter()
            .filter(|change| change.next_reference_count == 0)
            .filter(|change| {
                self.entries
                    .get(&change.resource)
                    .is_some_and(|entry| entry.active_artifact.is_some())
            })
            .count();
        self.gpu
            .ensure_ready_retirement_capacity(requested_retirements)
            .map_err(RenderAssetGpuRetirementBackpressure::into_admission_error)?;
        self.issue_reference_change_tickets(&mut prepared)?;
        let mut requests = Vec::new();
        let mut releases = Vec::new();
        for change in prepared {
            if change.next_reference_count == 0 {
                if let Some(mut entry) = self.entries.remove(&change.resource) {
                    self.gpu.detach_entry(&mut entry);
                    append_entry_release(entry, &mut releases);
                }
                continue;
            }
            if let Some(ticket) = change.request {
                requests.push(ticket);
                self.entries.insert(
                    change.resource,
                    RenderAssetResidencyEntry {
                        reference_count: change.next_reference_count,
                        pending: Some(PendingResidency {
                            ticket,
                            state: RenderAssetResidencyState::QueuedIo,
                            submission: None,
                        }),
                        pending_upload: None,
                        active: None,
                        active_artifact: None,
                    },
                );
            } else if let Some(entry) = self.entries.get_mut(&change.resource) {
                entry.reference_count = change.next_reference_count;
            }
        }
        if !self.entries.is_empty() {
            self.gpu.bind_device_epoch(device);
        }
        Ok(RenderAssetResidencyMutation::from_parts(
            requests, releases, stats,
        ))
    }

    pub(crate) fn reconcile_changed_resources(
        &mut self,
        changed_resources: &[UntypedResourceHandle],
        management: &ResourceManagementGeneration,
        readiness: &ResourceReadinessGeneration,
        device: RenderAssetDeviceEpoch,
        demand_generation: RenderAssetDemandGeneration,
    ) -> Result<RenderAssetResidencyMutation, RenderAssetResidencyAdmissionError> {
        self.ensure_device_epoch_admission(device)?;
        let mut prepared = Vec::with_capacity(changed_resources.len());
        let mut seen = HashSet::with_capacity(changed_resources.len());
        let mut stats = RenderAssetResidencyMutationStats {
            input_delta_count: changed_resources.len(),
            ..RenderAssetResidencyMutationStats::default()
        };
        for resource in changed_resources.iter().copied() {
            if !seen.insert(resource) {
                return Err(
                    RenderAssetResidencyAdmissionError::DuplicateReferenceDelta { resource },
                );
            }
            stats.preflight_entry_lookup_count =
                stats.preflight_entry_lookup_count.saturating_add(1);
            let Some(entry) = self.entries.get(&resource) else {
                continue;
            };
            stats.catalog_lookup_count = stats.catalog_lookup_count.saturating_add(1);
            let seed =
                resolve_ticket_seed(resource, management, readiness, device, demand_generation)?;
            if entry
                .pending
                .is_some_and(|pending| seed.matches(pending.ticket))
            {
                continue;
            }
            let active_matches = entry
                .active
                .is_some_and(|active| seed.matches(active.ticket));
            prepared.push(PreparedReconciliation {
                resource,
                cancel_pending: entry.pending.is_some(),
                request_seed: (!active_matches).then_some(seed),
                request: None,
            });
        }

        self.issue_reconciliation_tickets(&mut prepared)?;
        let mut requests = Vec::new();
        let mut releases = Vec::new();
        for reconciliation in prepared {
            let Some(entry) = self.entries.get_mut(&reconciliation.resource) else {
                continue;
            };
            if reconciliation.cancel_pending {
                self.gpu.detach_pending_upload(entry);
                if let Some(pending) = entry.pending.take() {
                    releases.push(release_pending(pending));
                }
            }
            if let Some(ticket) = reconciliation.request {
                requests.push(ticket);
                entry.pending = Some(PendingResidency {
                    ticket,
                    state: RenderAssetResidencyState::QueuedIo,
                    submission: None,
                });
            }
        }
        if !self.entries.is_empty() {
            self.gpu.bind_device_epoch(device);
        }
        Ok(RenderAssetResidencyMutation::from_parts(
            requests, releases, stats,
        ))
    }

    pub(crate) fn advance(
        &mut self,
        ticket: RenderAssetResidencyTicket,
        next: RenderAssetResidencyState,
    ) -> Result<(), RenderAssetResidencyTransitionError> {
        let entry = self.entry_for_pending_ticket_mut(ticket)?;
        let pending =
            entry
                .pending
                .as_mut()
                .ok_or(RenderAssetResidencyTransitionError::UnknownTicket {
                    presented: ticket.id(),
                })?;
        if !is_valid_pending_transition(pending.state, next) {
            return Err(RenderAssetResidencyTransitionError::InvalidTransition {
                from: pending.state,
                to: next,
            });
        }
        pending.state = next;
        Ok(())
    }

    pub(crate) fn bind_upload_submission(
        &mut self,
        ticket: RenderAssetResidencyTicket,
        submission: SubmissionTicket,
    ) -> Result<(), RenderAssetResidencyTransitionError> {
        self.bind_upload_submission_entry(ticket, submission)
            .map(|_| ())
    }

    pub(super) fn bind_upload_submission_entry(
        &mut self,
        ticket: RenderAssetResidencyTicket,
        submission: SubmissionTicket,
    ) -> Result<&mut RenderAssetResidencyEntry, RenderAssetResidencyTransitionError> {
        let entry = self.entry_for_pending_ticket_mut(ticket)?;
        let pending =
            entry
                .pending
                .as_mut()
                .ok_or(RenderAssetResidencyTransitionError::UnknownTicket {
                    presented: ticket.id(),
                })?;
        if pending.state != RenderAssetResidencyState::QueuedUpload {
            return Err(RenderAssetResidencyTransitionError::InvalidTransition {
                from: pending.state,
                to: RenderAssetResidencyState::Uploading,
            });
        }
        let actual = RenderAssetDeviceEpoch::new(submission.device_id(), submission.generation());
        if ticket.device() != actual {
            return Err(
                RenderAssetResidencyTransitionError::SubmissionDeviceMismatch {
                    expected: ticket.device(),
                    actual,
                },
            );
        }
        pending.state = RenderAssetResidencyState::Uploading;
        pending.submission = Some(submission);
        Ok(entry)
    }

    pub(crate) fn fail_pending(
        &mut self,
        ticket: RenderAssetResidencyTicket,
    ) -> Result<(), RenderAssetResidencyTransitionError> {
        let entry = self.entry_for_pending_ticket_mut(ticket)?;
        let pending =
            entry
                .pending
                .as_mut()
                .ok_or(RenderAssetResidencyTransitionError::UnknownTicket {
                    presented: ticket.id(),
                })?;
        if matches!(
            pending.state,
            RenderAssetResidencyState::Uploading
                | RenderAssetResidencyState::Failed
                | RenderAssetResidencyState::Cancelled
        ) {
            return Err(RenderAssetResidencyTransitionError::InvalidTransition {
                from: pending.state,
                to: RenderAssetResidencyState::Failed,
            });
        }
        pending.state = RenderAssetResidencyState::Failed;
        Ok(())
    }

    pub(crate) fn complete_upload(
        &mut self,
        ticket: RenderAssetResidencyTicket,
        submission: SubmissionTicket,
        status: SubmissionStatus,
    ) -> Result<RenderAssetResidencyMutation, RenderAssetResidencyTransitionError> {
        self.validate_upload_completion(ticket, submission, status)?;
        let entry = self.entry_for_pending_ticket_mut(ticket)?;
        let Some(mut pending) = entry.pending.take() else {
            return Err(RenderAssetResidencyTransitionError::UnknownTicket {
                presented: ticket.id(),
            });
        };
        if pending.state != RenderAssetResidencyState::Uploading {
            let from = pending.state;
            entry.pending = Some(pending);
            return Err(RenderAssetResidencyTransitionError::InvalidTransition {
                from,
                to: RenderAssetResidencyState::Resident,
            });
        }
        let Some(expected_submission) = pending.submission else {
            entry.pending = Some(pending);
            return Err(RenderAssetResidencyTransitionError::SubmissionNotBound {
                ticket: ticket.id(),
            });
        };
        if expected_submission != submission {
            entry.pending = Some(pending);
            return Err(RenderAssetResidencyTransitionError::SubmissionMismatch {
                expected: expected_submission,
                actual: submission,
            });
        }

        let mut releases = Vec::new();
        match status {
            SubmissionStatus::Completed => {
                if let Some(previous) = entry.active.replace(ActiveResidency { ticket, submission })
                {
                    releases.push(release_active(previous));
                }
            }
            SubmissionStatus::Cancelled => {
                pending.state = RenderAssetResidencyState::Cancelled;
                entry.pending = Some(pending);
            }
            SubmissionStatus::Failed | SubmissionStatus::DeviceLost => {
                pending.state = RenderAssetResidencyState::Failed;
                entry.pending = Some(pending);
            }
            SubmissionStatus::Accepted | SubmissionStatus::Submitted => {
                entry.pending = Some(pending);
                return Err(RenderAssetResidencyTransitionError::SubmissionNotTerminal { status });
            }
        }
        Ok(RenderAssetResidencyMutation::from_parts(
            Vec::new(),
            releases,
            RenderAssetResidencyMutationStats::default(),
        ))
    }

    pub(super) fn validate_upload_completion(
        &self,
        ticket: RenderAssetResidencyTicket,
        submission: SubmissionTicket,
        status: SubmissionStatus,
    ) -> Result<(), RenderAssetResidencyTransitionError> {
        if !status.is_terminal() {
            return Err(RenderAssetResidencyTransitionError::SubmissionNotTerminal { status });
        }
        let entry = self.entry_for_pending_ticket(ticket)?;
        let pending = entry
            .pending
            .ok_or(RenderAssetResidencyTransitionError::UnknownTicket {
                presented: ticket.id(),
            })?;
        if pending.state != RenderAssetResidencyState::Uploading {
            return Err(RenderAssetResidencyTransitionError::InvalidTransition {
                from: pending.state,
                to: RenderAssetResidencyState::Resident,
            });
        }
        let expected_submission =
            pending
                .submission
                .ok_or(RenderAssetResidencyTransitionError::SubmissionNotBound {
                    ticket: ticket.id(),
                })?;
        if expected_submission != submission {
            return Err(RenderAssetResidencyTransitionError::SubmissionMismatch {
                expected: expected_submission,
                actual: submission,
            });
        }
        Ok(())
    }

    pub(crate) fn reference_count(&self, resource: UntypedResourceHandle) -> usize {
        self.entries
            .get(&resource)
            .map(|entry| entry.reference_count)
            .unwrap_or(0)
    }

    pub(crate) fn pending_ticket(
        &self,
        resource: UntypedResourceHandle,
    ) -> Option<RenderAssetResidencyTicket> {
        self.entries
            .get(&resource)
            .and_then(|entry| entry.pending.map(|pending| pending.ticket))
    }

    pub(crate) fn resident_ticket(
        &self,
        resource: UntypedResourceHandle,
    ) -> Option<RenderAssetResidencyTicket> {
        self.entries
            .get(&resource)
            .and_then(|entry| entry.active.map(|active| active.ticket))
    }

    pub(crate) fn state(
        &self,
        ticket: RenderAssetResidencyTicket,
    ) -> Option<RenderAssetResidencyState> {
        let entry = self.entries.get(&ticket.resource())?;
        if entry
            .pending
            .is_some_and(|pending| pending.ticket == ticket)
        {
            return entry.pending.map(|pending| pending.state);
        }
        entry
            .active
            .is_some_and(|active| active.ticket == ticket)
            .then_some(RenderAssetResidencyState::Resident)
    }

    fn entry_for_pending_ticket_mut(
        &mut self,
        ticket: RenderAssetResidencyTicket,
    ) -> Result<&mut RenderAssetResidencyEntry, RenderAssetResidencyTransitionError> {
        let Some(entry) = self.entries.get_mut(&ticket.resource()) else {
            return Err(RenderAssetResidencyTransitionError::UnknownTicket {
                presented: ticket.id(),
            });
        };
        if entry
            .pending
            .is_some_and(|pending| pending.ticket == ticket)
        {
            return Ok(entry);
        }
        if entry.active.is_some_and(|active| active.ticket == ticket) && entry.pending.is_none() {
            return Err(RenderAssetResidencyTransitionError::InvalidTransition {
                from: RenderAssetResidencyState::Resident,
                to: RenderAssetResidencyState::Reading,
            });
        }
        let current = entry
            .pending
            .map(|pending| pending.ticket.id())
            .or_else(|| entry.active.map(|active| active.ticket.id()));
        match current {
            Some(current) => Err(RenderAssetResidencyTransitionError::StaleTicket {
                presented: ticket.id(),
                current,
            }),
            None => Err(RenderAssetResidencyTransitionError::UnknownTicket {
                presented: ticket.id(),
            }),
        }
    }

    fn entry_for_pending_ticket(
        &self,
        ticket: RenderAssetResidencyTicket,
    ) -> Result<&RenderAssetResidencyEntry, RenderAssetResidencyTransitionError> {
        let Some(entry) = self.entries.get(&ticket.resource()) else {
            return Err(RenderAssetResidencyTransitionError::UnknownTicket {
                presented: ticket.id(),
            });
        };
        if entry
            .pending
            .is_some_and(|pending| pending.ticket == ticket)
        {
            return Ok(entry);
        }
        if entry.active.is_some_and(|active| active.ticket == ticket) && entry.pending.is_none() {
            return Err(RenderAssetResidencyTransitionError::InvalidTransition {
                from: RenderAssetResidencyState::Resident,
                to: RenderAssetResidencyState::Reading,
            });
        }
        let current = entry
            .pending
            .map(|pending| pending.ticket.id())
            .or_else(|| entry.active.map(|active| active.ticket.id()));
        match current {
            Some(current) => Err(RenderAssetResidencyTransitionError::StaleTicket {
                presented: ticket.id(),
                current,
            }),
            None => Err(RenderAssetResidencyTransitionError::UnknownTicket {
                presented: ticket.id(),
            }),
        }
    }
}

fn resolve_ticket_seed(
    resource: UntypedResourceHandle,
    management: &ResourceManagementGeneration,
    readiness: &ResourceReadinessGeneration,
    device: RenderAssetDeviceEpoch,
    demand_generation: RenderAssetDemandGeneration,
) -> Result<RenderAssetResidencyTicketSeed, RenderAssetResidencyAdmissionError> {
    let Some(row) = management.row_by_id(resource.id()) else {
        return Err(RenderAssetResidencyAdmissionError::MissingCatalogRecord { resource });
    };
    if row.kind != resource.kind() {
        return Err(RenderAssetResidencyAdmissionError::CatalogKindMismatch {
            resource,
            catalog_kind: row.kind,
        });
    }
    if !readiness.contains_kind(resource.id(), resource.kind()) {
        return Err(RenderAssetResidencyAdmissionError::MissingReadinessRecord { resource });
    }
    let Some(dependency_revision) = readiness.dependency_revision(resource.id()) else {
        return Err(RenderAssetResidencyAdmissionError::MissingReadinessRecord { resource });
    };
    let Some(policy) = policy_for_resource_kind(resource.kind()) else {
        return Err(RenderAssetResidencyAdmissionError::UnsupportedResourceKind { resource });
    };
    Ok(RenderAssetResidencyTicketSeed {
        resource,
        asset_revision: row.revision,
        readiness_generation: readiness.sequence(),
        dependency_revision,
        demand_generation,
        device,
        scope: policy.scope,
        route: policy.route,
    })
}

#[derive(Clone, Copy)]
struct RenderAssetResidencyPolicy {
    scope: RenderAssetResidencyScope,
    route: RenderAssetResidencyRoute,
}

fn policy_for_resource_kind(kind: ResourceKind) -> Option<RenderAssetResidencyPolicy> {
    match kind {
        ResourceKind::Model => Some(RenderAssetResidencyPolicy {
            scope: RenderAssetResidencyScope::AllLods,
            route: RenderAssetResidencyRoute::CanonicalMeshSet,
        }),
        ResourceKind::Mesh => Some(RenderAssetResidencyPolicy {
            scope: RenderAssetResidencyScope::AllLods,
            route: RenderAssetResidencyRoute::SemanticBlocks,
        }),
        ResourceKind::Texture => Some(RenderAssetResidencyPolicy {
            scope: RenderAssetResidencyScope::Bootstrap,
            route: RenderAssetResidencyRoute::SemanticBlocks,
        }),
        ResourceKind::Material
        | ResourceKind::MaterialGraph
        | ResourceKind::Shader
        | ResourceKind::AnimationSkeleton => Some(RenderAssetResidencyPolicy {
            scope: RenderAssetResidencyScope::Bootstrap,
            route: RenderAssetResidencyRoute::PreparedDependencies,
        }),
        _ => None,
    }
}

fn is_valid_pending_transition(
    current: RenderAssetResidencyState,
    next: RenderAssetResidencyState,
) -> bool {
    matches!(
        (current, next),
        (
            RenderAssetResidencyState::QueuedIo,
            RenderAssetResidencyState::Reading
        ) | (
            RenderAssetResidencyState::Reading,
            RenderAssetResidencyState::Decoding
        ) | (
            RenderAssetResidencyState::Decoding,
            RenderAssetResidencyState::ReadyCpu
        ) | (
            RenderAssetResidencyState::ReadyCpu,
            RenderAssetResidencyState::QueuedUpload
        )
    )
}

fn append_entry_release(
    entry: RenderAssetResidencyEntry,
    releases: &mut Vec<RenderAssetResidencyRelease>,
) {
    if let Some(pending) = entry.pending {
        releases.push(release_pending(pending));
    }
    if let Some(active) = entry.active {
        releases.push(release_active(active));
    }
}

fn release_pending(pending: PendingResidency) -> RenderAssetResidencyRelease {
    let kind = match pending.state {
        RenderAssetResidencyState::Uploading => RenderAssetResidencyReleaseKind::RetireInFlight,
        RenderAssetResidencyState::Failed | RenderAssetResidencyState::Cancelled => {
            RenderAssetResidencyReleaseKind::DropTerminal
        }
        _ => RenderAssetResidencyReleaseKind::CancelPending,
    };
    RenderAssetResidencyRelease::new(pending.ticket, kind, pending.submission)
}

fn release_active(active: ActiveResidency) -> RenderAssetResidencyRelease {
    RenderAssetResidencyRelease::new(
        active.ticket,
        RenderAssetResidencyReleaseKind::RetireResident,
        Some(active.submission),
    )
}
