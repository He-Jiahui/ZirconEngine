const SOURCE: &str = include_str!("../render_scene.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("render scene source should retain a test-module boundary")
}

#[test]
fn direct_scene_binds_copy_and_query_diagnostics_to_its_scene_ticket() {
    let source = production_source();
    let submit = source
        .find("let scene_submission = match backend")
        .expect("direct scene submission");
    assert!(source[..submit].contains("product_diagnostic_frame"));
    assert!(source[..submit].contains("product_diagnostic_query_frame"));
    assert!(!source.contains("self.readback_queue"));
}

#[test]
fn direct_scene_core_does_not_own_the_frame_completion_pump() {
    let source = production_source();

    assert!(!source.contains("poll_submission_completions"));
    assert!(source.contains("let scene_submission ="));
    assert!(source.contains("Ok(scene_submission)"));
}

#[test]
fn environment_only_direct_render_skips_shadow_frame_work_before_resource_scanning() {
    let source = production_source();
    let profile_gate = source
        .find(".uses_full_shadow_atlas_resources()")
        .expect("direct render should gate shadow-frame work by its profile");
    let static_caster_scan = source
        .find("static_shadow_caster_revision_from_meshes_with_resource_revisions")
        .expect("full-scene path should retain static caster revision tracking");

    assert!(
        profile_gate < static_caster_scan,
        "EnvironmentOnly must skip resource revision scans before they can iterate mesh assets"
    );
    assert!(
        source.contains("shadow_frame_plan.as_ref()"),
        "only a full-scene shadow plan may supply light-slot assignments to mesh draws"
    );
}

#[test]
fn environment_only_direct_render_forwards_its_direct_light_policy_to_mesh_preparation() {
    let source = production_source();
    let profile_policy = source
        .find("let uses_direct_lights = self.deferred_lighting_profile.uses_direct_lights();")
        .expect("direct rendering must derive the light-buffer policy from its profile");
    let mesh_preparation = source
        .find("self.advanced_plugin_resources.build_mesh_draws(")
        .expect("direct rendering must prepare mesh draws");
    let preparation_call = &source[mesh_preparation..];

    assert!(
        profile_policy < mesh_preparation,
        "the profile policy must be decided before mesh preparation begins"
    );
    assert!(
        preparation_call.contains("uses_direct_lights,"),
        "the direct renderer must forward the profile policy to mesh preparation"
    );
}

#[test]
fn direct_render_records_its_real_encoder_stages_with_the_shared_gpu_timer() {
    let source = production_source();

    assert!(source.contains("gpu_pass_timer: Option<&mut GpuPassTimer>"));
    assert!(source.contains(".begin_product_diagnostic_query_scope("));
    assert!(source.contains("scope.attach_timers(gpu_pass_timer.as_deref_mut(), None)"));
    assert!(source.contains("timer.begin_pass(&mut encoder, DIRECT_REALTIME_IBL_GPU_PASS)"));
    assert!(source.contains("timer.begin_pass(&mut encoder, DIRECT_GPU_SCENE_UPLOAD_GPU_PASS)"));
    assert!(source.contains("timer.begin_pass(&mut encoder, DIRECT_SCENE_CONTENT_GPU_PASS)"));
    assert!(source.contains("timer.begin_pass(&mut encoder, DIRECT_OUTPUT_TRANSFER_GPU_PASS)"));
    assert!(source.contains("timer.begin_pass(&mut encoder, DIRECT_OVERLAYS_GPU_PASS)"));
    assert!(source.contains("if let Some(screen_space_ui_renderer)"));
    assert!(source.contains(".finish_and_prepare(&mut encoder"));
    assert!(!source.contains("resolve_and_request"));

    let begin_frame = source
        .find(".begin_product_diagnostic_query_scope(")
        .expect("direct rendering must reserve a native query frame");
    let scene_content = source
        .find("timer.begin_pass(&mut encoder, DIRECT_SCENE_CONTENT_GPU_PASS)")
        .expect("direct scene content must have a named timestamp scope");
    let realtime_ibl = source
        .find("timer.begin_pass(&mut encoder, DIRECT_REALTIME_IBL_GPU_PASS)")
        .expect("direct realtime IBL must have a named timestamp scope");
    let gpu_scene_upload = source
        .find("timer.begin_pass(&mut encoder, DIRECT_GPU_SCENE_UPLOAD_GPU_PASS)")
        .expect("direct GPU scene upload must have a named timestamp scope");
    let resolve = source
        .find(".finish_and_prepare(&mut encoder")
        .expect("direct timing must resolve through the typed query owner");

    assert!(begin_frame < realtime_ibl);
    assert!(realtime_ibl < gpu_scene_upload);
    assert!(gpu_scene_upload < scene_content);
    assert!(begin_frame < scene_content);
    assert!(scene_content < resolve);
}

