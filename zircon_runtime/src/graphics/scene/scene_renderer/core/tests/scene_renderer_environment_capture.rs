const SOURCE: &str = include_str!("../scene_renderer_environment_capture.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("environment capture submission must retain a test boundary")
}

fn source_capture_owner() -> &'static str {
    let source = production_source();
    let start = source
        .find("pub(in crate::graphics) fn submit_environment_capture_source(")
        .expect("source capture owner");
    let end = source[start..]
        .find("pub(in crate::graphics) fn commit_environment_capture_probe(")
        .map(|offset| start + offset)
        .expect("source capture owner boundary");
    &source[start..end]
}

fn persistence_submission_owner() -> &'static str {
    let source = production_source();
    let start = source
        .find("fn submit_environment_capture_persistence_batch(")
        .expect("environment capture persistence submission owner");
    let end = source[start..]
        .find("/// Records and submits the source cubemap")
        .map(|offset| start + offset)
        .expect("environment capture persistence submission owner boundary");
    &source[start..end]
}

#[test]
fn source_capture_merges_uploads_and_submits_one_six_face_command_buffer() {
    let source = source_capture_owner();

    assert!(source.contains("fn submit_environment_capture_source("));
    assert!(source.contains("EnvironmentCaptureSceneUniformPlan::"));
    assert!(source.contains("build_environment_capture_mesh_draws("));
    assert!(source.contains("EnvironmentCaptureWgpuRecorder::record("));
    assert!(source.contains("EnvironmentCaptureFilterWgpuRecorder::record("));
    let record = source
        .find("EnvironmentCaptureWgpuRecorder::record(")
        .expect("capture recorder");
    let reject_incomplete = source[record..]
        .find(".map_err(GraphicsError::WgpuValidation)?")
        .map(|offset| record + offset)
        .expect("incomplete capture rejection");
    let filter = source
        .find("EnvironmentCaptureFilterWgpuRecorder::record(")
        .expect("capture filter");
    assert!(record < reject_incomplete);
    assert!(reject_incomplete < filter);
    let provider = source
        .find("ensure_environment_capture_provider(device)")
        .expect("capture must size the probe provider before publication");
    let reservation = source
        .find("reserve_environment_capture_target(cubemap, revision)")
        .expect("capture must reserve the target after provider sizing");
    assert!(provider < reservation);
    assert_eq!(
        source
            .matches("enqueue_copy_resource_upload_batch(")
            .count(),
        1
    );
    assert_eq!(
        source
            .matches("submit_graphics_command_buffers_with_diagnostics(")
            .count(),
        1
    );
    assert_eq!(source.matches("copy_environment_capture_probe(").count(), 1);
}

#[test]
fn source_capture_preserves_existing_submission_transactions() {
    let source = source_capture_owner();

    assert!(source.contains("begin_submission_usage_recording()"));
    assert!(source.contains("take_gpu_scene_prepared_upload()"));
    assert!(source.contains("gpu_scene_prepared_upload.append_to("));
    assert!(source.contains("bind_recorded_pipeline_usage_to_submission("));
    assert!(source.contains("commit_scene_environment_frame()"));
    assert!(!source.contains("reflection_probes\n            .commit_pending_uploads()"));
    assert!(source.contains("gpu_scene_prepared_upload.commit("));
    assert!(!source.contains("roll_prev_transforms_after_success"));
}

#[test]
fn source_capture_writes_one_light_list_and_uploads_six_face_grids() {
    let source = source_capture_owner();
    let light_plan = source
        .find("EnvironmentCaptureLightGridPlan::from_scene_batch(")
        .expect("capture light-grid plan");
    let light_write = source
        .find(".write_lights(device, light_grid_plan.lights())")
        .expect("capture GPU light write");
    let mesh_build = source
        .find("build_environment_capture_mesh_draws(")
        .expect("capture mesh build");

    assert!(light_plan < light_write);
    assert!(light_write < mesh_build);
    let generic_profile = source
        .find("disable_environment_only_pbr_base_profile()")
        .expect("lit capture must use a direct-light PBR variant");
    assert!(light_plan < generic_profile);
    assert!(generic_profile < mesh_build);
    assert!(source.contains("EnvironmentCaptureLightGridWorkspace::new("));
    assert!(source.contains("light_grid_plan.prepare_uploads("));
}

#[test]
fn source_capture_does_not_publish_materials_or_prepare_viewport_sidebands() {
    let source = source_capture_owner();

    assert!(source.contains("coordinate_material_pipeline_publications("));
    assert!(source.contains("false,\n            false,"));
    assert!(!source.contains("prepare_buffers("));
    assert!(!source.contains("build_shadow_frame_plan"));
    assert!(!source.contains("record_overlays("));
    assert!(!source.contains("screen_space_ui"));
}

