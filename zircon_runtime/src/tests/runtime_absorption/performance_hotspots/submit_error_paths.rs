#[test]
fn runtime_07_submit_paths_return_errors_for_checked_viewport_records() {
    let submit_extract = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs"
    );
    let submit_runtime_frame = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs"
    );
    let present_frame_extract = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs"
    );
    let prepare_runtime_submission = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs"
    );

    for (label, source) in [
        ("submit_frame_extract", submit_extract),
        ("submit_runtime_frame", submit_runtime_frame),
        ("present_frame_extract", present_frame_extract),
    ] {
        assert!(
            !source.contains(".expect(\"viewport generation checked above\")"),
            "{label} should return RenderFrameworkError instead of panicking after viewport generation validation"
        );
        assert!(
            source.contains(
                "viewport_record_mut_after_generation_check(&mut state, viewport, &context)?"
            ),
            "{label} should use the shared checked-record helper"
        );
    }

    assert!(
        !prepare_runtime_submission
            .contains(".expect(\"viewport generation checked before runtime prepare\")"),
        "prepare_runtime_submission should return RenderFrameworkError instead of panicking when the checked viewport record disappears"
    );
    assert!(
        prepare_runtime_submission.contains("missing_runtime_provider("),
        "runtime prepare should report enabled-but-missing advanced providers as RenderFrameworkError"
    );
}
