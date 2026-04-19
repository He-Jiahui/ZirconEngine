use crate::core::framework::render::RenderFrameExtract;

use crate::graphics::EditorOrRuntimeFrame;

use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;

pub(super) fn build_runtime_frame(
    extract: RenderFrameExtract,
    context: &FrameSubmissionContext,
    prepared: &PreparedRuntimeSubmission,
) -> EditorOrRuntimeFrame {
    EditorOrRuntimeFrame::from_extract(extract, context.size)
        .with_hybrid_gi_prepare(prepared.hybrid_gi_prepare.clone())
        .with_hybrid_gi_resolve_runtime(prepared.hybrid_gi_resolve_runtime.clone())
        .with_virtual_geometry_prepare(prepared.virtual_geometry_prepare.clone())
}
