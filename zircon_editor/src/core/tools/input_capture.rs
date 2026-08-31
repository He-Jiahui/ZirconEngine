use std::collections::BTreeMap;
use std::fmt;
use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::dispatch::{
    UiDeviceId, UiPointerId, UiPointerSource, UiSurfaceId, UiUserId, UiWindowId,
};

use super::{ToolInstanceId, ToolLeaseHandle, ToolLeaseId, ToolOwnerGeneration, ToolResourceKey};

pub const DEFAULT_MAX_ACTIVE_TOOL_INPUT_CAPTURES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ToolInputCaptureId(NonZeroU64);

impl ToolInputCaptureId {
    pub const fn value(self) -> u64 {
        self.0.get()
    }

    const fn first() -> Self {
        Self(NonZeroU64::MIN)
    }

    const fn checked_next(self) -> Option<Self> {
        match self.0.get().checked_add(1).and_then(NonZeroU64::new) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

impl fmt::Display for ToolInputCaptureId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct ToolInputCapturePriority(u16);

impl ToolInputCapturePriority {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ToolInputScope {
    window_id: UiWindowId,
    surface_id: UiSurfaceId,
}

impl ToolInputScope {
    pub fn new(window_id: UiWindowId, surface_id: UiSurfaceId) -> Self {
        Self {
            window_id,
            surface_id,
        }
    }

    pub fn window_id(&self) -> &UiWindowId {
        &self.window_id
    }

    pub fn surface_id(&self) -> &UiSurfaceId {
        &self.surface_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ToolInputSource {
    Pointer {
        scope: ToolInputScope,
        user_id: Option<UiUserId>,
        device_id: Option<UiDeviceId>,
        pointer_id: Option<UiPointerId>,
        pointer_source: UiPointerSource,
    },
    Keyboard {
        scope: ToolInputScope,
        user_id: Option<UiUserId>,
        device_id: Option<UiDeviceId>,
    },
    Device {
        scope: ToolInputScope,
        device_id: UiDeviceId,
    },
}

impl ToolInputSource {
    pub fn scope(&self) -> &ToolInputScope {
        match self {
            Self::Pointer { scope, .. }
            | Self::Keyboard { scope, .. }
            | Self::Device { scope, .. } => scope,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolInputCaptureOwner {
    lease_id: ToolLeaseId,
    instance: ToolInstanceId,
}

impl ToolInputCaptureOwner {
    pub fn from_lease(lease: &ToolLeaseHandle) -> Self {
        Self {
            lease_id: lease.id(),
            instance: lease.instance().clone(),
        }
    }

    #[cfg(test)]
    fn new(lease_id: ToolLeaseId, instance: ToolInstanceId) -> Self {
        Self { lease_id, instance }
    }

    pub const fn lease_id(&self) -> ToolLeaseId {
        self.lease_id
    }

    pub fn instance(&self) -> &ToolInstanceId {
        &self.instance
    }

    pub const fn generation(&self) -> ToolOwnerGeneration {
        self.instance.owner_generation()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolInputCaptureRequest {
    owner: ToolInputCaptureOwner,
    source: ToolInputSource,
    resource: ToolResourceKey,
    priority: ToolInputCapturePriority,
}

impl ToolInputCaptureRequest {
    pub fn new(
        owner: ToolInputCaptureOwner,
        source: ToolInputSource,
        resource: ToolResourceKey,
        priority: ToolInputCapturePriority,
    ) -> Self {
        Self {
            owner,
            source,
            resource,
            priority,
        }
    }

    pub fn owner(&self) -> &ToolInputCaptureOwner {
        &self.owner
    }

    pub fn source(&self) -> &ToolInputSource {
        &self.source
    }

    pub fn resource(&self) -> &ToolResourceKey {
        &self.resource
    }

    pub const fn priority(&self) -> ToolInputCapturePriority {
        self.priority
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolInputCaptureHandle {
    id: ToolInputCaptureId,
    owner: ToolInputCaptureOwner,
    source: ToolInputSource,
    resource: ToolResourceKey,
    priority: ToolInputCapturePriority,
}

impl ToolInputCaptureHandle {
    fn new(id: ToolInputCaptureId, request: &ToolInputCaptureRequest) -> Self {
        Self {
            id,
            owner: request.owner.clone(),
            source: request.source.clone(),
            resource: request.resource.clone(),
            priority: request.priority,
        }
    }

    pub const fn id(&self) -> ToolInputCaptureId {
        self.id
    }

    pub fn owner(&self) -> &ToolInputCaptureOwner {
        &self.owner
    }

    pub fn source(&self) -> &ToolInputSource {
        &self.source
    }

    pub fn resource(&self) -> &ToolResourceKey {
        &self.resource
    }

    pub const fn priority(&self) -> ToolInputCapturePriority {
        self.priority
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolInputCaptureDisposition {
    Completed,
    Accepted,
    Cancelled,
    Aborted,
    Stolen,
    OwnerLost,
    FocusLost,
    Shutdown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolInputCaptureDenial {
    AlreadyHeld,
    LowerPriority {
        holder_priority: ToolInputCapturePriority,
    },
    CapacityReached {
        max_active_captures: usize,
    },
    LeaseNotActive {
        lease_id: ToolLeaseId,
    },
    ResourceNotLeased {
        resource: ToolResourceKey,
    },
    CaptureIdentityExhausted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolInputCaptureOutcome {
    Captured {
        handle: ToolInputCaptureHandle,
        preempted: Option<ToolInputCaptureHandle>,
    },
    AlreadyHeld {
        handle: ToolInputCaptureHandle,
    },
    Denied {
        holder: Option<ToolInputCaptureHandle>,
        reason: ToolInputCaptureDenial,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolInputCaptureEndOutcome {
    Ended {
        handle: ToolInputCaptureHandle,
        disposition: ToolInputCaptureDisposition,
    },
    NotHeld,
    OwnerMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolInputCaptureEvent {
    Started {
        handle: ToolInputCaptureHandle,
    },
    Ended {
        handle: ToolInputCaptureHandle,
        disposition: ToolInputCaptureDisposition,
    },
    Denied {
        request: ToolInputCaptureRequest,
        holder: Option<ToolInputCaptureHandle>,
        reason: ToolInputCaptureDenial,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "tool input capture reports contain ordered lifecycle events that must be observed"]
pub(crate) struct ToolInputCaptureReport<O> {
    outcome: O,
    events: Vec<ToolInputCaptureEvent>,
}

impl<O> ToolInputCaptureReport<O> {
    fn new(outcome: O, events: Vec<ToolInputCaptureEvent>) -> Self {
        Self { outcome, events }
    }

    pub(crate) fn outcome(&self) -> &O {
        &self.outcome
    }

    pub(crate) fn events(&self) -> &[ToolInputCaptureEvent] {
        &self.events
    }

    pub(crate) fn into_parts(self) -> (O, Vec<ToolInputCaptureEvent>) {
        (self.outcome, self.events)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ToolInputCaptureAuthority {
    next_capture_id: Option<ToolInputCaptureId>,
    max_active_captures: usize,
    captures: BTreeMap<ToolInputCaptureId, ToolInputCaptureHandle>,
    by_source: BTreeMap<ToolInputSource, ToolInputCaptureId>,
}

impl Default for ToolInputCaptureAuthority {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolInputCaptureAuthority {
    pub(crate) fn new() -> Self {
        Self {
            next_capture_id: Some(ToolInputCaptureId::first()),
            max_active_captures: DEFAULT_MAX_ACTIVE_TOOL_INPUT_CAPTURES,
            captures: BTreeMap::new(),
            by_source: BTreeMap::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_max_active_captures(max_active_captures: std::num::NonZeroUsize) -> Self {
        Self {
            next_capture_id: Some(ToolInputCaptureId::first()),
            max_active_captures: max_active_captures.get(),
            captures: BTreeMap::new(),
            by_source: BTreeMap::new(),
        }
    }

    pub(crate) fn begin(
        &mut self,
        request: ToolInputCaptureRequest,
    ) -> ToolInputCaptureReport<ToolInputCaptureOutcome> {
        if let Some(existing_id) = self.by_source.get(request.source()) {
            let Some(existing) = self.captures.get(existing_id).cloned() else {
                self.by_source.remove(request.source());
                return self.begin(request);
            };
            if existing.owner() == request.owner() {
                return ToolInputCaptureReport::new(
                    ToolInputCaptureOutcome::AlreadyHeld { handle: existing },
                    Vec::new(),
                );
            }
            if request.priority() <= existing.priority() {
                let reason = if request.priority() == existing.priority() {
                    ToolInputCaptureDenial::AlreadyHeld
                } else {
                    ToolInputCaptureDenial::LowerPriority {
                        holder_priority: existing.priority(),
                    }
                };
                let event = ToolInputCaptureEvent::Denied {
                    request: request.clone(),
                    holder: Some(existing.clone()),
                    reason: reason.clone(),
                };
                return ToolInputCaptureReport::new(
                    ToolInputCaptureOutcome::Denied {
                        holder: Some(existing),
                        reason,
                    },
                    vec![event],
                );
            }
            if self.next_capture_id.is_none() {
                let reason = ToolInputCaptureDenial::CaptureIdentityExhausted;
                return ToolInputCaptureReport::new(
                    ToolInputCaptureOutcome::Denied {
                        holder: Some(existing.clone()),
                        reason: reason.clone(),
                    },
                    vec![ToolInputCaptureEvent::Denied {
                        request,
                        holder: Some(existing),
                        reason,
                    }],
                );
            }
            let preempted = self.end_capture(existing.id(), ToolInputCaptureDisposition::Stolen);
            return match preempted.outcome {
                ToolInputCaptureEndOutcome::Ended { handle, .. } => {
                    self.start_capture(request, Some(handle))
                }
                ToolInputCaptureEndOutcome::NotHeld | ToolInputCaptureEndOutcome::OwnerMismatch => {
                    self.by_source.remove(request.source());
                    self.start_capture(request, None)
                }
            };
        }
        if self.captures.len() >= self.max_active_captures {
            let reason = ToolInputCaptureDenial::CapacityReached {
                max_active_captures: self.max_active_captures,
            };
            return ToolInputCaptureReport::new(
                ToolInputCaptureOutcome::Denied {
                    holder: None,
                    reason: reason.clone(),
                },
                vec![ToolInputCaptureEvent::Denied {
                    request,
                    holder: None,
                    reason,
                }],
            );
        }
        self.start_capture(request, None)
    }

    pub(crate) fn end(
        &mut self,
        capture_id: ToolInputCaptureId,
        owner: &ToolInputCaptureOwner,
        disposition: ToolInputCaptureDisposition,
    ) -> ToolInputCaptureReport<ToolInputCaptureEndOutcome> {
        match self.captures.get(&capture_id) {
            Some(handle) if handle.owner() != owner => {
                ToolInputCaptureReport::new(ToolInputCaptureEndOutcome::OwnerMismatch, Vec::new())
            }
            Some(_) => self.end_capture(capture_id, disposition),
            None => ToolInputCaptureReport::new(ToolInputCaptureEndOutcome::NotHeld, Vec::new()),
        }
    }

    #[cfg(test)]
    pub(crate) fn release_owner(
        &mut self,
        owner: &ToolInputCaptureOwner,
        disposition: ToolInputCaptureDisposition,
    ) -> ToolInputCaptureReport<Box<[ToolInputCaptureHandle]>> {
        let ids = self
            .captures
            .values()
            .filter(|capture| capture.owner() == owner)
            .map(ToolInputCaptureHandle::id)
            .collect::<Vec<_>>();
        self.end_ids(ids, disposition)
    }

    pub(crate) fn active_for_source(
        &self,
        source: &ToolInputSource,
    ) -> Option<&ToolInputCaptureHandle> {
        self.by_source
            .get(source)
            .and_then(|id| self.captures.get(id))
    }

    pub(crate) fn active(&self) -> impl Iterator<Item = &ToolInputCaptureHandle> {
        self.captures.values()
    }

    pub(crate) fn shutdown(&mut self) -> ToolInputCaptureReport<Box<[ToolInputCaptureHandle]>> {
        let ids = self.captures.keys().copied().collect::<Vec<_>>();
        self.end_ids(ids, ToolInputCaptureDisposition::Shutdown)
    }

    fn start_capture(
        &mut self,
        request: ToolInputCaptureRequest,
        preempted: Option<ToolInputCaptureHandle>,
    ) -> ToolInputCaptureReport<ToolInputCaptureOutcome> {
        let Some(id) = self.next_capture_id else {
            let reason = ToolInputCaptureDenial::CaptureIdentityExhausted;
            return ToolInputCaptureReport::new(
                ToolInputCaptureOutcome::Denied {
                    holder: None,
                    reason: reason.clone(),
                },
                vec![ToolInputCaptureEvent::Denied {
                    request,
                    holder: None,
                    reason,
                }],
            );
        };
        self.next_capture_id = id.checked_next();
        let handle = ToolInputCaptureHandle::new(id, &request);
        self.by_source.insert(request.source.clone(), id);
        self.captures.insert(id, handle.clone());
        let mut events = Vec::with_capacity(usize::from(preempted.is_some()).saturating_add(1));
        if let Some(preempted) = preempted.as_ref() {
            events.push(ToolInputCaptureEvent::Ended {
                handle: preempted.clone(),
                disposition: ToolInputCaptureDisposition::Stolen,
            });
        }
        events.push(ToolInputCaptureEvent::Started {
            handle: handle.clone(),
        });
        ToolInputCaptureReport::new(
            ToolInputCaptureOutcome::Captured { handle, preempted },
            events,
        )
    }

    pub(crate) fn deny(
        &self,
        request: ToolInputCaptureRequest,
        reason: ToolInputCaptureDenial,
    ) -> ToolInputCaptureReport<ToolInputCaptureOutcome> {
        let holder = self.active_for_source(request.source()).cloned();
        ToolInputCaptureReport::new(
            ToolInputCaptureOutcome::Denied {
                holder: holder.clone(),
                reason: reason.clone(),
            },
            vec![ToolInputCaptureEvent::Denied {
                request,
                holder,
                reason,
            }],
        )
    }

    pub(crate) fn release_lease(
        &mut self,
        lease_id: ToolLeaseId,
        disposition: ToolInputCaptureDisposition,
    ) -> ToolInputCaptureReport<Box<[ToolInputCaptureHandle]>> {
        let ids = self
            .captures
            .values()
            .filter(|capture| capture.owner().lease_id() == lease_id)
            .map(ToolInputCaptureHandle::id)
            .collect::<Vec<_>>();
        self.end_ids(ids, disposition)
    }

    pub(crate) fn release_window(
        &mut self,
        window_id: &UiWindowId,
        disposition: ToolInputCaptureDisposition,
    ) -> ToolInputCaptureReport<Box<[ToolInputCaptureHandle]>> {
        let ids = self
            .captures
            .values()
            .filter(|capture| capture.source().scope().window_id() == window_id)
            .map(ToolInputCaptureHandle::id)
            .collect::<Vec<_>>();
        self.end_ids(ids, disposition)
    }

    fn end_ids(
        &mut self,
        ids: Vec<ToolInputCaptureId>,
        disposition: ToolInputCaptureDisposition,
    ) -> ToolInputCaptureReport<Box<[ToolInputCaptureHandle]>> {
        let mut ended = Vec::with_capacity(ids.len());
        let mut events = Vec::with_capacity(ids.len());
        for id in ids {
            let report = self.end_capture(id, disposition);
            if let ToolInputCaptureEndOutcome::Ended { handle, .. } = report.outcome {
                events.extend(report.events);
                ended.push(handle);
            }
        }
        ToolInputCaptureReport::new(ended.into_boxed_slice(), events)
    }

    fn end_capture(
        &mut self,
        capture_id: ToolInputCaptureId,
        disposition: ToolInputCaptureDisposition,
    ) -> ToolInputCaptureReport<ToolInputCaptureEndOutcome> {
        let Some(handle) = self.captures.remove(&capture_id) else {
            return ToolInputCaptureReport::new(ToolInputCaptureEndOutcome::NotHeld, Vec::new());
        };
        self.by_source.remove(handle.source());
        ToolInputCaptureReport::new(
            ToolInputCaptureEndOutcome::Ended {
                handle: handle.clone(),
                disposition,
            },
            vec![ToolInputCaptureEvent::Ended {
                handle,
                disposition,
            }],
        )
    }
}

#[cfg(test)]
mod tests;