#[test]
fn completion_check_observes_both_tickets_without_polling_or_waiting() {
    let source = production_source();
    let status_owner = source
        .split("fn environment_capture_submission_status(")
        .nth(1)
        .and_then(|source| source.split("fn submit_environment_capture_source(").next())
        .expect("environment capture submission status owner");

    assert_eq!(status_owner.matches("submission_status(").count(), 2);
    assert!(status_owner.contains("resource_upload_submission()"));
    assert!(status_owner.contains("capture_submission()"));
    assert!(!status_owner.contains("poll_submission_completions"));
    assert!(!status_owner.contains("wait_for_submission"));
}

#[test]
fn probe_array_copy_is_recorded_before_submission_but_commit_is_deferred() {
    let source = source_capture_owner();
    let copy = source
        .find("copy_environment_capture_probe(")
        .expect("capture must record its typed probe-array copy");
    let submit = source
        .find("submit_graphics_command_buffers_with_diagnostics(")
        .expect("capture must submit the completed encoder");
    assert!(copy < submit);
    assert!(source.contains("commit_environment_capture_probe("));
}

#[test]
fn explicit_probe_target_resolution_and_reservation_fail_closed() {
    let source = source_capture_owner();
    let resolution = source
        .find("let probe_target = match request.reflection_probe_target()")
        .expect("typed probe target revision admission");
    let target_allocation = source
        .find("let target = EnvironmentCaptureGpuTarget::new(device, &request)")
        .expect("capture target allocation");
    let provider_expansion = source
        .find("ensure_environment_capture_provider(device)")
        .expect("reflection provider expansion");
    let target = source
        .split("let probe_publication =")
        .nth(1)
        .and_then(|source| source.split("// Artifact readback").next())
        .expect("typed probe target owner");

    assert!(resolution < target_allocation);
    assert!(resolution < provider_expansion);
    assert!(source[resolution..target_allocation]
        .contains("streamer.resource_revision(cubemap).map_err("));
    assert!(target.contains("match probe_target"));
    assert!(target.contains("reserve_environment_capture_target(cubemap, revision)"));
    assert!(target.contains(".ok_or_else("));
    assert!(!target.contains(".and_then("));
    assert!(!target.contains(".ok()?"));
}

#[test]
fn renderer_owned_ibl_cache_readback_shares_capture_ticket_and_commits_after_submit() {
    let source = source_capture_owner();
    let request = source
        .find("request.runtime_cache_artifact_request()")
        .expect("capture must derive the renderer-owned runtime-cache request");
    let prepare = source
        .find("prepare_from_capture_target(")
        .expect("capture must prepare readback from the filtered target");
    let diagnostic = source
        .find("begin_product_diagnostic_readback_scope(")
        .expect("artifact readback must reserve a bounded diagnostic frame");
    let submit = source
        .find("submit_graphics_command_buffers_with_diagnostics(")
        .expect("artifact readback must share the capture submission ticket");
    let commit = source
        .find("commit_submitted(prepared)")
        .expect("artifact writeback must commit only after submission");
    assert!(request < diagnostic);
    assert!(diagnostic < prepare);
    assert!(prepare < submit);
    assert!(submit < commit);
    assert!(
        !source.contains("if let Some(artifact_request) = request.persistence_artifact_request()")
    );
}

#[test]
fn source_persistence_uses_bounded_diagnostic_batches_without_waiting() {
    let source = production_source();
    let begin = source
        .find("fn begin_environment_capture_persistence(")
        .expect("persistence begin owner");
    let end = source[begin..]
        .find("/// Records and submits the source cubemap")
        .map(|offset| begin + offset)
        .expect("persistence owner boundary");
    let persistence = &source[begin..end];

    assert!(persistence.contains("begin_source_cubemap_wgpu_readback("));
    assert!(persistence.contains("request_source_cubemap_wgpu_readback_batch("));
    assert!(persistence.contains("begin_product_diagnostic_readback_scope("));
    assert!(persistence.contains("scope.submit(&label)?"));
    assert!(persistence.contains("Err((source, error))"));
    assert!(!persistence_submission_owner().contains(".device"));
    assert!(!persistence_submission_owner().contains("create_command_encoder"));
    assert!(!persistence_submission_owner().contains("scope.prepare"));
    assert!(!persistence_submission_owner()
        .contains("submit_graphics_command_buffers_with_diagnostics("));
    assert!(!persistence.contains("device.poll("));
    assert!(!persistence.contains("wait_for_submission"));
}
