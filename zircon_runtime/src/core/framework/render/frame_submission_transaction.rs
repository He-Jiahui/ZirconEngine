use std::sync::Arc;

use zr_rhi::{SubmissionPollReceipt, SubmissionStatus, SubmissionTicket};

use crate::core::resource::ResourceId;

use super::{
    RenderFrameSubmissionBoundaryReason, RenderFrameSubmissionFailureReceipt,
    RenderFrameSubmissionFailureReceiptError, RenderFrameSubmissionProducer,
    RenderFrameSubmissionProducerRecord, RenderFrameSubmissionReceipt,
    RenderFrameSubmissionReceiptError,
};

/// Frame-boundary ledger for submissions accepted before the scene packet.
///
/// The ledger allocates only when a real pre-scene producer submits work.
/// Completion pumping and final receipt publication remain with the caller.
pub(crate) struct RenderFrameSubmissionTransaction {
    frame_generation: u64,
    poll: SubmissionPollReceipt,
    pre_scene_submissions: Vec<RenderFrameSubmissionProducerRecord>,
}

impl RenderFrameSubmissionTransaction {
    pub(crate) fn begin(frame_generation: u64, poll: SubmissionPollReceipt) -> Self {
        Self {
            frame_generation,
            poll,
            pre_scene_submissions: Vec::new(),
        }
    }

    pub(crate) fn record_pre_scene_submission(
        &mut self,
        producer: RenderFrameSubmissionProducer,
        ticket: SubmissionTicket,
    ) -> Result<(), RenderFrameSubmissionReceiptError> {
        self.record_pre_scene_submission_record(RenderFrameSubmissionProducerRecord::new(
            producer, ticket,
        ))
    }

    pub(crate) fn record_pre_scene_resource_submission(
        &mut self,
        producer: RenderFrameSubmissionProducer,
        resource_id: ResourceId,
        ticket: SubmissionTicket,
    ) -> Result<(), RenderFrameSubmissionReceiptError> {
        self.record_pre_scene_submission_record(RenderFrameSubmissionProducerRecord::for_resource(
            producer,
            resource_id,
            ticket,
        ))
    }

    pub(crate) fn record_pre_scene_resource_submission_with_boundary(
        &mut self,
        producer: RenderFrameSubmissionProducer,
        resource_id: ResourceId,
        boundary_reason: RenderFrameSubmissionBoundaryReason,
        ticket: SubmissionTicket,
    ) -> Result<(), RenderFrameSubmissionReceiptError> {
        self.record_pre_scene_submission_record(
            RenderFrameSubmissionProducerRecord::for_resource_boundary(
                producer,
                resource_id,
                boundary_reason,
                ticket,
            ),
        )
    }

    fn record_pre_scene_submission_record(
        &mut self,
        record: RenderFrameSubmissionProducerRecord,
    ) -> Result<(), RenderFrameSubmissionReceiptError> {
        let producer = record.producer();
        if let Some(boundary_reason) = record.mismatched_boundary_reason() {
            return Err(
                RenderFrameSubmissionReceiptError::BoundaryReasonProducerMismatch {
                    producer,
                    boundary_reason,
                },
            );
        }
        let ticket = record.ticket();
        if ticket.device_id() != self.poll.device_id()
            || ticket.generation() != self.poll.generation()
        {
            return Err(RenderFrameSubmissionReceiptError::ProducerOwnerMismatch {
                producer,
                producer_device: ticket.device_id(),
                producer_generation: ticket.generation(),
                poll_device: self.poll.device_id(),
                poll_generation: self.poll.generation(),
            });
        }
        if let Some(previous) = self.pre_scene_submissions.last() {
            if ticket.sequence() <= previous.ticket().sequence() {
                return Err(
                    RenderFrameSubmissionReceiptError::ProducerSequenceDidNotAdvance {
                        previous_sequence: previous.ticket().sequence(),
                        producer_sequence: ticket.sequence(),
                    },
                );
            }
        }
        self.pre_scene_submissions.push(record);
        Ok(())
    }

    pub(crate) fn finish(
        self,
        scene: SubmissionTicket,
    ) -> Result<RenderFrameSubmissionReceipt, RenderFrameSubmissionReceiptError> {
        RenderFrameSubmissionReceipt::from_transaction(
            self.frame_generation,
            self.poll,
            scene,
            (!self.pre_scene_submissions.is_empty()).then(|| Arc::from(self.pre_scene_submissions)),
        )
    }

    pub(crate) fn validate_scene_submission(
        &self,
        scene: SubmissionTicket,
    ) -> Result<(), RenderFrameSubmissionReceiptError> {
        RenderFrameSubmissionReceipt::validate_transaction(
            self.poll,
            scene,
            &self.pre_scene_submissions,
        )
    }

    /// Copies the dense ticket list only for the cold failure settlement path.
    pub(crate) fn pre_scene_submission_tickets(&self) -> Vec<SubmissionTicket> {
        self.pre_scene_submissions
            .iter()
            .map(|record| record.ticket())
            .collect()
    }

    pub(crate) fn abort(
        self,
        statuses: Vec<SubmissionStatus>,
    ) -> Result<RenderFrameSubmissionFailureReceipt, RenderFrameSubmissionFailureReceiptError> {
        RenderFrameSubmissionFailureReceipt::from_transaction(
            self.frame_generation,
            self.poll,
            self.pre_scene_submissions,
            statuses,
            None,
        )
    }

    pub(crate) fn abort_after_scene_submission(
        self,
        scene_submission: SubmissionTicket,
        statuses: Vec<SubmissionStatus>,
    ) -> Result<RenderFrameSubmissionFailureReceipt, RenderFrameSubmissionFailureReceiptError> {
        RenderFrameSubmissionFailureReceipt::from_transaction(
            self.frame_generation,
            self.poll,
            self.pre_scene_submissions,
            statuses,
            Some(scene_submission),
        )
    }
}

#[cfg(test)]
#[path = "frame_submission_transaction/tests.rs"]
mod tests;
