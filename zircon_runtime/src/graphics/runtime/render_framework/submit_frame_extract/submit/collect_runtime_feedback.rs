use crate::{SceneRenderer, VirtualGeometryRuntimeFeedback};

use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::runtime_feedback_batch::RuntimeFeedbackBatch;

pub(super) fn collect_runtime_feedback(
    renderer: &mut SceneRenderer,
    context: &FrameSubmissionContext,
) -> RuntimeFeedbackBatch {
    RuntimeFeedbackBatch::new(collect_virtual_geometry_feedback(renderer, context))
}

fn collect_virtual_geometry_feedback(
    _renderer: &mut SceneRenderer,
    context: &FrameSubmissionContext,
) -> VirtualGeometryRuntimeFeedback {
    VirtualGeometryRuntimeFeedback::new(
        None,
        Vec::new(),
        context.virtual_geometry_feedback().cloned(),
        context.predicted_generation(),
    )
}
