use zr_rhi::SubmissionTicket;

use crate::core::resource::ResourceId;

use super::RenderFrameSubmissionBoundaryReason;

/// 场景命令包之前的工作来源；类别标识用途，不代表独立的物理提交边界。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RenderFrameSubmissionProducer {
    TexturePreUpload,
    TextureCopyUpload,
    TexturePostUpload,
    FrameResourceUpload,
}

/// 提交事务记录的生产者票据；仅纹理旧 mip 保留操作可携带强制边界原因。
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
