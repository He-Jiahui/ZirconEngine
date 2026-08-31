use std::collections::{BTreeMap, VecDeque};
use zircon_runtime_interface::ui::dispatch::UiWindowId;

use super::input_capture::{ToolInputCaptureAuthority, ToolInputCaptureReport};
use super::limits::ToolQueueLimits;
use super::{
    AcquireDenial, AcquireOutcome, ReleaseOutcome, ToolInputCaptureDenial,
    ToolInputCaptureDisposition, ToolInputCaptureEndOutcome, ToolInputCaptureHandle,
    ToolInputCaptureOutcome, ToolInputCaptureOwner, ToolInputCaptureRequest, ToolInputSource,
    ToolInstanceId, ToolLeaseHandle, ToolLeaseId, ToolLifecycleEvent, ToolOwnerGeneration,
    ToolOwnerRevokeOutcome, ToolRequestHandle, ToolRequestId, ToolResourceKey, ToolResourceKindId,
    ToolResourceSet, ToolResourceStateSnapshot, ToolScheduleReport, ToolSchedulerStateSnapshot,
    ToolShutdownOutcome, WithdrawOutcome,
};

#[derive(Debug, Default, PartialEq, Eq)]
struct ResourceState {
    holder: Option<ToolLeaseId>,
    queue: VecDeque<ToolRequestId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClaimStateRef {
    Queued(ToolRequestId),
    Active(ToolLeaseId),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ToolScheduler {
    resources: BTreeMap<ToolResourceKey, ResourceState>,
    set_queue: VecDeque<ToolRequestId>,
    requests: BTreeMap<ToolRequestId, ToolRequestHandle>,
    leases: BTreeMap<ToolLeaseId, ToolLeaseHandle>,
    instances: BTreeMap<ToolInstanceId, ClaimStateRef>,
    input_captures: ToolInputCaptureAuthority,
    limits: ToolQueueLimits,
    next_request_id: Option<ToolRequestId>,
    next_lease_id: Option<ToolLeaseId>,
}

impl ToolScheduler {
    pub fn new(limits: ToolQueueLimits) -> Self {
        Self {
            resources: BTreeMap::new(),
            set_queue: VecDeque::new(),
            requests: BTreeMap::new(),
            leases: BTreeMap::new(),
            instances: BTreeMap::new(),
            input_captures: ToolInputCaptureAuthority::new(),
            limits,
            next_request_id: Some(ToolRequestId::first()),
            next_lease_id: Some(ToolLeaseId::first()),
        }
    }

    pub const fn limits(&self) -> ToolQueueLimits {
        self.limits
    }

    pub fn holder(&self, resource: &ToolResourceKey) -> Option<&ToolLeaseHandle> {
        self.resources
            .get(resource)
            .and_then(|state| state.holder)
            .and_then(|lease_id| self.leases.get(&lease_id))
    }

    pub fn queued_requests(
        &self,
        resource: &ToolResourceKey,
    ) -> impl Iterator<Item = &ToolRequestHandle> {
        self.resources
            .get(resource)
            .into_iter()
            .flat_map(|state| state.queue.iter())
            .filter_map(|request_id| self.requests.get(request_id))
    }

    pub fn active_lease(&self, instance: &ToolInstanceId) -> Option<&ToolLeaseHandle> {
        match self.instances.get(instance) {
            Some(ClaimStateRef::Active(lease_id)) => self.leases.get(lease_id),
            Some(ClaimStateRef::Queued(_)) | None => None,
        }
    }

    pub fn pending_request(&self, instance: &ToolInstanceId) -> Option<&ToolRequestHandle> {
        match self.instances.get(instance) {
            Some(ClaimStateRef::Queued(request_id)) => self.requests.get(request_id),
            Some(ClaimStateRef::Active(_)) | None => None,
        }
    }

    pub fn snapshot(&self) -> ToolSchedulerStateSnapshot {
        let resources = self
            .resources
            .iter()
            .map(|(resource, state)| {
                ToolResourceStateSnapshot::new(
                    resource.clone(),
                    state
                        .holder
                        .and_then(|lease_id| self.leases.get(&lease_id))
                        .cloned(),
                    state
                        .queue
                        .iter()
                        .filter_map(|request_id| self.requests.get(request_id).cloned())
                        .collect(),
                )
            })
            .collect();
        ToolSchedulerStateSnapshot::new(
            resources,
            self.leases.values().cloned().collect(),
            self.requests.values().cloned().collect(),
            self.input_captures.active().cloned().collect(),
        )
    }

    pub fn active_input_capture(
        &self,
        source: &ToolInputSource,
    ) -> Option<&ToolInputCaptureHandle> {
        self.input_captures.active_for_source(source)
    }

    pub(crate) fn begin_input_capture(
        &mut self,
        request: ToolInputCaptureRequest,
    ) -> ToolScheduleReport<ToolInputCaptureOutcome> {
        let owner = request.owner();
        let denial = match self.leases.get(&owner.lease_id()) {
            Some(lease) if lease.instance() == owner.instance() => {
                (!lease.resources().as_slice().contains(request.resource())).then_some(
                    ToolInputCaptureDenial::ResourceNotLeased {
                        resource: request.resource().clone(),
                    },
                )
            }
            Some(_) | None => Some(ToolInputCaptureDenial::LeaseNotActive {
                lease_id: owner.lease_id(),
            }),
        };
        if let Some(reason) = denial {
            return capture_schedule_report(self.input_captures.deny(request, reason));
        }
        capture_schedule_report(self.input_captures.begin(request))
    }

    pub(crate) fn end_input_capture(
        &mut self,
        capture_id: super::ToolInputCaptureId,
        owner: &ToolInputCaptureOwner,
        disposition: ToolInputCaptureDisposition,
    ) -> ToolScheduleReport<ToolInputCaptureEndOutcome> {
        capture_schedule_report(self.input_captures.end(capture_id, owner, disposition))
    }

    pub(crate) fn release_input_window_on_focus_loss(
        &mut self,
        window_id: &UiWindowId,
    ) -> ToolScheduleReport<Box<[ToolInputCaptureHandle]>> {
        capture_schedule_report(
            self.input_captures
                .release_window(window_id, ToolInputCaptureDisposition::FocusLost),
        )
    }

    /// Acquires one canonical resource set or queues the complete claim with no partial hold.
    pub fn acquire(
        &mut self,
        instance: ToolInstanceId,
        resources: ToolResourceSet,
    ) -> ToolScheduleReport<AcquireOutcome> {
        if let Some(existing) = self.instances.get(&instance).copied() {
            return self.report_existing_claim(instance, resources, existing);
        }

        let immediate = self.claim_can_activate_immediately(&resources);
        if !immediate {
            let max_queued = self.queue_limit(&resources);
            if self.queue_len(&resources) >= max_queued {
                let reason = AcquireDenial::QueueFull { max_queued };
                return self.denied(instance, resources, reason);
            }
        }

        let Some(request) = self.reserve_request(instance.clone(), resources.clone()) else {
            return self.denied(instance, resources, AcquireDenial::ClaimIdentityExhausted);
        };
        if immediate {
            let lease = self.activate_reserved_request(&request);
            return ToolScheduleReport::new(
                AcquireOutcome::Acquired {
                    lease: lease.clone(),
                },
                vec![ToolLifecycleEvent::Activated { lease }],
            );
        }

        self.enqueue_request(request.clone());
        let position = self.request_position(&request).unwrap_or(1);
        ToolScheduleReport::new(
            AcquireOutcome::Queued {
                request: request.clone(),
                position,
            },
            vec![ToolLifecycleEvent::Queued { request, position }],
        )
    }

    pub fn release(&mut self, lease_id: ToolLeaseId) -> ToolScheduleReport<ReleaseOutcome> {
        if !self.leases.contains_key(&lease_id) {
            return ToolScheduleReport::new(ReleaseOutcome::NotHeld, Vec::new());
        }
        let mut events = Vec::new();
        self.release_lease_captures(
            lease_id,
            ToolInputCaptureDisposition::OwnerLost,
            &mut events,
        );
        let Some(lease) = self.detach_active_lease_state(lease_id, &mut events) else {
            return ToolScheduleReport::new(ReleaseOutcome::NotHeld, Vec::new());
        };
        let activated_leases = self.promote_available_claims(&mut events);
        self.remove_empty_resource_states();
        ToolScheduleReport::new(
            ReleaseOutcome::Released {
                lease,
                activated_leases: activated_leases.into_boxed_slice(),
            },
            events,
        )
    }

    pub fn withdraw(&mut self, request_id: ToolRequestId) -> ToolScheduleReport<WithdrawOutcome> {
        let Some(request) = self.requests.get(&request_id).cloned() else {
            return ToolScheduleReport::new(WithdrawOutcome::NotQueued, Vec::new());
        };
        let previous_position = self.request_position(&request).unwrap_or(1);
        let mut events = Vec::new();
        let Some(request) =
            self.detach_queued_request_state(request_id, previous_position, &mut events)
        else {
            return ToolScheduleReport::new(WithdrawOutcome::NotQueued, Vec::new());
        };
        let activated_leases = self.promote_available_claims(&mut events);
        self.remove_empty_resource_states();
        ToolScheduleReport::new(
            WithdrawOutcome::Withdrawn {
                request,
                previous_position,
                activated_leases: activated_leases.into_boxed_slice(),
            },
            events,
        )
    }

    pub(crate) fn revoke_owner_generation(
        &mut self,
        generation: ToolOwnerGeneration,
        revoked_resource_kinds: &[ToolResourceKindId],
    ) -> ToolScheduleReport<ToolOwnerRevokeOutcome> {
        let lease_ids = self
            .leases
            .iter()
            .filter_map(|(lease_id, lease)| {
                (lease.owner_generation() == generation
                    || claim_uses_revoked_kind(lease.resources(), revoked_resource_kinds))
                .then_some(*lease_id)
            })
            .collect::<Vec<_>>();
        let request_positions = self
            .requests
            .iter()
            .filter_map(|(request_id, request)| {
                (request.owner_generation() == generation
                    || claim_uses_revoked_kind(request.resources(), revoked_resource_kinds))
                .then(|| (*request_id, self.request_position(request).unwrap_or(1)))
            })
            .collect::<Vec<_>>();
        let mut events = Vec::new();

        for lease_id in &lease_ids {
            self.release_lease_captures(
                *lease_id,
                ToolInputCaptureDisposition::OwnerLost,
                &mut events,
            );
        }
        let released_leases = lease_ids
            .into_iter()
            .filter_map(|lease_id| self.detach_active_lease_state(lease_id, &mut events))
            .collect::<Vec<_>>();
        let withdrawn_requests = request_positions
            .into_iter()
            .filter_map(|(request_id, previous_position)| {
                self.detach_queued_request_state(request_id, previous_position, &mut events)
            })
            .collect::<Vec<_>>();
        let activated_leases = self.promote_available_claims(&mut events);
        self.remove_empty_resource_states();

        ToolScheduleReport::new(
            ToolOwnerRevokeOutcome::Revoked {
                generation,
                released_leases: released_leases.into_boxed_slice(),
                withdrawn_requests: withdrawn_requests.into_boxed_slice(),
                activated_leases: activated_leases.into_boxed_slice(),
                revoked_resource_kinds: revoked_resource_kinds.to_vec().into_boxed_slice(),
            },
            events,
        )
    }

    pub(crate) fn shutdown(&mut self) -> ToolScheduleReport<ToolShutdownOutcome> {
        let capture_report = self.input_captures.shutdown();
        let (_, capture_events) = capture_report.into_parts();
        let queued = self
            .requests
            .values()
            .cloned()
            .map(|request| {
                let position = self.request_position(&request).unwrap_or(1);
                (request, position)
            })
            .collect::<Vec<_>>();
        let leases = std::mem::take(&mut self.leases)
            .into_values()
            .collect::<Vec<_>>();
        self.requests.clear();
        self.instances.clear();
        self.resources.clear();
        self.set_queue.clear();

        let mut outcome = ToolShutdownOutcome::default();
        let mut events = Vec::with_capacity(
            capture_events
                .len()
                .saturating_add(leases.len())
                .saturating_add(queued.len()),
        );
        events.extend(
            capture_events
                .into_iter()
                .map(|event| ToolLifecycleEvent::InputCapture { event }),
        );
        for lease in leases {
            if lease.resources().len() == 1 {
                outcome.released_single_leases += 1;
            } else {
                outcome.released_set_leases += 1;
            }
            events.push(ToolLifecycleEvent::Deactivated { lease });
        }
        for (request, previous_position) in queued {
            if request.resources().len() == 1 {
                outcome.withdrawn_single_requests += 1;
            } else {
                outcome.withdrawn_set_requests += 1;
            }
            events.push(ToolLifecycleEvent::Withdrawn {
                request,
                previous_position,
            });
        }
        ToolScheduleReport::new(outcome, events)
    }

    fn report_existing_claim(
        &self,
        instance: ToolInstanceId,
        resources: ToolResourceSet,
        existing: ClaimStateRef,
    ) -> ToolScheduleReport<AcquireOutcome> {
        match existing {
            ClaimStateRef::Active(lease_id) => {
                let lease = self.leases.get(&lease_id).cloned();
                match lease {
                    Some(lease) if lease.resources() == &resources => {
                        ToolScheduleReport::new(AcquireOutcome::AlreadyHeld { lease }, Vec::new())
                    }
                    Some(lease) => {
                        let reason = AcquireDenial::AlreadyHeld {
                            resources: lease.resources().clone(),
                        };
                        ToolScheduleReport::new(
                            AcquireOutcome::Denied {
                                holder: Some(lease.clone()),
                                reason: reason.clone(),
                            },
                            vec![ToolLifecycleEvent::Denied {
                                instance,
                                resources,
                                holder: Some(lease),
                                reason,
                            }],
                        )
                    }
                    None => self.denied(instance, resources, AcquireDenial::ClaimIdentityExhausted),
                }
            }
            ClaimStateRef::Queued(request_id) => {
                let request = self.requests.get(&request_id).cloned();
                match request {
                    Some(request) if request.resources() == &resources => {
                        let position = self.request_position(&request).unwrap_or(1);
                        ToolScheduleReport::new(
                            AcquireOutcome::AlreadyQueued { request, position },
                            Vec::new(),
                        )
                    }
                    Some(request) => {
                        let position = self.request_position(&request).unwrap_or(1);
                        let reason = AcquireDenial::AlreadyQueued {
                            resources: request.resources().clone(),
                            position,
                        };
                        ToolScheduleReport::new(
                            AcquireOutcome::Denied {
                                holder: None,
                                reason: reason.clone(),
                            },
                            vec![ToolLifecycleEvent::Denied {
                                instance,
                                resources,
                                holder: None,
                                reason,
                            }],
                        )
                    }
                    None => self.denied(instance, resources, AcquireDenial::ClaimIdentityExhausted),
                }
            }
        }
    }

    fn denied(
        &self,
        instance: ToolInstanceId,
        resources: ToolResourceSet,
        reason: AcquireDenial,
    ) -> ToolScheduleReport<AcquireOutcome> {
        let holder = resources
            .as_slice()
            .iter()
            .find_map(|resource| self.holder(resource).cloned());
        ToolScheduleReport::new(
            AcquireOutcome::Denied {
                holder: holder.clone(),
                reason: reason.clone(),
            },
            vec![ToolLifecycleEvent::Denied {
                instance,
                resources,
                holder,
                reason,
            }],
        )
    }

    fn reserve_request(
        &mut self,
        instance: ToolInstanceId,
        resources: ToolResourceSet,
    ) -> Option<ToolRequestHandle> {
        let request_id = self.next_request_id?;
        let lease_id = self.next_lease_id?;
        self.next_request_id = request_id.checked_next();
        self.next_lease_id = lease_id.checked_next();
        Some(ToolRequestHandle::new(
            request_id, lease_id, instance, resources,
        ))
    }

    fn enqueue_request(&mut self, request: ToolRequestHandle) {
        let request_id = request.id();
        for resource in request.resources().as_slice() {
            self.resources
                .entry(resource.clone())
                .or_default()
                .queue
                .push_back(request_id);
        }
        if request.resources().len() > 1 {
            self.set_queue.push_back(request_id);
        }
        self.instances.insert(
            request.instance().clone(),
            ClaimStateRef::Queued(request_id),
        );
        self.requests.insert(request_id, request);
    }

    fn activate_reserved_request(&mut self, request: &ToolRequestHandle) -> ToolLeaseHandle {
        let lease = ToolLeaseHandle::from_request(request);
        for resource in lease.resources().as_slice() {
            let state = self.resources.entry(resource.clone()).or_default();
            debug_assert!(state.holder.is_none());
            state.holder = Some(lease.id());
        }
        self.instances
            .insert(lease.instance().clone(), ClaimStateRef::Active(lease.id()));
        self.leases.insert(lease.id(), lease.clone());
        lease
    }

    fn claim_can_activate_immediately(&self, resources: &ToolResourceSet) -> bool {
        if !self.resources_are_available(resources) {
            return false;
        }
        if resources.len() > 1 {
            self.set_queue.is_empty()
        } else {
            !self.set_head_overlaps(&resources.as_slice()[0])
        }
    }

    fn resources_are_available(&self, resources: &ToolResourceSet) -> bool {
        resources
            .as_slice()
            .iter()
            .all(|resource| self.holder(resource).is_none())
    }

    fn queue_limit(&self, resources: &ToolResourceSet) -> usize {
        if resources.len() == 1 {
            self.limits.max_single_queue_per_resource()
        } else {
            self.limits.max_set_queue()
        }
    }

    fn queue_len(&self, resources: &ToolResourceSet) -> usize {
        if resources.len() > 1 {
            return self.set_queue.len();
        }
        self.resources
            .get(&resources.as_slice()[0])
            .map(|state| {
                state
                    .queue
                    .iter()
                    .filter(|request_id| {
                        self.requests
                            .get(request_id)
                            .is_some_and(|request| request.resources().len() == 1)
                    })
                    .count()
            })
            .unwrap_or_default()
    }

    fn request_position(&self, request: &ToolRequestHandle) -> Option<usize> {
        if request.resources().len() > 1 {
            return self
                .set_queue
                .iter()
                .position(|request_id| *request_id == request.id())
                .map(|index| index + 1);
        }
        let state = self.resources.get(&request.resources().as_slice()[0])?;
        let mut position = 0;
        for request_id in &state.queue {
            let Some(candidate) = self.requests.get(request_id) else {
                continue;
            };
            if candidate.resources().len() != 1 {
                continue;
            }
            position += 1;
            if *request_id == request.id() {
                return Some(position);
            }
        }
        None
    }

    fn detach_request(&mut self, request: &ToolRequestHandle) {
        for resource in request.resources().as_slice() {
            if let Some(state) = self.resources.get_mut(resource) {
                state.queue.retain(|request_id| *request_id != request.id());
            }
        }
        if request.resources().len() > 1 {
            self.set_queue
                .retain(|request_id| *request_id != request.id());
        }
    }

    fn release_lease_captures(
        &mut self,
        lease_id: ToolLeaseId,
        disposition: ToolInputCaptureDisposition,
        events: &mut Vec<ToolLifecycleEvent>,
    ) {
        let capture_report = self.input_captures.release_lease(lease_id, disposition);
        let (_, capture_events) = capture_report.into_parts();
        events.extend(
            capture_events
                .into_iter()
                .map(|event| ToolLifecycleEvent::InputCapture { event }),
        );
    }

    fn detach_active_lease_state(
        &mut self,
        lease_id: ToolLeaseId,
        events: &mut Vec<ToolLifecycleEvent>,
    ) -> Option<ToolLeaseHandle> {
        let lease = self.leases.remove(&lease_id)?;
        for resource in lease.resources().as_slice() {
            if let Some(state) = self.resources.get_mut(resource) {
                debug_assert_eq!(state.holder, Some(lease_id));
                if state.holder == Some(lease_id) {
                    state.holder = None;
                }
            }
        }
        if self.instances.get(lease.instance()) == Some(&ClaimStateRef::Active(lease_id)) {
            self.instances.remove(lease.instance());
        }
        events.push(ToolLifecycleEvent::Deactivated {
            lease: lease.clone(),
        });
        Some(lease)
    }

    fn detach_queued_request_state(
        &mut self,
        request_id: ToolRequestId,
        previous_position: usize,
        events: &mut Vec<ToolLifecycleEvent>,
    ) -> Option<ToolRequestHandle> {
        let request = self.requests.get(&request_id).cloned()?;
        self.detach_request(&request);
        self.requests.remove(&request_id);
        if self.instances.get(request.instance()) == Some(&ClaimStateRef::Queued(request_id)) {
            self.instances.remove(request.instance());
        }
        events.push(ToolLifecycleEvent::Withdrawn {
            request: request.clone(),
            previous_position,
        });
        Some(request)
    }

    fn set_head_overlaps(&self, resource: &ToolResourceKey) -> bool {
        self.set_queue
            .front()
            .and_then(|request_id| self.requests.get(request_id))
            .is_some_and(|request| request.resources().as_slice().contains(resource))
    }

    fn promote_available_claims(
        &mut self,
        events: &mut Vec<ToolLifecycleEvent>,
    ) -> Vec<ToolLeaseHandle> {
        let mut activated = self.promote_available_sets(events);
        activated.extend(self.promote_waiting_singles(events));
        activated
    }

    fn promote_available_sets(
        &mut self,
        events: &mut Vec<ToolLifecycleEvent>,
    ) -> Vec<ToolLeaseHandle> {
        let mut activated = Vec::new();
        loop {
            let Some(request_id) = self.set_queue.front().copied() else {
                break;
            };
            let Some(request) = self.requests.get(&request_id).cloned() else {
                self.set_queue.pop_front();
                continue;
            };
            if !self.resources_are_available(request.resources()) {
                break;
            }
            self.detach_request(&request);
            self.requests.remove(&request_id);
            let lease = self.activate_reserved_request(&request);
            events.push(ToolLifecycleEvent::Activated {
                lease: lease.clone(),
            });
            activated.push(lease);
        }
        activated
    }

    fn promote_waiting_singles(
        &mut self,
        events: &mut Vec<ToolLifecycleEvent>,
    ) -> Vec<ToolLeaseHandle> {
        let mut activated = Vec::new();
        let resources = self.resources.keys().cloned().collect::<Vec<_>>();
        for resource in resources {
            if self.holder(&resource).is_some() || self.set_head_overlaps(&resource) {
                continue;
            }
            let request_id = self.resources.get(&resource).and_then(|state| {
                state.queue.iter().copied().find(|request_id| {
                    self.requests
                        .get(request_id)
                        .is_some_and(|request| request.resources().len() == 1)
                })
            });
            let Some(request_id) = request_id else {
                continue;
            };
            let Some(request) = self.requests.get(&request_id).cloned() else {
                continue;
            };
            self.detach_request(&request);
            self.requests.remove(&request_id);
            let lease = self.activate_reserved_request(&request);
            events.push(ToolLifecycleEvent::Activated {
                lease: lease.clone(),
            });
            activated.push(lease);
        }
        activated
    }

    fn remove_empty_resource_states(&mut self) {
        self.resources
            .retain(|_, state| state.holder.is_some() || !state.queue.is_empty());
    }
}

fn capture_schedule_report<O>(report: ToolInputCaptureReport<O>) -> ToolScheduleReport<O> {
    let (outcome, capture_events) = report.into_parts();
    ToolScheduleReport::new(
        outcome,
        capture_events
            .into_iter()
            .map(|event| ToolLifecycleEvent::InputCapture { event })
            .collect(),
    )
}

impl Default for ToolScheduler {
    fn default() -> Self {
        Self::new(ToolQueueLimits::default())
    }
}

fn claim_uses_revoked_kind(
    resources: &ToolResourceSet,
    revoked_resource_kinds: &[ToolResourceKindId],
) -> bool {
    resources.as_slice().iter().any(|resource| {
        revoked_resource_kinds
            .binary_search(resource.kind())
            .is_ok()
    })
}
