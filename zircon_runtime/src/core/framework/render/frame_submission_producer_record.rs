use zr_rhi::SubmissionTicket;

use crate::core::resource::ResourceId;

use super::RenderFrameSubmissionBoundaryReason;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RenderFrameSubmissionProducer {
    TexturePreUpload,
    TextureCopyUpload,
    TexturePostUpload,
    FrameResourceUpload,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderFrameSubmissionProducerRecord {
    producer: RenderFrameSubmissionProducer,
    resource_id: Option<ResourceId>,
    boundary_reason: Option<RenderFrameSubmissionBoundaryReason>,
    ticket: SubmissionTicket,
}

impl RenderFrameSubmissionProducerRecord {
    pub const fn new(producer: RenderFrameSubmissionProducer, ticket: SubmissionTicket) -> Self {
        Self {
            producer,
            resource_id: None,
            boundary_reason: None,
            ticket,
        }
    }

    pub const fn for_resource(
        producer: RenderFrameSubmissionProducer,
        resource_id: ResourceId,
        ticket: SubmissionTicket,
    ) -> Self {
        Self {
            producer,
            resource_id: Some(resource_id),
            boundary_reason: None,
            ticket,
        }
    }

    pub const fn for_resource_boundary(
        producer: RenderFrameSubmissionProducer,
        resource_id: ResourceId,
        boundary_reason: RenderFrameSubmissionBoundaryReason,
        ticket: SubmissionTicket,
    ) -> Self {
        Self {
            producer,
            resource_id: Some(resource_id),
            boundary_reason: Some(boundary_reason),
            ticket,
        }
    }

    pub const fn producer(self) -> RenderFrameSubmissionProducer {
        self.producer
    }

    pub const fn resource_id(self) -> Option<ResourceId> {
        self.resource_id
    }

    pub const fn boundary_reason(self) -> Option<RenderFrameSubmissionBoundaryReason> {
        self.boundary_reason
    }

    pub const fn ticket(self) -> SubmissionTicket {
        self.ticket
    }

    pub(crate) const fn mismatched_boundary_reason(
        self,
    ) -> Option<RenderFrameSubmissionBoundaryReason> {
        match (self.producer, self.boundary_reason) {
            (_, None)
            | (
                RenderFrameSubmissionProducer::TexturePreUpload,
                Some(RenderFrameSubmissionBoundaryReason::TextureMipPreservationBeforeUpload),
            ) => None,
            (_, boundary_reason) => boundary_reason,
        }
    }
}
