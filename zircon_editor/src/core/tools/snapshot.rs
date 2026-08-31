use super::{ToolInputCaptureHandle, ToolLeaseHandle, ToolRequestHandle, ToolResourceKey};

/// Immutable state of one exclusive resource in a scheduler snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolResourceStateSnapshot {
    resource: ToolResourceKey,
    holder: Option<ToolLeaseHandle>,
    queued: Box<[ToolRequestHandle]>,
}

impl ToolResourceStateSnapshot {
    pub(super) fn new(
        resource: ToolResourceKey,
        holder: Option<ToolLeaseHandle>,
        queued: Box<[ToolRequestHandle]>,
    ) -> Self {
        Self {
            resource,
            holder,
            queued,
        }
    }

    pub fn resource(&self) -> &ToolResourceKey {
        &self.resource
    }

    pub fn holder(&self) -> Option<&ToolLeaseHandle> {
        self.holder.as_ref()
    }

    pub fn queued(&self) -> &[ToolRequestHandle] {
        &self.queued
    }
}

/// Complete deterministic state of the pure scheduler authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolSchedulerStateSnapshot {
    resources: Box<[ToolResourceStateSnapshot]>,
    active_leases: Box<[ToolLeaseHandle]>,
    queued_requests: Box<[ToolRequestHandle]>,
    active_input_captures: Box<[ToolInputCaptureHandle]>,
}

impl ToolSchedulerStateSnapshot {
    pub(super) fn new(
        resources: Box<[ToolResourceStateSnapshot]>,
        active_leases: Box<[ToolLeaseHandle]>,
        queued_requests: Box<[ToolRequestHandle]>,
        active_input_captures: Box<[ToolInputCaptureHandle]>,
    ) -> Self {
        Self {
            resources,
            active_leases,
            queued_requests,
            active_input_captures,
        }
    }

    pub fn resources(&self) -> &[ToolResourceStateSnapshot] {
        &self.resources
    }

    pub fn active_leases(&self) -> &[ToolLeaseHandle] {
        &self.active_leases
    }

    pub fn queued_requests(&self) -> &[ToolRequestHandle] {
        &self.queued_requests
    }

    pub fn active_input_captures(&self) -> &[ToolInputCaptureHandle] {
        &self.active_input_captures
    }

    pub fn resource(&self, resource: &ToolResourceKey) -> Option<&ToolResourceStateSnapshot> {
        self.resources
            .binary_search_by(|state| state.resource().cmp(resource))
            .ok()
            .map(|index| &self.resources[index])
    }
}
