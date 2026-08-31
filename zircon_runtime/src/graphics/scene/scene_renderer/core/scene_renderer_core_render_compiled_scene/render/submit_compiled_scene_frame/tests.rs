#[test]
fn history_domains_commit_after_scene_submit_before_auxiliary_finalization() {
    let source = include_str!("../submit_compiled_scene_frame.rs");
    let submit = source
        .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("compiled frame must submit its command buffers");
    let history_commit = source
        .find("history.commit_history_frame(")
        .expect("history transaction must commit after submission acceptance");
    let report_commit = source
        .find("graph_execution_record.set_history_domains_report(")
        .expect("committed domain state must reach the execution record");
    let auxiliary_error = source
        .find("let environment_ibl_error")
        .expect("auxiliary finalization errors must remain explicit");

    assert!(submit < history_commit);
    assert!(history_commit < report_commit);
    assert!(report_commit < auxiliary_error);
}

#[test]
fn compiled_scene_propagates_ibl_post_submit_errors_after_transient_cleanup() {
    let source = include_str!("../submit_compiled_scene_frame.rs");
    let release = source
        .find("graph_resources.retire_transient_backings_after_submission")
        .unwrap_or_default();
    let environment_ibl_error = source.find("let environment_ibl_error").unwrap_or_default();
    let aggregate = source
        .find("GraphicsError::SceneSubmissionFinalization")
        .unwrap_or_default();
    let submitted_failure = source
        .find("GraphicsError::FrameFailedAfterSceneSubmission")
        .unwrap_or_default();

    assert!(release < environment_ibl_error);
    assert!(release < submitted_failure);
    assert!(submitted_failure < aggregate);
    assert!(source.contains("scene_submission: submission_ticket"));
    assert!(source.contains("readback: None"));
    assert!(source.contains("environment_ibl: environment_ibl_error"));
}

#[test]
fn ibl_runtime_writeback_joins_the_frame_submit_without_waiting_for_gpu_idle() {
    let render = include_str!("../render.rs");
    let terminal = include_str!("../terminal_frame_packet.rs");
    let submit_source = include_str!("../submit_compiled_scene_frame.rs");
    let writeback_source = include_str!("../../../../environment/ibl_bake_runtime_writeback.rs");
    let prepare = render
        .find("prepare_environment_ibl_runtime_cache_writeback")
        .expect("IBL artifact sections must join the product diagnostic frame");
    let terminal_entry = render
        .find("prepare_terminal_frame_packet(")
        .expect("product diagnostic copies must enter the scene terminal packet");
    let prepare_diagnostic_tail = terminal
        .find("scope.prepare(")
        .expect("product diagnostic copies must be encoded in the scene tail");
    let scene_boundary = render
        .find("self.submit_compiled_scene_frame(")
        .expect("compiled scene must retain one submission boundary");
    let submit = submit_source
        .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("compiled scene must submit diagnostics with scene work");
    let commit = submit_source
        .find(".commit_submitted(prepared)")
        .expect("IBL artifact ownership must commit after successful submission");

    assert!(prepare < terminal_entry);
    assert!(terminal_entry < scene_boundary);
    assert!(terminal[prepare_diagnostic_tail..].contains("product-diagnostic-readback"));
    assert!(submit < commit);
    assert!(!writeback_source.contains("device.poll("));
    assert!(!writeback_source.contains("map_async("));
    assert!(!writeback_source.contains("take_command_buffer("));
    assert!(!submit_source.contains("wait_indefinitely"));
    assert!(!submit_source.contains("queue.submit("));
}

#[test]
fn prepared_cubemap_upload_commits_only_after_the_compiled_frame_submit() {
    let source = include_str!("../submit_compiled_scene_frame.rs");
    let submit = source
        .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("compiled frame must submit its command buffers");
    let commit_cubemap_upload = source
        .find("self.scene_environment_cubemap.commit_pending_upload()")
        .expect("prepared cubemap upload must be committed after frame submission");
    let commit_ibl_writeback = source
        .find("self.ibl_bake_runtime_writebacks.commit_submitted(prepared)")
        .expect("IBL writeback submission state must remain explicit");

    assert!(submit < commit_cubemap_upload);
    assert!(commit_cubemap_upload < commit_ibl_writeback);
}

#[test]
fn prepared_probe_slots_commit_only_after_the_compiled_frame_submit() {
    let source = include_str!("../submit_compiled_scene_frame.rs");
    let submit = source
        .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("compiled frame must submit its command buffers");
    let commit_probe_uploads = source
        .find(".reflection_probes\n            .commit_pending_uploads()")
        .expect("prepared probe slots must commit after scene submission");

    assert!(submit < commit_probe_uploads);
}

#[test]
fn immediate_scene_finalizes_submitted_state_after_its_scene_ticket() {
    let source = include_str!("../../../scene_renderer_core_render_scene/render_scene.rs");
    let submit = source
        .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("direct scene must retain one ticketed submission");
    let commit_probe_uploads = source[submit..]
        .find(".reflection_probes\n            .commit_pending_uploads()")
        .map(|offset| submit + offset)
        .expect("probe slot state must finalize after scene submission");
    let complete_submission = source[submit..]
        .find("self.realtime_ibl.complete_submission")
        .map(|offset| submit + offset)
        .expect("submitted realtime IBL state must be finalized");
    let roll_transforms = source[submit..]
        .find("self.gpu_scene.roll_prev_transforms_after_success")
        .map(|offset| submit + offset)
        .expect("submitted transforms must roll after the scene ticket");
    let success = source[submit..]
        .find("Ok(scene_submission)")
        .map(|offset| submit + offset)
        .expect("direct scene must return its submission ticket");

    assert!(submit < commit_probe_uploads);
    assert!(commit_probe_uploads < complete_submission);
    assert!(complete_submission < roll_transforms);
    assert!(roll_transforms < success);
}
