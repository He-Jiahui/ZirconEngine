use std::num::NonZeroU64;

use crate::core::resource::UntypedResourceHandle;
use zr_rhi::{DeviceGeneration, DeviceId, SubmissionStatus, SubmissionTicket};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RenderAssetDemandGeneration(NonZeroU64);

impl RenderAssetDemandGeneration {
    pub(crate) const fn new(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RenderAssetDeviceEpoch {
    device_id: DeviceId,
    generation: DeviceGeneration,
}

impl RenderAssetDeviceEpoch {
    pub(crate) const fn new(device_id: DeviceId, generation: DeviceGeneration) -> Self {
        Self {
            device_id,
            generation,
        }
    }

    pub(crate) const fn device_id(self) -> DeviceId {
        self.device_id
    }

    pub(crate) const fn generation(self) -> DeviceGeneration {
        self.generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RenderAssetResidencyTicketId(NonZeroU64);

impl RenderAssetResidencyTicketId {
    pub(super) const fn new(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum RenderAssetResidencyScope {
    Bootstrap,
    AllLods,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum RenderAssetResidencyRoute {
    SemanticBlocks,
    CanonicalMeshSet,
    PreparedDependencies,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RenderAssetResidencyTicket {
    id: RenderAssetResidencyTicketId,
    resource: UntypedResourceHandle,
    asset_revision: u64,
    readiness_generation: u64,
    dependency_revision: u64,
    demand_generation: RenderAssetDemandGeneration,
    device: RenderAssetDeviceEpoch,
    scope: RenderAssetResidencyScope,
    route: RenderAssetResidencyRoute,
}

impl RenderAssetResidencyTicket {
    pub(super) const fn from_parts(
        id: RenderAssetResidencyTicketId,
        resource: UntypedResourceHandle,
        asset_revision: u64,
        readiness_generation: u64,
        dependency_revision: u64,
        demand_generation: RenderAssetDemandGeneration,
        device: RenderAssetDeviceEpoch,
        scope: RenderAssetResidencyScope,
        route: RenderAssetResidencyRoute,
    ) -> Self {
        Self {
            id,
            resource,
            asset_revision,
            readiness_generation,
            dependency_revision,
            demand_generation,
            device,
            scope,
            route,
        }
    }

    pub(crate) const fn id(self) -> RenderAssetResidencyTicketId {
        self.id
    }

    pub(crate) const fn resource(self) -> UntypedResourceHandle {
        self.resource
    }

    pub(crate) const fn asset_revision(self) -> u64 {
        self.asset_revision
    }

    pub(crate) const fn readiness_generation(self) -> u64 {
        self.readiness_generation
    }

    pub(crate) const fn dependency_revision(self) -> u64 {
        self.dependency_revision
    }

    pub(crate) const fn demand_generation(self) -> RenderAssetDemandGeneration {
        self.demand_generation
    }

    pub(crate) const fn device(self) -> RenderAssetDeviceEpoch {
        self.device
    }

    pub(crate) const fn scope(self) -> RenderAssetResidencyScope {
        self.scope
    }

    pub(crate) const fn route(self) -> RenderAssetResidencyRoute {
        self.route
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderAssetResidencyState {
    QueuedIo,
    Reading,
    Decoding,
    ReadyCpu,
    QueuedUpload,
    Uploading,
    Resident,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderAssetResidencyReleaseKind {
    CancelPending,
    RetireInFlight,
    RetireResident,
    DropTerminal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderAssetResidencyRelease {
    ticket: RenderAssetResidencyTicket,
    kind: RenderAssetResidencyReleaseKind,
    submission: Option<SubmissionTicket>,
}

impl RenderAssetResidencyRelease {
    pub(super) const fn new(
        ticket: RenderAssetResidencyTicket,
        kind: RenderAssetResidencyReleaseKind,
        submission: Option<SubmissionTicket>,
    ) -> Self {
        Self {
            ticket,
            kind,
            submission,
        }
    }

    pub(crate) const fn ticket(self) -> RenderAssetResidencyTicket {
        self.ticket
    }

    pub(crate) const fn kind(self) -> RenderAssetResidencyReleaseKind {
        self.kind
    }

    pub(crate) const fn submission(self) -> Option<SubmissionTicket> {
        self.submission
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderAssetResidencyMutationStats {
    pub(super) input_delta_count: usize,
    pub(super) preflight_entry_lookup_count: usize,
    pub(super) catalog_lookup_count: usize,
    pub(super) request_count: usize,
    pub(super) release_count: usize,
}

impl RenderAssetResidencyMutationStats {
    pub(crate) const fn input_delta_count(self) -> usize {
        self.input_delta_count
    }

    pub(crate) const fn preflight_entry_lookup_count(self) -> usize {
        self.preflight_entry_lookup_count
    }

    pub(crate) const fn catalog_lookup_count(self) -> usize {
        self.catalog_lookup_count
    }

    pub(crate) const fn request_count(self) -> usize {
        self.request_count
    }

    pub(crate) const fn release_count(self) -> usize {
        self.release_count
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderAssetResidencyMutation {
    requests: Vec<RenderAssetResidencyTicket>,
    releases: Vec<RenderAssetResidencyRelease>,
    stats: RenderAssetResidencyMutationStats,
}

impl RenderAssetResidencyMutation {
    pub(super) fn from_parts(
        requests: Vec<RenderAssetResidencyTicket>,
        releases: Vec<RenderAssetResidencyRelease>,
        mut stats: RenderAssetResidencyMutationStats,
    ) -> Self {
        stats.request_count = requests.len();
        stats.release_count = releases.len();
        Self {
            requests,
            releases,
            stats,
        }
    }

    pub(crate) fn requests(&self) -> &[RenderAssetResidencyTicket] {
        &self.requests
    }

    pub(crate) fn releases(&self) -> &[RenderAssetResidencyRelease] {
        &self.releases
    }

    pub(crate) const fn stats(&self) -> RenderAssetResidencyMutationStats {
        self.stats
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderAssetResidencyAdmissionError {
    MalformedReferenceDelta {
        resource: UntypedResourceHandle,
        acquired_count: usize,
        released_count: usize,
    },
    DuplicateReferenceDelta {
        resource: UntypedResourceHandle,
    },
    ReferenceCountOverflow {
        resource: UntypedResourceHandle,
    },
    ReferenceCountUnderflow {
        resource: UntypedResourceHandle,
        current_count: usize,
        released_count: usize,
    },
    MissingCatalogRecord {
        resource: UntypedResourceHandle,
    },
    CatalogKindMismatch {
        resource: UntypedResourceHandle,
        catalog_kind: crate::core::resource::ResourceKind,
    },
    MissingReadinessRecord {
        resource: UntypedResourceHandle,
    },
    UnsupportedResourceKind {
        resource: UntypedResourceHandle,
    },
    DeviceEpochMismatch {
        expected: RenderAssetDeviceEpoch,
        actual: RenderAssetDeviceEpoch,
    },
    GpuRetirementBackpressure {
        ready_retirements: usize,
        requested_retirements: usize,
        limit: usize,
    },
    TicketIdExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderAssetResidencyTransitionError {
    UnknownTicket {
        presented: RenderAssetResidencyTicketId,
    },
    StaleTicket {
        presented: RenderAssetResidencyTicketId,
        current: RenderAssetResidencyTicketId,
    },
    InvalidTransition {
        from: RenderAssetResidencyState,
        to: RenderAssetResidencyState,
    },
    SubmissionDeviceMismatch {
        expected: RenderAssetDeviceEpoch,
        actual: RenderAssetDeviceEpoch,
    },
    SubmissionNotBound {
        ticket: RenderAssetResidencyTicketId,
    },
    UploadLeaseNotBound {
        ticket: RenderAssetResidencyTicketId,
    },
    SubmissionAlreadyTracked {
        submission: SubmissionTicket,
    },
    GpuTrackingBackpressure {
        tracked_submissions: usize,
        limit: usize,
    },
    GpuRetirementBackpressure {
        ready_retirements: usize,
        requested_retirements: usize,
        limit: usize,
    },
    SubmissionMismatch {
        expected: SubmissionTicket,
        actual: SubmissionTicket,
    },
    SubmissionNotTerminal {
        status: SubmissionStatus,
    },
}
