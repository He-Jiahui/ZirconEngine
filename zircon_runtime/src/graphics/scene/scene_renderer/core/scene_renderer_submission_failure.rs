use crate::core::framework::render::{
    RenderFrameSubmissionReceipt, RenderFrameSubmissionTransaction,
};
use crate::graphics::backend::{
    RenderBackend, ViewportSurfacePresentFailure, ViewportSurfacePresentOutcome,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::GraphicsError;

/// 场景构建中途失败时，由唯一帧事务收集已提交的上传票据并回滚 streamer 的帧状态。
/// 调用方必须传入原事务与原始错误，避免漏报 scene submission 已发生的情况。
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

/// 将真实的 present ticket 附到已存在的 scene receipt；呈现失败仍保留场景提交证据。
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
#[path = "tests/scene_renderer_submission_failure.rs"]
mod tests;
