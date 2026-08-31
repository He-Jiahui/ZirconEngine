pub(super) struct SubmitContextSources {
    pub(super) context: &'static str,
    pub(super) build_context: &'static str,
    pub(super) camera_loop: &'static str,
    pub(super) frame_extract: &'static str,
    pub(super) collect_feedback: &'static str,
    pub(super) build_runtime_frame: &'static str,
    pub(super) submit_extract: &'static str,
    pub(super) present_frame_extract: &'static str,
    pub(super) submit_runtime_frame: &'static str,
    pub(super) record_submission: &'static str,
    pub(super) record_present_submission: &'static str,
    pub(super) record_camera_history: &'static str,
    pub(super) prepared_submission: &'static str,
    pub(super) prepared_runtime_sidebands: &'static str,
    pub(super) viewport_render_frame: &'static str,
    pub(super) viewport_render_frame_from_extract: &'static str,
    pub(super) runtime_07_plan: &'static str,
    pub(super) runtime_index: &'static str,
    pub(super) review_findings: &'static str,
}

impl SubmitContextSources {
    pub(super) fn load() -> Self {
        Self {
            context: include_str!(
                "../../../../graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs"
            ),
            build_context: include_str!(
                "../../../../graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs"
            ),
            camera_loop: include_str!(
                "../../../../graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs"
            ),
            frame_extract: include_str!("../../../../core/framework/render/frame_extract/frame.rs"),
            collect_feedback: include_str!(
                "../../../../graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs"
            ),
            build_runtime_frame: include_str!(
                "../../../../graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs"
            ),
            submit_extract: include_str!(
                "../../../../graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs"
            ),
            present_frame_extract: include_str!(
                "../../../../graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs"
            ),
            submit_runtime_frame: include_str!(
                "../../../../graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs"
            ),
            record_submission: include_str!(
                "../../../../graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs"
            ),
            record_present_submission: include_str!(
                "../../../../graphics/runtime/render_framework/submit_frame_extract/record_submission/record_present.rs"
            ),
            record_camera_history: include_str!(
                "../../../../graphics/runtime/render_framework/submit_frame_extract/submit/record_camera_history.rs"
            ),
            prepared_submission: include_str!(
                "../../../../graphics/runtime/render_framework/submit_frame_extract/prepared_runtime_submission.rs"
            ),
            prepared_runtime_sidebands: include_str!(
                "../../../../core/framework/render/prepared_runtime_sidebands.rs"
            ),
            viewport_render_frame: include_str!("../../../../graphics/types/viewport_render_frame.rs"),
            viewport_render_frame_from_extract: include_str!(
                "../../../../graphics/types/viewport_render_frame_from_extract.rs"
            ),
            runtime_07_plan: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md"
            ),
            runtime_index: include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-index-output-records.md"
            ),
            review_findings: include_str!(
                "../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
            ),
        }
    }
}
