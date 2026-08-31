#[test]
fn compiled_frame_retains_scene_and_present_submission_identity() {
    let source = [
        include_str!("../render_frame_with_pipeline.rs"),
        include_str!("frame_submission_owner.rs"),
        include_str!("surface_presentation.rs"),
    ]
    .concat();

    assert!(source.contains("runtime_outputs.scene_submission()"));
    assert!(source.contains("submission_transaction.finish(scene_submission)"));
    assert!(source.contains("finalize_surface_presentation(submission_receipt, present_result)"));
    assert!(source.contains("surface.acquire_frame_target()"));
    assert!(source.contains("surface.present_frame_target("));
    assert!(source.contains("surface.discard_frame_target(surface_target, source)"));
    assert!(!source.contains(concat!("surface.", "present_texture(")));
    assert!(
        source.contains("self.last_frame_submission_receipt = Some(submission_receipt.clone());")
    );
}

#[test]
fn compiled_frame_owner_polls_before_resource_and_history_preparation() {
    let source = include_str!("frame_submission_owner.rs");
    let poll = source
        .find("self.poll_frame_submission_completions()?")
        .expect("frame owner must pump completion");
    let transaction = source
        .find("RenderFrameSubmissionTransaction::begin(frame_generation, poll_receipt)")
        .expect("frame owner must begin the submission ledger");
    let ensure = source
        .find("self.streamer.ensure_scene_resources(")
        .expect("frame owner must prepare scene resources");
    let history = source
        .find("prepare_history_textures(")
        .expect("frame owner must prepare history");

    assert!(poll < transaction);
    assert!(transaction < ensure);
    assert!(ensure < history);
}

#[test]
fn compiled_frame_publishes_one_submission_metrics_interval() {
    let source = include_str!("frame_submission_owner.rs");
    let baseline = source
        .find("let submission_metrics_baseline = self.backend.submission_metrics();")
        .expect("compiled frame must sample after its completion poll");
    let ensure = source
        .find("self.streamer.ensure_scene_resources(")
        .expect("compiled frame resource preparation");
    let finish = source
        .find("submission_transaction.finish(scene_submission)")
        .expect("compiled frame receipt finalization");
    let attach = source
        .find(".with_submission_metrics(")
        .expect("compiled frame metrics publication");

    assert!(baseline < ensure);
    assert!(ensure < finish);
    assert!(finish < attach);
    assert_eq!(source.matches("frame_submission_metrics_since(").count(), 1);
}

#[test]
fn compiled_frame_failure_settles_recorded_pre_scene_submissions() {
    let source = include_str!("frame_submission_owner.rs");

    assert!(source.contains("&mut submission_transaction"));
    assert!(source.contains("if let Err(source) = self.streamer.ensure_scene_resources("));
    assert!(source.contains("settle_failed_frame_submissions("));
    assert!(source.contains("validate_scene_submission(scene_submission)"));
    assert!(source.contains("FrameFailedAfterSceneSubmission"));
    assert!(source.contains("GraphicsError::MissingViewFamilyPhase"));
    assert!(!source.contains("view-family scene phase must always be enabled"));
}

#[test]
fn compiled_viewport_product_is_prepared_before_recording_and_completed_with_scene_ticket() {
    let source = include_str!("frame_submission_owner.rs");
    let prepare = source
        .find(".prepare_texture_for_external_image(")
        .expect("product target must exist before compiled scene recording");
    let render = source
        .find("core.render_compiled_scene(")
        .expect("compiled scene recording");
    let complete = source
        .find("target.complete(scene_submission)")
        .expect("product target must be completed with the scene ticket");
    let attach = source
        .find(".with_viewport_product_submission(scene_submission)")
        .expect("frame receipt must retain the shared scene ticket");

    assert!(prepare < render);
    assert!(render < complete);
    assert!(complete < attach);
}

#[test]
fn hdr_capture_reads_the_retained_compiled_scene_color_without_a_second_scene_render() {
    let source = include_str!("../render_frame_with_pipeline.rs");
    let capture_method = concat!("fn capture_latest_scene_color", "_hdr");
    let capture_source = source
        .split(capture_method)
        .nth(1)
        .expect("compiled HDR capture should exist")
        .split("pub(crate) fn ui_surface_context")
        .next()
        .expect("HDR capture should end before the UI context accessor");

    assert!(capture_source.contains("target.scene_color"));
    assert!(capture_source.contains("read_product_diagnostic_texture_rgba16float_blocking"));
    assert!(!capture_source.contains("read_texture_rgba16float_region"));
    assert!(capture_source.contains("decode_rgba16f_texels"));
    assert!(!capture_source.contains("render_scene("));
}