#[test]
fn environment_only_timestamp_evidence_has_required_core_scopes_and_optional_realtime_work() {
    let source = production_source();

    for scope in [
        "DIRECT_GPU_SCENE_UPLOAD_GPU_PASS",
        "DIRECT_SCENE_CONTENT_GPU_PASS",
        "DIRECT_OUTPUT_TRANSFER_GPU_PASS",
        "DIRECT_OVERLAYS_GPU_PASS",
    ] {
        assert!(source.contains(scope), "HDRI evidence must retain {scope}");
    }
    assert!(source.contains("realtime_ibl_prepared.is_some()"));
    assert!(source.contains("if let Some(screen_space_ui_renderer)"));
}

#[test]
fn direct_render_defers_gpu_timing_when_query_admission_is_unavailable() {
    let source = production_source();
    assert!(source.contains("timer.defer_frame(frame_generation)"));
    assert!(source.contains("product_diagnostic_query_scope.as_ref()"));
    assert!(source.contains(
        "let realtime_ibl_gpu_timing_enabled = self.realtime_ibl.gpu_timestamps_supported();"
    ));
    assert!(!source.contains("readback_ready"));
    assert!(!source.contains("self.readback_queue"));
}

#[test]
fn static_environment_uploads_share_the_direct_frame_encoder() {
    let source = production_source();
    let encoder = source
        .find("let mut encoder = device.create_command_encoder")
        .expect("direct render owns a frame encoder");
    let write_uniform = source
        .find("self.write_scene_uniform(")
        .expect("direct render must update scene bindings");
    let submit = source
        .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("direct render submits its one frame encoder");
    let commit = source
        .find("self.commit_scene_environment_frame();")
        .expect("submitted environment uploads must advance their content key");

    assert!(encoder < write_uniform);
    assert!(write_uniform < submit);
    assert!(submit < commit);
}

