use super::{execution_draw_ref_index, VirtualGeometrySubmissionDetail};

#[test]
fn execution_draw_ref_index_prefers_explicit_submission_detail_source() {
    let submission_detail = VirtualGeometrySubmissionDetail::new(
        Some(3),
        42,
        300,
        7,
        2,
        9,
        3,
        1,
        4,
        Some(5),
        crate::core::framework::render::RenderVirtualGeometryExecutionState::Resident,
        2,
        1,
        6,
    );

    assert_eq!(
        execution_draw_ref_index(
            Some(submission_detail),
            3 * super::INDIRECT_ARGS_STRIDE_BYTES
        ),
        9,
        "expected execution ownership to keep the authoritative draw-ref index emitted by the shared submission truth instead of reconstructing it from indirect args offsets"
    );
}

#[test]
fn execution_draw_ref_index_falls_back_to_indirect_args_offset_stride() {
    assert_eq!(
        execution_draw_ref_index(None, 4 * super::INDIRECT_ARGS_STRIDE_BYTES),
        4,
        "expected offset-based draw-ref recovery to remain available when explicit authoritative draw-ref truth is absent"
    );
}
