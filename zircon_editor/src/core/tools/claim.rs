use serde::{Deserialize, Serialize};

use super::{
    ToolAuthorityState, ToolInputCaptureEvent, ToolInstanceId, ToolLeaseId, ToolOwnerGeneration,
    ToolRequestId, ToolResourceKindId, ToolResourceKindRegistration, ToolResourceSet,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolRequestHandle {
    id: ToolRequestId,
    reserved_lease_id: ToolLeaseId,
    instance: ToolInstanceId,
    resources: ToolResourceSet,
}

impl ToolRequestHandle {
    pub(crate) fn new(
        id: ToolRequestId,
        reserved_lease_id: ToolLeaseId,
        instance: ToolInstanceId,
        resources: ToolResourceSet,
    ) -> Self {
        Self {
            id,
            reserved_lease_id,
            instance,
            resources,
        }
    }

    pub const fn id(&self) -> ToolRequestId {
        self.id
    }

    pub fn instance(&self) -> &ToolInstanceId {
        &self.instance
    }

    pub const fn owner_generation(&self) -> ToolOwnerGeneration {
        self.instance.owner_generation()
    }

    pub fn resources(&self) -> &ToolResourceSet {
        &self.resources
    }

    pub(crate) const fn reserved_lease_id(&self) -> ToolLeaseId {
        self.reserved_lease_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolLeaseHandle {
    id: ToolLeaseId,
    request_id: ToolRequestId,
    instance: ToolInstanceId,
    resources: ToolResourceSet,
}

impl ToolLeaseHandle {
    pub(crate) fn from_request(request: &ToolRequestHandle) -> Self {
        Self {
            id: request.reserved_lease_id(),
            request_id: request.id(),
            instance: request.instance().clone(),
            resources: request.resources().clone(),
        }
    }

    pub const fn id(&self) -> ToolLeaseId {
        self.id
    }

    pub const fn request_id(&self) -> ToolRequestId {
        self.request_id
    }

    pub fn instance(&self) -> &ToolInstanceId {
        &self.instance
    }

    pub const fn owner_generation(&self) -> ToolOwnerGeneration {
        self.instance.owner_generation()
    }

    pub fn resources(&self) -> &ToolResourceSet {
        &self.resources
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcquireDenial {
    QueueFull {
        max_queued: usize,
    },
    AlreadyHeld {
        resources: ToolResourceSet,
    },
    AlreadyQueued {
        resources: ToolResourceSet,
        position: usize,
    },
    ClaimIdentityExhausted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcquireOutcome {
    Acquired {
        lease: ToolLeaseHandle,
    },
    AlreadyHeld {
        lease: ToolLeaseHandle,
    },
    Queued {
        request: ToolRequestHandle,
        position: usize,
    },
    AlreadyQueued {
        request: ToolRequestHandle,
        position: usize,
    },
    Denied {
        holder: Option<ToolLeaseHandle>,
        reason: AcquireDenial,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReleaseOutcome {
    Released {
        lease: ToolLeaseHandle,
        activated_leases: Box<[ToolLeaseHandle]>,
    },
    NotHeld,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WithdrawOutcome {
    Withdrawn {
        request: ToolRequestHandle,
        previous_position: usize,
        activated_leases: Box<[ToolLeaseHandle]>,
    },
    NotQueued,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolOwnerRevokeOutcome {
    Revoked {
        generation: ToolOwnerGeneration,
        released_leases: Box<[ToolLeaseHandle]>,
        withdrawn_requests: Box<[ToolRequestHandle]>,
        activated_leases: Box<[ToolLeaseHandle]>,
        revoked_resource_kinds: Box<[ToolResourceKindId]>,
    },
    NotRegistered {
        generation: ToolOwnerGeneration,
    },
    BuiltinProtected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolLifecycleEvent {
    AuthorityStateChanged {
        previous: ToolAuthorityState,
        current: ToolAuthorityState,
    },
    OwnerGenerationRegistered {
        generation: ToolOwnerGeneration,
    },
    OwnerGenerationRevoked {
        generation: ToolOwnerGeneration,
    },
    ResourceKindRegistered {
        registration: ToolResourceKindRegistration,
    },
    ResourceKindsRevoked {
        owner_generation: ToolOwnerGeneration,
        kinds: Box<[ToolResourceKindId]>,
    },
    InputCapture {
        event: ToolInputCaptureEvent,
    },
    Activated {
        lease: ToolLeaseHandle,
    },
    Deactivated {
        lease: ToolLeaseHandle,
    },
    Queued {
        request: ToolRequestHandle,
        position: usize,
    },
    Withdrawn {
        request: ToolRequestHandle,
        previous_position: usize,
    },
    Denied {
        instance: ToolInstanceId,
        resources: ToolResourceSet,
        holder: Option<ToolLeaseHandle>,
        reason: AcquireDenial,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "tool scheduler reports must be committed to the ordered transition dispatcher"]
pub struct ToolScheduleReport<O> {
    outcome: O,
    events: Vec<ToolLifecycleEvent>,
}

impl<O> ToolScheduleReport<O> {
    pub(crate) fn new(outcome: O, events: Vec<ToolLifecycleEvent>) -> Self {
        Self { outcome, events }
    }

    pub fn outcome(&self) -> &O {
        &self.outcome
    }

    pub fn events(&self) -> &[ToolLifecycleEvent] {
        &self.events
    }

    pub fn into_parts(self) -> (O, Vec<ToolLifecycleEvent>) {
        (self.outcome, self.events)
    }
}
