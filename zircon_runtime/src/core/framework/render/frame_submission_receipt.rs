use std::sync::Arc;

use thiserror::Error;
use zr_rhi::{DeviceGeneration, DeviceId, SubmissionPollReceipt, SubmissionTicket};

use super::{
    RenderFrameSubmissionBoundaryReason, RenderFrameSubmissionMetrics,
    RenderFrameSubmissionProducer, RenderFrameSubmissionProducerRecord,
};

/// Device-generation-qualified identity for one product render frame.
///
/// The scene ticket is always present. Pre-scene producer tickets are retained
/// only when work exists. A viewport-product ticket may share the scene packet when its copy lives
/// in the scene tail. A present ticket may also share that packet when the acquired surface blit is
/// fused before the packet flush.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFrameSubmissionReceipt {
    frame_generation: u64,
    poll: SubmissionPollReceipt,
    pre_scene_submissions: Option<Arc<[RenderFrameSubmissionProducerRecord]>>,
    scene: SubmissionTicket,
    viewport_product: Option<SubmissionTicket>,
    present: Option<SubmissionTicket>,
    submission_metrics: Option<RenderFrameSubmissionMetrics>,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RenderFrameSubmissionReceiptError {
    #[error(
        "frame poll owner {poll_device:?}/{poll_generation:?} does not match scene submission owner {scene_device:?}/{scene_generation:?}"
    )]
    PollOwnerMismatch {
        poll_device: DeviceId,
        poll_generation: DeviceGeneration,
        scene_device: DeviceId,
        scene_generation: DeviceGeneration,
    },
    #[error(
        "{producer:?} submission owner {producer_device:?}/{producer_generation:?} does not match frame poll owner {poll_device:?}/{poll_generation:?}"
    )]
    ProducerOwnerMismatch {
        producer: RenderFrameSubmissionProducer,
        producer_device: DeviceId,
        producer_generation: DeviceGeneration,
        poll_device: DeviceId,
        poll_generation: DeviceGeneration,
    },
    #[error("{boundary_reason:?} cannot annotate {producer:?} frame work")]
    BoundaryReasonProducerMismatch {
        producer: RenderFrameSubmissionProducer,
        boundary_reason: RenderFrameSubmissionBoundaryReason,
    },
    #[error(
        "pre-scene producer sequence {producer_sequence} must advance beyond prior producer sequence {previous_sequence}"
    )]
    ProducerSequenceDidNotAdvance {
        previous_sequence: u64,
        producer_sequence: u64,
    },
    #[error(
        "{producer:?} submission sequence {producer_sequence} must precede scene submission sequence {scene_sequence}"
    )]
    ProducerDidNotPrecedeScene {
        producer: RenderFrameSubmissionProducer,
        producer_sequence: u64,
        scene_sequence: u64,
    },
    #[error(
        "viewport-product submission owner {viewport_product_device:?}/{viewport_product_generation:?} does not match scene submission owner {scene_device:?}/{scene_generation:?}"
    )]
    ViewportProductOwnerMismatch {
        viewport_product_device: DeviceId,
        viewport_product_generation: DeviceGeneration,
        scene_device: DeviceId,
        scene_generation: DeviceGeneration,
    },
    #[error(
        "viewport-product submission sequence {viewport_product_sequence} cannot precede scene submission sequence {scene_sequence}"
    )]
    ViewportProductPrecededScene {
        scene_sequence: u64,
        viewport_product_sequence: u64,
    },
    #[error(
        "published viewport-product submission {published:?} does not match frame receipt submission {recorded:?}"
    )]
    ViewportProductIdentityMismatch {
        recorded: Option<SubmissionTicket>,
        published: SubmissionTicket,
    },
    #[error(
        "viewport-product frame generation {viewport_product_generation} does not match render frame generation {frame_generation}"
    )]
    ViewportProductFrameGenerationMismatch {
        frame_generation: u64,
        viewport_product_generation: u64,
    },
    #[error(
        "present submission owner {present_device:?}/{present_generation:?} does not match scene submission owner {scene_device:?}/{scene_generation:?}"
    )]
    PresentOwnerMismatch {
        present_device: DeviceId,
        present_generation: DeviceGeneration,
        scene_device: DeviceId,
        scene_generation: DeviceGeneration,
    },
    #[error(
        "present submission sequence {present_sequence} cannot precede scene submission sequence {scene_sequence}"
    )]
    PresentPrecededScene {
        scene_sequence: u64,
        present_sequence: u64,
    },
    #[error(
        "present submission sequence {present_sequence} cannot precede viewport-product submission sequence {viewport_product_sequence}"
    )]
    PresentPrecededViewportProduct {
        viewport_product_sequence: u64,
        present_sequence: u64,
    },
}

impl RenderFrameSubmissionReceipt {
    pub fn new(
        frame_generation: u64,
        poll: SubmissionPollReceipt,
        scene: SubmissionTicket,
    ) -> Result<Self, RenderFrameSubmissionReceiptError> {
        Self::from_transaction(frame_generation, poll, scene, None)
    }

    pub(crate) fn from_transaction(
        frame_generation: u64,
        poll: SubmissionPollReceipt,
        scene: SubmissionTicket,
        pre_scene_submissions: Option<Arc<[RenderFrameSubmissionProducerRecord]>>,
    ) -> Result<Self, RenderFrameSubmissionReceiptError> {
        Self::validate_transaction(
            poll,
            scene,
            pre_scene_submissions.as_deref().unwrap_or_default(),
        )?;
        Ok(Self {
            frame_generation,
            poll,
            pre_scene_submissions,
            scene,
            viewport_product: None,
            present: None,
            submission_metrics: None,
        })
    }

