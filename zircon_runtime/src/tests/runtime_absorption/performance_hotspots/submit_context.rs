#[test]
fn runtime_07_submit_context_shares_large_extract_payloads() {
    let context = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs"
    );
    let build_context = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs"
    );
    let camera_loop = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs"
    );
    let frame_extract = include_str!("../../../core/framework/render/frame_extract.rs");
    let collect_feedback = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs"
    );
    let build_runtime_frame = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs"
    );
    let submit_extract = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs"
    );
    let present_frame_extract = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs"
    );
    let submit_runtime_frame = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs"
    );
    let record_submission = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs"
    );
    let record_present_submission = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/record_submission/record_present.rs"
    );
    let record_camera_history = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/submit/record_camera_history.rs"
    );
    let prepared_submission = include_str!(
        "../../../graphics/runtime/render_framework/submit_frame_extract/prepared_runtime_submission.rs"
    );
    let prepared_runtime_sidebands =
        include_str!("../../../core/framework/render/prepared_runtime_sidebands.rs");
    let viewport_render_frame = include_str!("../../../graphics/types/viewport_render_frame.rs");
    let viewport_render_frame_from_extract =
        include_str!("../../../graphics/types/viewport_render_frame_from_extract.rs");
    let runtime_07_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");

    assert!(
        context.contains("source_extract: Arc<RenderFrameExtract>"),
        "FrameSubmissionContext should own one shared viewport-sized RenderFrameExtract payload"
    );
    assert!(
        viewport_render_frame.contains("pub extract: Arc<RenderFrameExtract>"),
        "ViewportRenderFrame should store RenderFrameExtract as an Arc for shared submit sources"
    );

    for forbidden_owned_payload in [
        "scene_meshes: Vec<RenderMeshSnapshot>",
        "scene_directional_lights: Vec<RenderDirectionalLightSnapshot>",
        "scene_point_lights: Vec<RenderPointLightSnapshot>",
        "scene_spot_lights: Vec<RenderSpotLightSnapshot>",
        "scene_ambient_lights: Vec<RenderAmbientLightSnapshot>",
        "scene_rect_lights: Vec<RenderRectLightSnapshot>",
        "particle_previous_sprites: Vec<RenderParticlePreviousSpriteSnapshot>",
    ] {
        assert!(
            !context.contains(forbidden_owned_payload),
            "FrameSubmissionContext should not reclaim cloned payload field `{forbidden_owned_payload}`"
        );
    }

    for forbidden_build_clone in [
        "extract.geometry.meshes.clone(),",
        "extract.lighting.directional_lights.clone(),",
        "extract.lighting.point_lights.clone(),",
        "extract.lighting.spot_lights.clone(),",
        "extract.lighting.ambient_lights.clone(),",
        "extract.lighting.rect_lights.clone(),",
        "extract.particles.previous_sprites.clone(),",
    ] {
        assert!(
            !build_context.contains(forbidden_build_clone),
            "build_frame_submission_context should not clone large extract payload `{forbidden_build_clone}` into FrameSubmissionContext::new"
        );
    }

    for forbidden_effective_clone_helper in [
        "let mut sized_extract = extract.clone();",
        "fn post_process_extract_with_effective_settings",
        "fn visibility_extract_with_effective_advanced_features",
        "Arc::try_unwrap(extract).unwrap_or_else(|extract| (*extract).clone())",
    ] {
        assert!(
            !build_context.contains(forbidden_effective_clone_helper),
            "build_frame_submission_context should not restore full-extract clone helper `{forbidden_effective_clone_helper}`"
        );
    }
    assert!(
        !build_runtime_frame
            .contains("Arc::try_unwrap(extract).unwrap_or_else(|extract| (*extract).clone())"),
        "build_runtime_frame should not clone the shared extract to append virtual-geometry debug overlays"
    );

    for required_anchor in [
        "build_frame_submission_context_from_runtime_frame_extract(",
        "fn build_frame_submission_context_from_source(",
        "extract_source: &mut Arc<RenderFrameExtract>",
        "Arc::make_mut(extract_source)",
        "let source_extract = Arc::clone(extract_source);",
        "runtime_virtual_geometry_debug_overlays(",
        "source_overlays: &RenderOverlayExtract",
        "with_runtime_overlays(runtime_overlays)",
        "runtime_overlay_override: Option<RenderOverlayExtract>",
        ".unwrap_or(&self.extract.debug.overlays)",
        "pub(super) fn source_extract(&self) -> Arc<RenderFrameExtract>",
        "&self.source_extract.geometry.meshes",
        "&self.source_extract.lighting.directional_lights",
        "FrameSubmissionSourcePayloads",
        "source_payloads: Option<FrameSubmissionSourcePayloads<'_>>",
        "VisibilityContext::from_extract_with_history_static_index_task_pool_and_feature_payloads",
        "FrameHistoryValidationKey::from_extract_with_hybrid_gi",
        "apply_effective_post_process_settings(",
    ] {
        assert!(
            context.contains(required_anchor)
                || build_context.contains(required_anchor)
                || build_runtime_frame.contains(required_anchor)
                || viewport_render_frame.contains(required_anchor),
            "Runtime 07 F3 source-extract sharing should retain anchor `{required_anchor}`"
        );
    }
    for required_shared_frame_anchor in [
        "pub(crate) fn from_shared_extract(",
        "ViewportRenderFrame::from_shared_extract(extract, context.size())",
        "let extract = context.source_extract();",
        "build_frame_submission_context_from_runtime_frame_extract(",
        "extract: &mut Arc<RenderFrameExtract>",
        "build_runtime_frame(ui, &context, prepared, output_policy)",
        "frame.extract = context.source_extract();",
    ] {
        assert!(
            viewport_render_frame_from_extract.contains(required_shared_frame_anchor)
                || build_runtime_frame.contains(required_shared_frame_anchor)
                || submit_extract.contains(required_shared_frame_anchor)
                || present_frame_extract.contains(required_shared_frame_anchor)
                || submit_runtime_frame.contains(required_shared_frame_anchor),
            "Runtime 07 F3 shared effective frame source should retain anchor `{required_shared_frame_anchor}`"
        );
    }

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

    for required_feedback_anchor in [
        "sidebands: &mut RenderPreparedRuntimeSidebands",
        "sidebands.take_hybrid_gi_readback_outputs()",
        "sidebands.take_particle_readback_outputs()",
        "sidebands.take_virtual_geometry_readback_outputs()",
        "sideband_outputs: RenderHybridGiReadbackOutputs",
        "sideband_outputs: RenderParticleGpuReadbackOutputs",
        "sideband_outputs: RenderVirtualGeometryReadbackOutputs",
        "renderer_outputs.cache_entries.extend(cache_entries);",
        "renderer_outputs.page_replacements.extend(page_replacements);",
    ] {
        assert!(
            collect_feedback.contains(required_feedback_anchor),
            "Runtime 07 F3 feedback sideband merge should retain owned-merge anchor `{required_feedback_anchor}`"
        );
    }
    for required_prepared_sideband_anchor in [
        "fn take_hybrid_gi_readback_outputs(",
        "fn take_particle_readback_outputs(",
        "fn take_virtual_geometry_readback_outputs(",
    ] {
        assert!(
            prepared_runtime_sidebands.contains(required_prepared_sideband_anchor),
            "RenderPreparedRuntimeSidebands should retain sideband take anchor `{required_prepared_sideband_anchor}`"
        );
    }
    for forbidden_feedback_clone in [
        "return sideband_outputs.clone();",
        "sideband_outputs: &RenderHybridGiReadbackOutputs",
        "sideband_outputs: &RenderParticleGpuReadbackOutputs",
        "sideband_outputs: &RenderVirtualGeometryReadbackOutputs",
        "sideband_outputs.cache_entries.iter().cloned()",
        "sideband_outputs.completed_page_assignments.iter().cloned()",
        "sideband_outputs.scene_prepare.clone()",
    ] {
        assert!(
            !collect_feedback.contains(forbidden_feedback_clone),
            "Runtime 07 F3 feedback sideband merge should not restore borrowed clone `{forbidden_feedback_clone}`"
        );
    }
    for required_prepared_sideband_move_anchor in [
        "fn into_prepared_runtime_sidebands(self) -> RenderPreparedRuntimeSidebands",
        "prepared.into_prepared_runtime_sidebands()",
        "sidebands: &mut RenderPreparedRuntimeSidebands",
        "sidebands.take_hybrid_gi_readback_outputs()",
        "sidebands.take_particle_readback_outputs()",
        "sidebands.take_virtual_geometry_readback_outputs()",
        "with_evictable_probe_ids(sidebands.take_hybrid_gi_evictable_probe_ids())",
        "with_evictable_page_ids(sidebands.take_virtual_geometry_evictable_page_ids())",
        "pub(crate) fn prepared_runtime_sidebands_mut(&mut self) -> &mut RenderPreparedRuntimeSidebands",
    ] {
        assert!(
            prepared_submission.contains(required_prepared_sideband_move_anchor)
                || prepared_runtime_sidebands.contains(required_prepared_sideband_move_anchor)
                || collect_feedback.contains(required_prepared_sideband_move_anchor)
                || build_runtime_frame.contains(required_prepared_sideband_move_anchor)
                || submit_runtime_frame.contains(required_prepared_sideband_move_anchor)
                || viewport_render_frame.contains(required_prepared_sideband_move_anchor),
            "Runtime 07 F3 prepared sideband frame-owner move should retain anchor `{required_prepared_sideband_move_anchor}`"
        );
    }
    for forbidden_prepared_sideband_clone in [
        "plugin_renderer_outputs.clone()",
        "hybrid_gi_evictable_probe_ids.clone()",
        "virtual_geometry_evictable_page_ids.clone()",
        "prepared.prepared_runtime_sidebands()",
        "mut prepared: PreparedRuntimeSubmission",
        "prepared.take_hybrid_gi_evictable_probe_ids()",
        "prepared.take_virtual_geometry_evictable_page_ids()",
        "with_prepared_runtime_sidebands(frame.prepared_runtime_sidebands.clone())",
        "frame.prepared_runtime_sidebands.clone()",
    ] {
        assert!(
            !prepared_submission.contains(forbidden_prepared_sideband_clone)
                && !camera_loop.contains(forbidden_prepared_sideband_clone)
                && !build_runtime_frame.contains(forbidden_prepared_sideband_clone)
                && !submit_runtime_frame.contains(forbidden_prepared_sideband_clone)
                && !record_submission.contains(forbidden_prepared_sideband_clone)
                && !record_present_submission.contains(forbidden_prepared_sideband_clone)
                && !record_camera_history.contains(forbidden_prepared_sideband_clone),
            "Runtime 07 F3 prepared sideband frame-owner move should not restore `{forbidden_prepared_sideband_clone}`"
        );
    }

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
