use std::sync::Arc;

use thiserror::Error;
use zr_rhi::{
    DeviceGeneration, DeviceId, SubmissionPollReceipt, SubmissionStatus, SubmissionTicket,
};

use super::{
    RenderFrameSubmissionBoundaryReason, RenderFrameSubmissionProducer,
    RenderFrameSubmissionProducerRecord,
};
use crate::core::resource::ResourceId;

/// 失败帧中已结算的前场景提交，保留生产者、资源及真实后端状态以供回滚。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderFrameSubmissionFailureRecord {
    producer: RenderFrameSubmissionProducer,
    resource_id: Option<ResourceId>,
    boundary_reason: Option<RenderFrameSubmissionBoundaryReason>,
    ticket: SubmissionTicket,
    status: SubmissionStatus,
}

impl RenderFrameSubmissionFailureRecord {
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

    pub const fn status(self) -> SubmissionStatus {
        self.status
    }
}

/// Settled identity retained when a product frame fails before scene submission.
/// 帧失败时保留已结算的前场景提交；若场景提交后收尾失败，也保留场景票据。
/// 资源流送器按这份身份回滚，不能把已提交的 GPU 工作当作从未发生。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFrameSubmissionFailureReceipt {
    frame_generation: u64,
    poll: SubmissionPollReceipt,
    pre_scene_submissions: Option<Arc<[RenderFrameSubmissionFailureRecord]>>,
    scene_submission: Option<SubmissionTicket>,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RenderFrameSubmissionFailureReceiptError {
    #[error(
        "frame failure settlement returned {status_count} statuses for {submission_count} submissions"
    )]
    StatusCountMismatch {
        submission_count: usize,
        status_count: usize,
    },
    #[error("{producer:?} submission {ticket:?} remained accepted after frame failure settlement")]
    SubmissionRemainedAccepted {
        producer: RenderFrameSubmissionProducer,
        ticket: SubmissionTicket,
    },
    #[error(
        "frame poll owner {poll_device:?}/{poll_generation:?} does not match submitted scene owner {scene_device:?}/{scene_generation:?}"
    )]
    SceneOwnerMismatch {
        poll_device: DeviceId,
        poll_generation: DeviceGeneration,
        scene_device: DeviceId,
        scene_generation: DeviceGeneration,
    },
    #[error(
        "{producer:?} submission sequence {producer_sequence} must precede failed frame scene submission sequence {scene_sequence}"
    )]
    ProducerDidNotPrecedeScene {
        producer: RenderFrameSubmissionProducer,
        producer_sequence: u64,
        scene_sequence: u64,
    },
}

impl RenderFrameSubmissionFailureReceipt {
    pub(crate) fn from_transaction(
        frame_generation: u64,
        poll: SubmissionPollReceipt,
        submissions: Vec<RenderFrameSubmissionProducerRecord>,
        statuses: Vec<SubmissionStatus>,
        scene_submission: Option<SubmissionTicket>,
    ) -> Result<Self, RenderFrameSubmissionFailureReceiptError> {
        if submissions.len() != statuses.len() {
            return Err(
                RenderFrameSubmissionFailureReceiptError::StatusCountMismatch {
                    submission_count: submissions.len(),
                    status_count: statuses.len(),
                },
            );
        }

        let mut settled = Vec::with_capacity(submissions.len());
        for (submission, status) in submissions.into_iter().zip(statuses) {
            if status == SubmissionStatus::Accepted {
                return Err(
                    RenderFrameSubmissionFailureReceiptError::SubmissionRemainedAccepted {
                        producer: submission.producer(),
                        ticket: submission.ticket(),
                    },
                );
            }
            settled.push(RenderFrameSubmissionFailureRecord {
                producer: submission.producer(),
                resource_id: submission.resource_id(),
                boundary_reason: submission.boundary_reason(),
                ticket: submission.ticket(),
                status,
            });
        }
        if let Some(scene) = scene_submission {
            if scene.device_id() != poll.device_id() || scene.generation() != poll.generation() {
                return Err(
                    RenderFrameSubmissionFailureReceiptError::SceneOwnerMismatch {
                        poll_device: poll.device_id(),
                        poll_generation: poll.generation(),
                        scene_device: scene.device_id(),
                        scene_generation: scene.generation(),
                    },
                );
            }
            for producer in &settled {
                if producer.ticket().sequence() >= scene.sequence() {
                    return Err(
                        RenderFrameSubmissionFailureReceiptError::ProducerDidNotPrecedeScene {
                            producer: producer.producer(),
                            producer_sequence: producer.ticket().sequence(),
                            scene_sequence: scene.sequence(),
                        },
                    );
                }
            }
        }

        Ok(Self {
            frame_generation,
            poll,
            pre_scene_submissions: (!settled.is_empty()).then(|| Arc::from(settled)),
            scene_submission,
        })
    }

    pub const fn frame_generation(&self) -> u64 {
        self.frame_generation
    }

    pub const fn poll(&self) -> SubmissionPollReceipt {
        self.poll
    }

    pub fn pre_scene_submissions(&self) -> &[RenderFrameSubmissionFailureRecord] {
        self.pre_scene_submissions.as_deref().unwrap_or_default()
    }

    /// Scene ticket returned by the submission owner before post-submit finalization failed.
    pub const fn scene_submission(&self) -> Option<SubmissionTicket> {
        self.scene_submission
    }
}

#[cfg(test)]
#[path = "tests/frame_submission_failure_receipt.rs"]
mod tests;
