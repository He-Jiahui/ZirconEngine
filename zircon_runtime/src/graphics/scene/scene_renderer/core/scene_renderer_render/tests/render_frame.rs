#[test]
fn direct_frame_retains_the_device_qualified_submission_receipt() {
    let source = include_str!("../render_frame.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("direct frame owner must retain a test boundary");

    assert!(source.contains("let (scene_submission, viewport_product_copy) = match"));
    assert!(source.contains("core.render_scene("));
    assert!(source.contains("validate_scene_submission(scene_submission)"));
    assert!(source.contains("submission_transaction.finish(scene_submission)"));
    assert!(production.contains(".track(frame_generation, scene_submission)"));
    assert!(
        source.contains("self.last_frame_submission_receipt = Some(submission_receipt.clone());")
    );
    assert!(source.contains("Ok((submission_receipt, viewport_product_copy))"));
}

#[test]
fn direct_frame_owner_polls_before_resource_preparation_and_core_recording() {
    let source = include_str!("../render_frame.rs");
    let poll = source
        .find("self.poll_frame_submission_completions()?")
        .expect("frame owner must pump completion");
    let transaction = source
        .find("RenderFrameSubmissionTransaction::begin(frame_generation, poll_receipt)")
        .expect("frame owner must begin the submission ledger");
    let admission = source
        .find("self.admit_render_scene_frame(frame.extract.as_ref(), frame_generation)")
        .expect("frame owner must admit the persistent render-scene journal");
    let membership = source
        .find("self.stage_pending_gpu_scene_membership()")
        .expect("frame owner must apply journal-owned GPUScene membership");
    let ensure = source
        .find("self.streamer.ensure_scene_resources(")
        .expect("frame owner must prepare scene resources");
    let output_plan = source
        .find("let output_target_frame_plan = self.streamer.output_target_frame_plan();")
        .expect("frame owner must capture the resolved output plan");
    let render = source
        .find("core.render_scene(")
        .expect("frame owner must invoke core recording");

    assert!(poll < transaction);
    assert!(transaction < admission);
    assert!(admission < membership);
    assert!(membership < ensure);
    assert!(ensure < output_plan);
    assert!(output_plan < render);
    assert!(ensure < render);
}

#[test]
fn direct_frame_publishes_one_submission_metrics_interval() {
    let source = include_str!("../render_frame.rs");
    let baseline = source
        .find("let submission_metrics_baseline = self.backend.submission_metrics();")
        .expect("direct frame must sample after its completion poll");
    let ensure = source
        .find("self.streamer.ensure_scene_resources(")
        .expect("direct frame resource preparation");
    let finish = source
        .find("submission_transaction.finish(scene_submission)")
        .expect("direct frame receipt finalization");
    let attach = source
        .find(".with_submission_metrics(")
        .expect("direct frame metrics publication");

    assert!(baseline < ensure);
    assert!(ensure < finish);
    assert!(finish < attach);
    assert_eq!(source.matches("frame_submission_metrics_since(").count(), 1);
}

#[test]
fn direct_frame_failure_settles_recorded_texture_submissions() {
    let source = include_str!("../render_frame.rs");

    assert!(source.contains("&mut submission_transaction"));
    assert!(source.contains("let scene_submission_result = (||"));
    assert!(source.contains("settle_failed_frame_submissions("));
}

#[test]
fn direct_viewport_product_is_prepared_before_recording_and_completed_with_scene_ticket() {
    let source = include_str!("../render_frame.rs");
    let prepare = source
        .find(".prepare_texture_for_external_image(")
        .expect("product target must exist before scene recording");
    let render = source
        .find("core.render_scene(")
        .expect("direct scene recording");
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
fn frame_timing_clocks_are_only_read_after_an_explicit_request() {
    let source = include_str!("../render_frame.rs");

    assert!(source.contains("let capture_frame_timing = self.frame_timing_report_requested;"));
    assert!(
        source.contains("let render_submission_started = capture_frame_timing.then(Instant::now);")
    );
    assert!(source.contains(
        "let readback_and_completion_started = capture_frame_timing.then(Instant::now);"
    ));
    assert!(source.contains(
        "let render_submission = render_submission_started.map(|started| started.elapsed());"
    ));
    assert!(
        source.contains("if let (Some(render_submission), Some(readback_and_completion_started))")
    );
    assert!(source.contains("self.frame_timing_report_requested = false;"));

    let submission_end = source
        .find("let render_submission = render_submission_started.map")
        .expect("submission interval must end before readback begins");
    let readback_start = source
        .find("let readback_and_completion_started = capture_frame_timing.then(Instant::now);")
        .expect("readback interval must have an explicit start");
    assert!(
        submission_end < readback_start,
        "submission and readback timing intervals must not overlap"
    );
}

#[test]
fn scene_completion_owner_routes_every_poll_before_draining_timer_results() {
    let source = include_str!("../../scene_renderer_completion.rs");
    let direct_source = include_str!("../render_frame.rs");
    let direct_caller = direct_source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("direct frame caller must retain a test boundary");
    let compiled_caller = include_str!(
        "../../scene_renderer_render_with_pipeline/render_frame_with_pipeline/frame_submission_owner.rs"
    );
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("scene completion owner must retain a test boundary");
    let poll = production
        .find("self.backend.poll_submission_completions()")
        .expect("scene renderer must poll its sole submission owner");
    let typed_query_route = production
        .find("backend.drain_product_diagnostic_query_results()")
        .expect("scene renderer must route completed typed query frames");
    let ibl_artifact_delivery = production
        .find("core.ibl_bake_runtime_writebacks")
        .expect("IBL artifact callbacks must drain after a completion poll");
    let timer_drain = production
        .find("GpuPassTimer::try_collect")
        .expect("scene renderer must drain timestamp results after polling");
    let statistics_drain = production
        .find("GpuPipelineStatisticsTimer::try_collect")
        .expect("scene renderer must drain statistics results after polling");

    assert_eq!(
        production.matches("poll_submission_completions()").count(),
        1
    );
    assert!(poll < ibl_artifact_delivery);
    assert!(ibl_artifact_delivery < typed_query_route);
    assert!(poll < typed_query_route);
    assert!(typed_query_route < timer_drain);
    assert!(typed_query_route < statistics_drain);
    assert!(poll < timer_drain);
    assert!(poll < statistics_drain);
    for caller in [direct_caller, compiled_caller] {
        assert!(caller.contains("self.poll_frame_submission_completions()?"));
        assert!(!caller.contains("self.backend.poll_submission_completions()?"));
        assert!(!caller.contains("readback_queue.poll_completed()"));
    }

    let readback_owner = include_str!(
        "../../scene_renderer_render_with_pipeline/render_frame_with_pipeline/readback.rs"
    );
    let backend_submission =
        include_str!("../../../../../backend/render_backend/render_backend_submission.rs");
    let backend_diagnostics =
        include_str!("../../../../../backend/render_backend/render_backend_diagnostics.rs");
    assert!(readback_owner.contains("self.poll_frame_submission_completions()?"));
    assert!(!readback_owner.contains("self.backend.poll_submission_completions()?"));
    for backend_loop in [backend_submission, backend_diagnostics] {
        let poll = backend_loop
            .find("let poll_receipt = self.poll_submission_completions()?")
            .expect("explicit completion loop must poll through the backend owner");
        let route = backend_loop
            .find("observe_poll(poll_receipt)?")
            .expect("every explicit poll must be routed to SceneRenderer consumers");
        assert!(poll < route);
    }
}