    pub(crate) fn validate_transaction(
        poll: SubmissionPollReceipt,
        scene: SubmissionTicket,
        pre_scene_submissions: &[RenderFrameSubmissionProducerRecord],
    ) -> Result<(), RenderFrameSubmissionReceiptError> {
        if poll.device_id() != scene.device_id() || poll.generation() != scene.generation() {
            return Err(RenderFrameSubmissionReceiptError::PollOwnerMismatch {
                poll_device: poll.device_id(),
                poll_generation: poll.generation(),
                scene_device: scene.device_id(),
                scene_generation: scene.generation(),
            });
        }
        for record in pre_scene_submissions {
            if let Some(boundary_reason) = record.mismatched_boundary_reason() {
                return Err(
                    RenderFrameSubmissionReceiptError::BoundaryReasonProducerMismatch {
                        producer: record.producer(),
                        boundary_reason,
                    },
                );
            }
            if record.ticket().sequence() >= scene.sequence() {
                return Err(
                    RenderFrameSubmissionReceiptError::ProducerDidNotPrecedeScene {
                        producer: record.producer(),
                        producer_sequence: record.ticket().sequence(),
                        scene_sequence: scene.sequence(),
                    },
                );
            }
        }
        Ok(())
    }

    pub fn with_present_submission(
        mut self,
        present: SubmissionTicket,
    ) -> Result<Self, RenderFrameSubmissionReceiptError> {
        if present.device_id() != self.scene.device_id()
            || present.generation() != self.scene.generation()
        {
            return Err(RenderFrameSubmissionReceiptError::PresentOwnerMismatch {
                present_device: present.device_id(),
                present_generation: present.generation(),
                scene_device: self.scene.device_id(),
                scene_generation: self.scene.generation(),
            });
        }
        if present.sequence() < self.scene.sequence() {
            return Err(RenderFrameSubmissionReceiptError::PresentPrecededScene {
                scene_sequence: self.scene.sequence(),
                present_sequence: present.sequence(),
            });
        }
        if let Some(viewport_product) = self.viewport_product {
            if present.sequence() < viewport_product.sequence() {
                return Err(
                    RenderFrameSubmissionReceiptError::PresentPrecededViewportProduct {
                        viewport_product_sequence: viewport_product.sequence(),
                        present_sequence: present.sequence(),
                    },
                );
            }
        }
        self.present = Some(present);
        Ok(self)
    }

    pub fn with_viewport_product_submission(
        mut self,
        viewport_product: SubmissionTicket,
    ) -> Result<Self, RenderFrameSubmissionReceiptError> {
        if viewport_product.device_id() != self.scene.device_id()
            || viewport_product.generation() != self.scene.generation()
        {
            return Err(
                RenderFrameSubmissionReceiptError::ViewportProductOwnerMismatch {
                    viewport_product_device: viewport_product.device_id(),
                    viewport_product_generation: viewport_product.generation(),
                    scene_device: self.scene.device_id(),
                    scene_generation: self.scene.generation(),
                },
            );
        }
        if viewport_product.sequence() < self.scene.sequence() {
            return Err(
                RenderFrameSubmissionReceiptError::ViewportProductPrecededScene {
                    scene_sequence: self.scene.sequence(),
                    viewport_product_sequence: viewport_product.sequence(),
                },
            );
        }
        if let Some(present) = self.present {
            if viewport_product.sequence() > present.sequence() {
                return Err(
                    RenderFrameSubmissionReceiptError::PresentPrecededViewportProduct {
                        viewport_product_sequence: viewport_product.sequence(),
                        present_sequence: present.sequence(),
                    },
                );
            }
        }
        self.viewport_product = Some(viewport_product);
        Ok(self)
    }

    pub const fn frame_generation(&self) -> u64 {
        self.frame_generation
    }

    pub const fn poll(&self) -> SubmissionPollReceipt {
        self.poll
    }

    pub fn pre_scene_submissions(&self) -> &[RenderFrameSubmissionProducerRecord] {
        self.pre_scene_submissions.as_deref().unwrap_or_default()
    }

    pub const fn scene_submission(&self) -> SubmissionTicket {
        self.scene
    }

    pub const fn viewport_product_submission(&self) -> Option<SubmissionTicket> {
        self.viewport_product
    }

    pub fn validate_viewport_product_publication(
        &self,
        viewport_product_generation: u64,
        published: SubmissionTicket,
    ) -> Result<(), RenderFrameSubmissionReceiptError> {
        if viewport_product_generation != self.frame_generation {
            return Err(
                RenderFrameSubmissionReceiptError::ViewportProductFrameGenerationMismatch {
                    frame_generation: self.frame_generation,
                    viewport_product_generation,
                },
            );
        }
        if self.viewport_product != Some(published) {
            return Err(
                RenderFrameSubmissionReceiptError::ViewportProductIdentityMismatch {
                    recorded: self.viewport_product,
                    published,
                },
            );
        }
        Ok(())
    }

    pub const fn present_submission(&self) -> Option<SubmissionTicket> {
        self.present
    }

    /// Counts frame-owned logical packets. Product publication and present do not add packets when
    /// they share the scene ticket.
    pub fn logical_packet_count(&self) -> u64 {
        u64::try_from(self.pre_scene_submissions().len())
            .unwrap_or(u64::MAX)
            .saturating_add(1)
    }

    pub(crate) fn with_submission_metrics(
        mut self,
        submission_metrics: Option<RenderFrameSubmissionMetrics>,
    ) -> Self {
        self.submission_metrics = submission_metrics;
        self
    }

    pub const fn submission_metrics(&self) -> Option<RenderFrameSubmissionMetrics> {
        self.submission_metrics
    }
}

#[cfg(test)]
#[path = "frame_submission_receipt/tests.rs"]
mod tests;
