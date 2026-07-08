use super::sources::SubmitContextSources;

pub(super) fn assert_camera_loop_uses_shared_sources(sources: &SubmitContextSources) {
    let camera_loop = sources.camera_loop;
    let frame_extract = sources.frame_extract;
    let submit_runtime_frame = sources.submit_runtime_frame;

    let camera_loop_submission_body = camera_loop
        .split("fn camera_loop_submissions(")
        .nth(1)
        .and_then(|tail| {
            tail.split("pub(super) fn camera_loop_frame_submissions")
                .next()
        })
        .expect("camera_loop_submissions body should remain visible to the guard");
    assert!(
        !camera_loop_submission_body.contains("with_selected_camera_descriptor"),
        "camera_loop_submissions should enumerate descriptors instead of cloning RenderFrameExtract"
    );
    assert!(
        !camera_loop_submission_body.contains("resolve_camera_sequence(extract.view.cameras.clone())"),
        "camera_loop_submissions should borrow RenderViewExtract.cameras before cloning only the final descriptor sequence"
    );
    let submit_camera_loop_body = camera_loop
        .split("pub(super) fn submit_camera_loop(")
        .nth(1)
        .and_then(|tail| {
            tail.split("pub(super) fn viewport_terminal_camera_target")
                .next()
        })
        .expect("submit_camera_loop body should remain visible to the guard");
    assert!(
        !submit_camera_loop_body.contains(".clone()\n                .with_selected_camera_descriptor"),
        "submit_camera_loop should stream a shared source extract instead of cloning it for child cameras"
    );
    assert!(
        !submit_camera_loop_body.contains("let mut source_extract = Some(extract);")
            && !submit_camera_loop_body.contains("source_extract.take()"),
        "submit_camera_loop should not restore the old owned-extract terminal move model"
    );
    for required_camera_loop_anchor in [
        "struct CameraLoopSubmission {",
        "camera: CameraRenderDescriptor,",
        "resolve_camera_sequence_borrowed(&extract.view.cameras)",
        ".map(CameraLoopSubmission::from)",
        "stream_camera_loop_extract_submissions(",
        "let mut source_extract = Arc::new(extract);",
        "Some(CameraLoopExtractSourceState::capture(source))",
        "for (submission_index, submission) in submissions.into_iter().enumerate()",
        "if submission_index > 0",
        "view_target_size: Option<crate::core::math::UVec2>",
        "extract.view.target_size = self.view_target_size",
        "post_process: CameraLoopPostProcessSourceState",
        "CameraLoopPostProcessSourceState::capture(&extract.post_process)",
        "self.post_process.restore_to(&mut extract.post_process)",
        "virtual_geometry: extract.geometry.virtual_geometry.take()",
        "hybrid_global_illumination: extract.lighting.hybrid_global_illumination.take()",
        "fn source_payloads(&self) -> FrameSubmissionSourcePayloads<'_>",
        "FrameSubmissionSourcePayloads {",
        "Arc::make_mut(&mut source_extract)",
        "source_state.restore_for_submission(extract)",
        "extract.select_camera_descriptor(submission.camera)",
        "submit_selected_camera(\n            &mut source_extract,",
        ".map(|submission| submission.camera.target.clone())",
    ] {
        assert!(
            camera_loop.contains(required_camera_loop_anchor),
            "Runtime 07 F3 camera-loop descriptor sharing should retain anchor `{required_camera_loop_anchor}`"
        );
    }
    for forbidden_camera_loop_view_restore in [
        "view: RenderViewExtract",
        "view: extract.view.clone()",
        "extract.view = self.view.clone()",
    ] {
        assert!(
            !camera_loop.contains(forbidden_camera_loop_view_restore),
            "CameraLoopExtractSourceState should not restore full RenderViewExtract clone `{forbidden_camera_loop_view_restore}`"
        );
    }
    for forbidden_camera_loop_post_process_restore in [
        "post_process: PostProcessExtract",
        "post_process: extract.post_process.clone()",
        "post_process: frame.extract.post_process.clone()",
        "extract.post_process = self.post_process.clone()",
    ] {
        assert!(
            !camera_loop.contains(forbidden_camera_loop_post_process_restore),
            "CameraLoopSourceState should not restore full PostProcessExtract clone `{forbidden_camera_loop_post_process_restore}`"
        );
    }
    for forbidden_camera_loop_feature_payload_restore in [
        "extract.geometry.virtual_geometry = self.virtual_geometry.clone()",
        "extract.lighting.hybrid_global_illumination = self.hybrid_global_illumination.clone()",
        "virtual_geometry: extract.geometry.virtual_geometry.clone()",
        "hybrid_global_illumination: extract.lighting.hybrid_global_illumination.clone()",
        "virtual_geometry: frame.extract.geometry.virtual_geometry.clone()",
        "hybrid_global_illumination: frame.extract.lighting.hybrid_global_illumination.clone()",
    ] {
        assert!(
            !camera_loop.contains(forbidden_camera_loop_feature_payload_restore),
            "CameraLoopSourceState should not restore feature payloads through unconditional clone `{forbidden_camera_loop_feature_payload_restore}`"
        );
    }

    for required_frame_loop_anchor in [
        "pub(super) fn submit_camera_loop_frame(",
        "stream_camera_loop_frame_submissions(",
        "CameraLoopFrameSourceState::capture(&mut frame)",
        "for (submission_index, submission) in submissions.into_iter().enumerate()",
        "if submission_index > 0",
        "source_state.restore_for_submission(&mut frame);",
        "select_frame_camera_for_submission(&mut frame, submission.camera);",
        "terminal_ui.take()",
        "submit_selected_frame(\n            &mut frame,",
        "fn select_frame_camera_for_submission(",
        "fn restore_for_submission(&self, frame: &mut ViewportRenderFrame)",
    ] {
        assert!(
            camera_loop.contains(required_frame_loop_anchor),
            "Runtime 07 F3 direct runtime-frame streaming loop should retain anchor `{required_frame_loop_anchor}`"
        );
    }
    for required_runtime_stream_anchor in [
        "submit_camera_loop_frame(",
        "submit_selected_runtime_frame",
        "frame: &mut ViewportRenderFrame,",
        "source_payloads: Option<FrameSubmissionSourcePayloads<'_>>",
        "build_frame_submission_context_from_runtime_frame_extract(",
        "&mut frame.extract",
        "source_payloads,",
        "attach_prepared_sidebands_to_runtime_frame(frame, prepared);",
        "render_frame_with_pipeline(",
        "&*frame,",
    ] {
        assert!(
            submit_runtime_frame.contains(required_runtime_stream_anchor),
            "submit_runtime_frame should retain direct streaming anchor `{required_runtime_stream_anchor}`"
        );
    }
    for required_runtime_profile_anchor in [
        "profile_scope!(\"runtime\", \"render_framework\", \"build_submission_context\")",
        "profile_scope!(\"runtime\", \"render_framework\", \"prepare_runtime_submission\")",
        "profile_scope!(\"runtime\", \"render_framework\", \"render_frame_with_pipeline\")",
        "profile_scope!(\"runtime\", \"render_framework\", \"collect_runtime_feedback\")",
    ] {
        assert!(
            submit_runtime_frame.contains(required_runtime_profile_anchor),
            "submit_runtime_frame should retain Runtime 07 direct runtime-frame profiling anchor `{required_runtime_profile_anchor}`"
        );
    }
    assert!(
        frame_extract.contains(
            "pub fn select_camera_descriptor(&mut self, descriptor: CameraRenderDescriptor)"
        ),
        "RenderFrameExtract should expose in-place selected-camera projection for streaming submit"
    );
    assert!(
        !camera_loop.contains("project_frame_to_selected_camera("),
        "camera_loop should not restore the old borrowed full-frame projection helper"
    );
    assert!(
        !camera_loop.contains("camera_loop_frame_submissions(&frame)"),
        "production camera-loop frame submission should pass owned frames, not borrowed frames"
    );
    for forbidden_streaming_regression in [
        "camera_loop_frame_submissions(frame)",
        "CameraLoopFrameSubmission",
        "project_borrowed_frame_to_selected_camera(frame, submission.camera)",
        "frame.extract.as_ref().clone()",
    ] {
        assert!(
            !submit_runtime_frame.contains(forbidden_streaming_regression),
            "submit_runtime_frame should stream camera submissions instead of restoring `{forbidden_streaming_regression}`"
        );
    }
}