#[test]
fn direct_frame_resource_upload_is_merged_and_recorded_before_scene_submission() {
    let source = production_source();
    let constants = source
        .find("let mut frame_buffer_uploads = self.write_scene_uniform(")
        .expect("direct rendering must prepare the packed scene constants");
    let shadow = source
        .find("prepared_upload.append_to(&mut frame_buffer_uploads)")
        .expect("direct rendering must append shadow data to the frame batch");
    let gpu_scene = source
        .find("gpu_scene_prepared_upload.append_to(&self.gpu_scene, &mut frame_buffer_uploads)")
        .expect("direct rendering must merge GPU Scene writes into the frame batch");
    let icon_prepare = source
        .find("self.overlay_renderer.prepare_buffers(")
        .expect("direct rendering must prepare viewport icon uploads in the frame batch");
    let ui = source
        .find("if !prepared_upload.append_to(")
        .expect("direct rendering must merge UI writes into the frame batch");
    let enqueue = source
        .find("backend.enqueue_copy_resource_upload_batch(")
        .expect("direct rendering must accept one frame resource upload batch");
    let ledger = source
        .find("RenderFrameSubmissionProducer::FrameResourceUpload")
        .expect("direct rendering must retain the merged frame upload ticket");
    let commit = source
        .find("gpu_scene_prepared_upload.commit(&mut self.gpu_scene)")
        .expect("GPU Scene dirty state must commit after backend acceptance");
    let ui_commit = source
        .find("renderer.commit_prepared_upload(prepared_upload)")
        .expect("UI reuse state must commit after backend acceptance");
    let icon_commit = source
        .find("self.overlay_renderer.commit_pending_icon_uploads()")
        .expect("viewport icon reuse state must commit after backend acceptance");
    let submit = source
        .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("direct rendering must submit its scene packet");
    let validate = source
        .find("submission_transaction.validate_scene_submission(scene_submission)")
        .expect("direct rendering must validate the scene ticket before resource commit");
    let pipeline_usage = source[submit..]
        .find(".bind_recorded_pipeline_usage_to_submission(scene_submission)")
        .map(|offset| submit + offset)
        .expect("submitted pipeline usage must bind before fallible ledger validation");
    let cubemap_commit = source[submit..]
        .find("self.commit_scene_environment_frame()")
        .map(|offset| submit + offset)
        .expect("submitted cubemap upload must settle before fallible ledger validation");
    let realtime_ibl_commit = source[submit..]
        .find("self.realtime_ibl.complete_submission(submission, true)")
        .map(|offset| submit + offset)
        .expect("submitted realtime IBL work must settle before fallible ledger validation");

    assert!(constants < shadow);
    assert!(shadow < gpu_scene);
    assert!(gpu_scene < icon_prepare);
    assert!(icon_prepare < ui);
    assert!(ui < enqueue);
    assert!(enqueue < ledger);
    assert!(ledger < submit);
    assert!(submit < validate);
    assert!(submit < pipeline_usage);
    assert!(pipeline_usage < validate);
    assert!(cubemap_commit < validate);
    assert!(realtime_ibl_commit < validate);
    assert!(validate < commit);
    assert!(validate < ui_commit);
    assert!(validate < icon_commit);
}

#[test]
fn direct_output_target_writeback_precedes_the_diagnostic_tail_and_scene_submit() {
    let source = production_source();
    let writeback = source
        .find("streamer.encode_output_target_writeback_with_frame_plan(")
        .expect("direct rendering must encode the resolved output plan in its frame packet");
    let submit = source
        .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("direct rendering must retain one frame submission boundary");
    let query_tail = source
        .find(".finish_and_prepare(&mut encoder")
        .expect("typed queries must resolve in the serial diagnostic tail");
    let viewport_product_copy = source
        .find("viewport_product_copy.encode_copy(&mut encoder, final_color)")
        .expect("viewport product copy must share the scene encoder");

    assert!(writeback < viewport_product_copy);
    assert!(viewport_product_copy < query_tail);
    assert!(writeback < query_tail);
    assert!(query_tail < submit);
    assert!(!source.contains("queue.submit("));
}

#[test]
fn direct_realtime_ibl_readback_shares_the_scene_diagnostic_ticket() {
    let source = production_source();
    let begin = source
        .find("backend.begin_product_diagnostic_readback_frame(frame_generation)")
        .expect("direct realtime IBL diagnostics must open the product frame");
    let request = source
        .find("self.realtime_ibl.request_product_gpu_timestamp_readback(")
        .expect("direct realtime IBL timestamps must use the product router");
    let writeback = source
        .find("streamer.encode_output_target_writeback_with_frame_plan(")
        .expect("resolved output writeback must precede the diagnostic tail");
    let prepare = source
        .find("backend.prepare_product_diagnostic_readback_frame(")
        .expect("direct rendering must encode the product diagnostic tail");
    let submit = source
        .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("direct rendering must bind diagnostics to its scene ticket");
    let query_prepare = source
        .find(".finish_and_prepare(&mut encoder")
        .expect("typed queries must use the native diagnostic query tail");

    assert!(begin < request);
    assert!(request < writeback);
    assert!(writeback < prepare);
    assert!(prepare < submit);
    assert!(prepare < query_prepare);
    assert!(query_prepare < submit);
    assert!(source[submit..].contains("product_diagnostic_frame,"));
    assert!(source[submit..].contains("product_diagnostic_query_frame,"));
    assert!(!source.contains("request_gpu_timestamp_readback("));
}
