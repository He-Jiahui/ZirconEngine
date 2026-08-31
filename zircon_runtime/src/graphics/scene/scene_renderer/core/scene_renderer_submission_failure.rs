use crate::core::framework::render::{
    RenderFrameSubmissionReceipt, RenderFrameSubmissionTransaction,
};
use crate::graphics::backend::{
    RenderBackend, ViewportSurfacePresentFailure, ViewportSurfacePresentOutcome,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::GraphicsError;

pub(in crate::graphics::scene::scene_renderer::core) fn settle_failed_frame_submissions(
    backend: &RenderBackend,
    streamer: &mut ResourceStreamer,
    transaction: RenderFrameSubmissionTransaction,
    source: GraphicsError,
) -> GraphicsError {
    let scene_submission = source.submitted_scene_submission();
    let tickets = transaction.pre_scene_submission_tickets();
    if tickets.is_empty() && scene_submission.is_none() {
        return source;
    }
    let statuses = if tickets.is_empty() {
        Vec::new()
    } else {
        match backend.settle_abandoned_submissions(&tickets) {
            Ok(statuses) => statuses,
            Err(settlement) => {
                return GraphicsError::FrameSubmissionSettlement {
                    settlement: settlement.to_string(),
                    source: Box::new(source),
                };
            }
        }
    };
    let failure_receipt = match scene_submission {
        Some(scene_submission) => {
            transaction.abort_after_scene_submission(scene_submission, statuses)
        }
        None => transaction.abort(statuses),
    };
    match failure_receipt {
        Ok(receipt) => {
            streamer.rollback_failed_frame_submissions(&receipt);
            GraphicsError::FrameSubmissionFailed {
                receipt,
                source: Box::new(source),
            }
        }
        Err(settlement) => GraphicsError::FrameSubmissionSettlement {
            settlement: settlement.to_string(),
            source: Box::new(source),
        },
    }
}

pub(in crate::graphics::scene::scene_renderer::core) fn finalize_surface_presentation(
    receipt: RenderFrameSubmissionReceipt,
    present_result: Result<ViewportSurfacePresentOutcome, ViewportSurfacePresentFailure>,
) -> Result<RenderFrameSubmissionReceipt, GraphicsError> {
    let outcome = present_result.map_err(|failure| {
        let (source, present_submission) = failure.into_parts();
        GraphicsError::FramePresentationFailed {
            receipt: receipt.clone(),
            present_submission,
            source: Box::new(source),
        }
    })?;
    let Some(present_submission) = outcome.submission_ticket() else {
        return Ok(receipt);
    };
    let scene_receipt = receipt.clone();
    receipt
        .with_present_submission(present_submission)
        .map_err(|source| GraphicsError::FramePresentationFailed {
            receipt: scene_receipt,
            present_submission: Some(present_submission),
            source: Box::new(source.into()),
        })
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::RenderFrameSubmissionReceipt;
    use zr_rhi::{
        DeviceGeneration, DeviceId, RenderQueueClass, SubmissionPollReceipt, SubmissionTicket,
    };

    use super::finalize_surface_presentation;
    use crate::graphics::backend::{ViewportSurfacePresentFailure, ViewportSurfacePresentOutcome};
    use crate::graphics::types::GraphicsError;

    fn ticket(sequence: u64) -> SubmissionTicket {
        SubmissionTicket::new(
            DeviceId::new(3),
            DeviceGeneration::new(2),
            RenderQueueClass::Graphics,
            sequence,
        )
    }

    fn scene_receipt() -> RenderFrameSubmissionReceipt {
        RenderFrameSubmissionReceipt::new(
            7,
            SubmissionPollReceipt::new(DeviceId::new(3), DeviceGeneration::new(2), 11),
            ticket(40),
        )
        .expect("scene receipt")
    }

    #[test]
    fn frame_failure_owner_settles_the_transaction_without_a_second_ticket_table() {
        let source = include_str!("scene_renderer_submission_failure.rs");

        assert!(source.contains("transaction.pre_scene_submission_tickets()"));
        assert!(source.contains("backend.settle_abandoned_submissions(&tickets)"));
        assert!(source.contains("transaction.abort_after_scene_submission"));
        assert!(source.contains("None => transaction.abort(statuses)"));
        assert!(source.contains("streamer.rollback_failed_frame_submissions(&receipt)"));
        assert!(source.contains("GraphicsError::FrameSubmissionFailed"));
    }

    #[test]
    fn presentation_finalize_attaches_the_real_present_ticket() {
        let receipt = finalize_surface_presentation(
            scene_receipt(),
            Ok(ViewportSurfacePresentOutcome::Presented(ticket(40))),
        )
        .expect("present receipt");

        assert_eq!(receipt.present_submission(), Some(ticket(40)));
    }

    #[test]
    fn presentation_error_retains_the_scene_receipt() {
        let error = finalize_surface_presentation(
            scene_receipt(),
            Err(ViewportSurfacePresentFailure::before_submission(
                GraphicsError::SurfaceStatus("validation"),
            )),
        )
        .expect_err("surface error");

        assert_eq!(
            error
                .frame_submission_receipt()
                .map(RenderFrameSubmissionReceipt::scene_submission),
            Some(ticket(40))
        );
    }

    #[test]
    fn presentation_error_retains_a_post_submit_surface_ticket() {
        let error = finalize_surface_presentation(
            scene_receipt(),
            Err(ViewportSurfacePresentFailure::after_submission(
                GraphicsError::SurfaceStatus("present"),
                ticket(40),
            )),
        )
        .expect_err("surface error");

        assert!(matches!(
            error,
            GraphicsError::FramePresentationFailed {
                present_submission: Some(submission),
                ..
            } if submission == ticket(40)
        ));
    }
}
