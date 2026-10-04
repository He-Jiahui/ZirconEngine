use super::*;

#[test]
fn frame_input_changes_invalidate_content_without_reallocating_history_textures() {
    assert!(!history_invalidation_requires_reallocation(Some(
        FrameHistoryInvalidationReason::FrameInputsChanged
    )));
    assert!(!history_invalidation_requires_reallocation(Some(
        FrameHistoryInvalidationReason::CameraCut
    )));
    assert!(!history_invalidation_requires_reallocation(None));
    assert!(history_invalidation_requires_reallocation(Some(
        FrameHistoryInvalidationReason::RenderSizeChanged
    )));
    assert!(history_invalidation_requires_reallocation(Some(
        FrameHistoryInvalidationReason::PipelineChanged
    )));
}

#[test]
fn history_resolution_reuses_context_invalidation_without_recomparing_history() {
    let source = include_str!("../resolve_history_handle.rs");

    assert!(source.contains("context.history_invalidation_reason()"));
    assert!(!source.contains(concat!("history", ".incompatibility_reason")));
}
