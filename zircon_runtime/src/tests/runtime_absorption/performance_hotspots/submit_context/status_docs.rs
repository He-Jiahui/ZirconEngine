use super::sources::SubmitContextSources;

pub(super) fn assert_submit_context_status_docs(sources: &SubmitContextSources) {
    let runtime_07_plan = sources.runtime_07_plan;
    let runtime_index = sources.runtime_index;
    let review_findings = sources.review_findings;

    for status_anchor in [
        "Runtime 07 render submit source-extract sharing",
        "Runtime 07 render camera-loop descriptor submissions",
        "Runtime 07 render camera-loop borrowed sequence resolution",
        "Runtime 07 render camera-loop frame terminal move",
        "Runtime 07 render camera-loop post-process source restore narrowing",
        "Runtime 07 render camera-loop VG/HGI conditional source restore",
        "Runtime 07 render camera-loop single-child source-state capture skip",
        "Runtime 07 render camera-loop source payload slot ownership",
        "Runtime 07 render submit feedback sideband owned merge",
        "Runtime 07 render prepared sideband frame owner move",
        "Runtime 07 render direct runtime-frame streaming camera loop",
        "Runtime 07 render generated camera-loop shared extract",
        "Runtime 07 render shared effective extract frame source",
        "Runtime 07 render direct runtime-frame shared context extract",
        "Runtime 07 render VG debug overlay frame override",
        "render_submit_source_extract_shared_coremin_check_passed_partial",
        "render_camera_loop_descriptor_submissions_coremin_check_passed_partial",
        "render_camera_loop_borrowed_sequence_resolution_static_passed_cargo_deferred",
        "render_camera_loop_source_view_restore_narrowed_static_passed_cargo_deferred",
        "render_camera_loop_post_process_restore_narrowed_static_passed_cargo_deferred",
        "render_camera_loop_vg_hgi_conditional_restore_static_passed_cargo_deferred",
        "render_camera_loop_single_child_source_state_capture_skipped_static_passed_cargo_deferred",
        "render_camera_loop_source_payload_slot_owned_static_passed_cargo_deferred",
        "render_camera_loop_frame_terminal_move_coremin_check_passed_partial",
        "render_submit_feedback_sidebands_owned_merge_coremin_check_passed_partial",
        "render_prepared_sideband_frame_owner_move_coremin_check_passed_partial",
        "render_direct_runtime_frame_streaming_camera_loop_coremin_check_passed_partial",
        "render_generated_camera_loop_shared_extract_static_passed_cargo_locked_blocked",
        "render_shared_effective_extract_frame_source_coremin_check_passed_partial",
        "render_direct_runtime_frame_shared_context_extract_coremin_check_passed_partial",
        "render_vg_debug_overlay_frame_override_coremin_check_passed_partial",
        "source_extract: Arc<RenderFrameExtract>",
        "ViewportRenderFrame::from_shared_extract",
        "stream_camera_loop_extract_submissions",
        "CameraLoopExtractSourceState",
        "CameraLoopPostProcessSourceState",
        "FrameSubmissionSourcePayloads",
        "VisibilityContext::from_extract_with_history_static_index_task_pool_and_feature_payloads",
        "FrameHistoryValidationKey::from_extract_with_hybrid_gi",
        "resolve_camera_sequence_borrowed",
        "view_target_size: Option<UVec2>",
        "build_frame_submission_context_from_runtime_frame_extract",
        "runtime_overlay_override",
        "runtime_07_submit_context_shares_large_extract_payloads",
    ] {
        assert!(
            runtime_07_plan.contains(status_anchor)
                || runtime_index.contains(status_anchor)
                || review_findings.contains(status_anchor),
            "Runtime 07/F3 docs should record source-extract sharing anchor `{status_anchor}`"
        );
    }
}
