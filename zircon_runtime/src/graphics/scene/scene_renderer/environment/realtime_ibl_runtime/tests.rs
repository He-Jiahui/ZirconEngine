use super::compiled_graph_cache::RealtimeIblCompiledGraphCache;
use super::*;
use crate::core::framework::render::{
    IblBakeArtifactRequest, SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};
use crate::core::runtime::diagnostics::profiling;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::environment::ibl_bake_wgpu_pipeline_cache::IblBakeWgpuPipelineCache;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_profile_test_support::start_manual_cpu_profile_capture;
use std::time::Instant;

#[test]
fn realtime_runtime_defers_gpu_resource_creation_until_a_procedural_frame() {
    let runtime = RealtimeIblRuntime::new();

    assert!(!runtime.is_gpu_initialized());
    assert!(!runtime.gpu_timestamps_supported());
    assert_eq!(
        runtime.compiled_graph_cache_stats(),
        RealtimeIblCompiledGraphCacheStats::default()
    );
}

#[test]
fn realtime_ibl_graph_profile_exports_to_an_explicit_non_c_root() {
    let source = include_str!("tests.rs");
    let profile_function = ["fn profile_realtime_ibl_graph_resource_preparation", "()"].concat();
    let profile = source
        .split(profile_function.as_str())
        .nth(1)
        .and_then(|body| body.split("#[test]").next())
        .expect("realtime IBL graph-preparation profile source");

    let support = include_str!("../realtime_ibl_profile_test_support.rs");
    assert!(profile.contains("profile_capture.finish_and_export();"));
    assert!(profile.contains("ProfileFrameScope::enter"));
    assert!(profile.contains("ProfileScope::enter"));
    assert!(support.contains("ZIRCON_PROFILE_OUTPUT_ROOT"));
    assert!(support.contains("absolute non-C"));
}

#[test]
fn first_procedural_frame_initializes_realtime_gpu_resources_and_starts_a_ticket() {
    let backend = RenderBackend::new_offscreen()
        .expect("offscreen backend required for the first procedural realtime IBL frame");
    let mut runtime = RealtimeIblRuntime::new();

    let prepared = runtime.prepare_frame(&backend.device, ProceduralSkyParams::default_gradient());

    assert!(runtime.is_gpu_initialized());
    assert!(prepared.batch.is_some());
    assert!(!prepared.uses_realtime_resources());
}

#[test]
fn initial_ticket_keeps_sampling_on_procedural_fallback() {
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(8, 2).expect("valid test config"),
    );
    scheduler.request_rebake(ProceduralSkyParams::default_gradient().ibl_bake_key());
    let _batch = scheduler.begin_frame(1).expect("initial batch");

    assert!(!scheduler.has_published_environment());
}

#[test]
fn ticket_update_keeps_sampling_the_published_ready_slot() {
    let sky = ProceduralSkyParams::default_gradient();
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(8, 2).expect("valid test config"),
    );
    scheduler.request_rebake(sky.ibl_bake_key());
    let mut frame_number = 1;
    complete_pending_realtime_ibl_rebake(&mut scheduler, &mut frame_number);
    let mut changed = sky;
    changed.horizon_color.x += 0.1;
    scheduler.request_rebake(changed.ibl_bake_key());
    let sliced = scheduler.begin_frame(frame_number).expect("ticket batch");

    assert!(scheduler.has_published_environment());
    assert_eq!(sliced.ready_slot(), scheduler.ready_slot());
}

#[test]
fn realtime_bake_key_ignores_final_sampling_intensity_and_rotation() {
    let first = ProceduralSkyParams::default_gradient();
    let mut second = first;
    second.rotation_radians += 0.25;
    let mut third = first;
    third.intensity += 0.5;

    assert_eq!(runtime_bake_key(&first), runtime_bake_key(&second));
    assert_eq!(runtime_bake_key(&first), runtime_bake_key(&third));
}

#[test]
fn realtime_bake_key_includes_capture_shader_content_identity() {
    let sky = ProceduralSkyParams::default_gradient();
    let parameter_key = sky.ibl_bake_key();
    let runtime_key = runtime_bake_key(&sky);

    assert_eq!(parameter_key.source_hash, [0; 4]);
    assert_ne!(runtime_key, parameter_key);
    assert_ne!(runtime_key.source_hash, [0; 4]);
}

#[test]
fn realtime_bake_key_tracks_effective_directional_sun_changes() {
    let mut first = ProceduralSkyParams::default_gradient();
    first.sun_intensity = 3.0;
    let mut second = first;
    second.sun_color.x = 0.5;

    assert_ne!(runtime_bake_key(&first), runtime_bake_key(&second));
}

#[test]
fn operation_label_preserves_realtime_ibl_stage_order() {
    assert_eq!(
        operation_label(&[
            RealtimeIblOperation::CaptureSky(
                super::super::realtime_ibl_time_slice::CubeFaceRange::ALL,
            ),
            RealtimeIblOperation::GenerateSourceMip { mip_level: 1 },
            RealtimeIblOperation::ProjectDiffuseSh9,
        ]),
        "capture_sky+source_mip+diffuse_sh9"
    );
}

#[test]
fn realtime_prepare_reuses_one_derived_bake_key_for_request_and_scheduler() {
    let source = include_str!("../realtime_ibl_runtime.rs");
    let start = source
        .find("pub(in crate::graphics) fn prepare_frame")
        .expect("realtime prepare_frame");
    let end = source[start..]
        .find("pub(in crate::graphics) fn record_prepared_frame")
        .map(|offset| start + offset)
        .expect("realtime prepare_frame boundary");
    let prepare = &source[start..end];

    assert!(prepare.contains("let (bake_key, bake_sky) = self.resolve_bake_snapshot(sky);"));
    assert!(prepare.contains("request_for_key(bake_key)"));
    assert!(prepare.contains("sky: bake_sky"));
    assert!(!prepare.contains("request_for_sky(&sky)"));
}

#[test]
fn changing_sky_inputs_finish_the_active_generation_and_coalesce_the_latest_snapshot() {
    let mut runtime = RealtimeIblRuntime::new();
    let mut initial = ProceduralSkyParams::default_gradient();
    initial.source_revision = 100;
    let initial_key = runtime_bake_key(&initial);
    let (active_key, active_sky) = runtime.resolve_bake_snapshot(initial);

    assert_eq!(active_key, initial_key);
    assert_eq!(active_sky, initial);
    assert_eq!(runtime.active_generation_start_frame_number, Some(1));
    assert_eq!(runtime.active_generation_coalesced_source_change_count, 0);

    let mut latest = initial;
    let mut published_frame_number = None;
    for frame_number in 1..=64 {
        latest.source_revision = 100 + frame_number;
        let (active_key, active_sky) = runtime.resolve_bake_snapshot(latest);
        assert_eq!(active_key, initial_key);
        assert_eq!(active_sky, initial);
        assert_eq!(
            runtime.active_generation_coalesced_source_change_count,
            frame_number
        );
        assert!(runtime.queued_sky.is_some());
        runtime.frame_number = frame_number;

        let batch = runtime
            .scheduler
            .begin_frame(frame_number)
            .expect("the active realtime IBL generation must keep advancing");
        let completion =
            runtime.complete_scheduler_frame(batch.token(), RealtimeIblSliceAttempt::Succeeded);
        if completion == RealtimeIblCompletion::Published {
            published_frame_number = Some(frame_number);
            break;
        }
    }

    assert_eq!(
        published_frame_number,
        Some(21),
        "continuous sky edits must preserve the default generation publication bound"
    );
    assert_eq!(runtime.scheduler.published_key(), Some(initial_key));
    assert_eq!(
        runtime.scheduler.pending_key(),
        Some(runtime_bake_key(&latest)),
        "only the latest edit should become the next generation"
    );
    assert_eq!(runtime.active_sky, Some(latest));
    assert_eq!(runtime.queued_sky, None);
    assert_eq!(runtime.active_generation_start_frame_number, Some(22));
    assert_eq!(runtime.active_generation_coalesced_source_change_count, 0);
}

#[test]
fn repeating_the_same_queued_sky_does_not_inflate_the_coalesced_change_count() {
    let mut runtime = RealtimeIblRuntime::new();
    let mut active = ProceduralSkyParams::default_gradient();
    active.source_revision = 200;
    runtime.resolve_bake_snapshot(active);

    let mut queued = active;
    queued.source_revision = 201;
    runtime.resolve_bake_snapshot(queued);
    runtime.resolve_bake_snapshot(queued);

    assert_eq!(runtime.queued_sky, Some(queued));
    assert_eq!(runtime.active_generation_coalesced_source_change_count, 1);

    runtime.resolve_bake_snapshot(active);
    assert_eq!(runtime.queued_sky, None);
    assert_eq!(runtime.active_generation_coalesced_source_change_count, 1);
}

#[test]
fn realtime_status_tracks_fallback_baking_ready_and_last_good_refresh() {
    let mut runtime = RealtimeIblRuntime::new();
    let initial = runtime.status_report();
    assert_eq!(initial.readiness, RealtimeIblReadiness::Fallback);
    assert_eq!(initial.current_frame_number, 0);
    assert_eq!(initial.published_key, None);
    assert_eq!(initial.pending_key, None);
    assert_eq!(initial.queued_key, None);
    assert_eq!(initial.published_generation_frame_number, None);
    assert_eq!(initial.last_good_age_frame_count, None);
    assert_eq!(initial.active_generation_elapsed_frame_count, None);
    assert_eq!(initial.failure, None);

    let mut first_sky = ProceduralSkyParams::default_gradient();
    first_sky.source_revision = 250;
    let first_key = runtime_bake_key(&first_sky);
    runtime.resolve_bake_snapshot(first_sky);
    let baking = runtime.status_report();
    assert_eq!(baking.readiness, RealtimeIblReadiness::Baking);
    assert_eq!(baking.pending_key, Some(first_key));
    assert_eq!(baking.active_generation_start_frame_number, Some(1));
    assert_eq!(baking.active_generation_elapsed_frame_count, Some(0));

    for frame_number in 1..=64 {
        runtime.frame_number = frame_number;
        let batch = runtime
            .scheduler
            .begin_frame(frame_number)
            .expect("successful status generation batch");
        if runtime.complete_scheduler_frame(batch.token(), RealtimeIblSliceAttempt::Succeeded)
            == RealtimeIblCompletion::Published
        {
            break;
        }
    }
    let ready = runtime.status_report();
    assert_eq!(ready.readiness, RealtimeIblReadiness::Ready);
    assert_eq!(ready.published_key, Some(first_key));
    assert_eq!(ready.pending_key, None);
    assert_eq!(ready.published_generation_frame_number, Some(21));
    assert_eq!(ready.last_good_age_frame_count, Some(0));
    assert_eq!(ready.active_generation_start_frame_number, None);
    assert_eq!(ready.active_generation_elapsed_frame_count, None);

    let mut next_sky = first_sky;
    next_sky.source_revision = 251;
    let next_key = runtime_bake_key(&next_sky);
    runtime.resolve_bake_snapshot(next_sky);
    let refreshing = runtime.status_report();
    assert_eq!(
        refreshing.readiness,
        RealtimeIblReadiness::RefreshingLastGood
    );
    assert_eq!(refreshing.published_key, Some(first_key));
    assert_eq!(refreshing.pending_key, Some(next_key));
    assert_eq!(refreshing.published_generation_frame_number, Some(21));
    assert_eq!(refreshing.last_good_age_frame_count, Some(0));
    assert_eq!(refreshing.active_generation_elapsed_frame_count, Some(0));

    for (frame_number, failure_kind) in [
        (22, RealtimeIblFailureKind::Recording),
        (24, RealtimeIblFailureKind::Submission),
        (27, RealtimeIblFailureKind::Submission),
    ] {
        runtime.frame_number = frame_number;
        let batch = runtime
            .scheduler
            .begin_frame(frame_number)
            .expect("last-good refresh failure attempt");
        runtime
            .complete_scheduler_frame(batch.token(), RealtimeIblSliceAttempt::Failed(failure_kind));
    }
    let failed_last_good = runtime.status_report();
    assert_eq!(
        failed_last_good.readiness,
        RealtimeIblReadiness::FailedLastGood
    );
    assert_eq!(failed_last_good.published_key, Some(first_key));
    assert_eq!(failed_last_good.pending_key, None);
    assert_eq!(failed_last_good.published_generation_frame_number, Some(21));
    assert_eq!(failed_last_good.last_good_age_frame_count, Some(6));
    assert_eq!(failed_last_good.active_generation_elapsed_frame_count, None);
    assert!(
        failed_last_good
            .failure
            .expect("last-good terminal report")
            .terminal
    );

    runtime.resolve_bake_snapshot(first_sky);
    let restored_published = runtime.status_report();
    assert_eq!(restored_published.readiness, RealtimeIblReadiness::Ready);
    assert_eq!(restored_published.published_key, Some(first_key));
    assert_eq!(restored_published.pending_key, None);
    assert_eq!(
        restored_published.published_generation_frame_number,
        Some(21)
    );
    assert_eq!(restored_published.last_good_age_frame_count, Some(6));
    assert!(restored_published.failure.is_none());
}

#[test]
fn realtime_status_freshness_is_rollover_safe_and_replaced_only_by_publication() {
    let mut runtime = RealtimeIblRuntime::new();
    runtime.frame_number = u64::MAX - 21;
    let mut first_sky = ProceduralSkyParams::default_gradient();
    first_sky.source_revision = 275;
    runtime.resolve_bake_snapshot(first_sky);

    let before_first_slice = runtime.status_report();
    assert_eq!(
        before_first_slice.active_generation_start_frame_number,
        Some(u64::MAX - 20)
    );
    assert_eq!(
        before_first_slice.active_generation_elapsed_frame_count,
        Some(0)
    );
    let published_frame = publish_pending_realtime_ibl_generation(&mut runtime, u64::MAX - 20);
    assert_eq!(published_frame, u64::MAX);
    assert_eq!(
        runtime.status_report().published_generation_frame_number,
        Some(u64::MAX)
    );
    assert_eq!(runtime.status_report().last_good_age_frame_count, Some(0));

    runtime.frame_number = 1;
    let aged = runtime.status_report();
    assert_eq!(aged.published_generation_frame_number, Some(u64::MAX));
    assert_eq!(aged.last_good_age_frame_count, Some(2));

    let mut next_sky = first_sky;
    next_sky.source_revision = 276;
    runtime.resolve_bake_snapshot(next_sky);
    let refreshing = runtime.status_report();
    assert_eq!(refreshing.published_generation_frame_number, Some(u64::MAX));
    assert_eq!(refreshing.last_good_age_frame_count, Some(2));
    assert_eq!(refreshing.active_generation_elapsed_frame_count, Some(0));

    let next_published_frame = publish_pending_realtime_ibl_generation(&mut runtime, 2);
    assert_eq!(next_published_frame, 22);
    let replaced = runtime.status_report();
    assert_eq!(replaced.published_generation_frame_number, Some(22));
    assert_eq!(replaced.last_good_age_frame_count, Some(0));
}

#[test]
fn terminal_failure_clears_runtime_generation_and_suppresses_the_same_source_revision() {
    let mut runtime = RealtimeIblRuntime::new();
    let mut failed_sky = ProceduralSkyParams::default_gradient();
    failed_sky.source_revision = 300;
    let failed_key = runtime_bake_key(&failed_sky);
    runtime.resolve_bake_snapshot(failed_sky);

    let mut queued_sky = failed_sky;
    queued_sky.source_revision = 301;
    runtime.resolve_bake_snapshot(queued_sky);
    assert_eq!(runtime.active_sky, Some(failed_sky));
    assert_eq!(runtime.queued_sky, Some(queued_sky));

    let first = runtime.scheduler.begin_frame(1).expect("first attempt");
    assert_eq!(
        runtime.complete_scheduler_frame(
            first.token(),
            RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Recording),
        ),
        RealtimeIblCompletion::RetryScheduled
    );
    assert_eq!(runtime.scheduler.begin_frame(2), None);

    let second = runtime.scheduler.begin_frame(3).expect("second attempt");
    assert_eq!(
        runtime.complete_scheduler_frame(
            second.token(),
            RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Submission),
        ),
        RealtimeIblCompletion::RetryScheduled
    );
    assert_eq!(runtime.scheduler.begin_frame(4), None);
    assert_eq!(runtime.scheduler.begin_frame(5), None);

    let terminal = runtime.scheduler.begin_frame(6).expect("terminal attempt");
    assert_eq!(
        runtime.complete_scheduler_frame(
            terminal.token(),
            RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Submission),
        ),
        RealtimeIblCompletion::Failed
    );
    assert_eq!(runtime.active_sky, None);
    assert_eq!(runtime.queued_sky, None);
    assert_eq!(runtime.scheduler.pending_key(), None);
    let status = runtime.status_report();
    assert_eq!(status.readiness, RealtimeIblReadiness::FailedFallback);
    let report = status.failure.expect("terminal failure report");
    assert_eq!(report.bake_key, failed_key);
    assert!(report.terminal);

    runtime.resolve_bake_snapshot(failed_sky);
    assert_eq!(runtime.active_sky, None);
    assert_eq!(runtime.scheduler.pending_key(), None);
    assert_eq!(
        runtime.status_report().readiness,
        RealtimeIblReadiness::FailedFallback
    );

    runtime.resolve_bake_snapshot(queued_sky);
    assert_eq!(runtime.active_sky, Some(queued_sky));
    assert_eq!(
        runtime.scheduler.pending_key(),
        Some(runtime_bake_key(&queued_sky))
    );
    let recovered = runtime.status_report();
    assert_eq!(recovered.readiness, RealtimeIblReadiness::Baking);
    assert!(recovered.failure.is_none());
}

#[test]
fn runtime_attributes_recording_and_submission_failures_to_distinct_reports() {
    let source = include_str!("../realtime_ibl_runtime.rs");

    assert!(source.contains("RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Recording)"));
    assert!(source.contains("RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Submission)"));
    assert!(!source.contains("self.scheduler.complete_frame("));
}

#[test]
fn realtime_operation_label_does_not_allocate_an_intermediate_vector() {
    let source = include_str!("../realtime_ibl_runtime.rs");
    let start = source
        .find("fn operation_label")
        .expect("operation_label implementation");
    let operation_label = &source[start..];

    assert!(!operation_label.contains("collect::<Vec<_>>()"));
}

#[test]
fn realtime_timestamp_metadata_is_deferred_until_timestamp_recording_exists() {
    let source = include_str!("../realtime_ibl_runtime.rs");
    let normalized = source.split_whitespace().collect::<String>();

    assert!(source.contains("timestamp_metadata: Option<RealtimeIblGpuTimingMetadata>"));
    assert!(normalized.contains("timestamp_readback.as_ref().map(|_|"));
}

#[test]
fn realtime_ibl_gpu_timing_is_explicitly_forwarded_to_timestamp_recording() {
    let source = include_str!("../realtime_ibl_runtime.rs");
    let start = source
        .find("pub(in crate::graphics) fn record_prepared_frame")
        .expect("realtime IBL recording entrypoint");
    let end = source[start..]
        .find("pub(in crate::graphics) fn complete_submission")
        .map(|offset| start + offset)
        .expect("realtime IBL recording boundary");
    let recording = &source[start..end];

    assert!(recording.contains("gpu_timing_enabled: bool"));
    assert!(recording.contains("gpu_timing_enabled,"));
}

#[test]
fn realtime_ibl_cpu_measurements_require_an_active_profile_capture() {
    let source = include_str!("../realtime_ibl_runtime.rs");
    let start = source
        .find("pub(in crate::graphics) fn record_prepared_frame")
        .expect("realtime IBL recording entrypoint");
    let end = source[start..]
        .find("pub(in crate::graphics) fn complete_submission")
        .map(|offset| start + offset)
        .expect("realtime IBL recording boundary");
    let recording = &source[start..end];

    assert!(recording.contains("profiling::capture_epoch()"));
    assert!(recording.contains("cpu_timing_enabled,"));
}

#[test]
fn realtime_ibl_cpu_timing_reports_require_an_accepted_submission_in_the_same_capture_epoch() {
    let source = include_str!("../realtime_ibl_runtime.rs");
    let completion = source
        .split("pub(in crate::graphics) fn complete_submission")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(in crate::graphics) fn request_product_gpu_timestamp_readback")
                .next()
        })
        .expect("realtime IBL completion implementation");

    assert!(completion.contains("gpu_succeeded"));
    assert!(completion.contains("profiling::capture_epoch_for_completion()"));
    assert!(completion.contains("self.cpu_timing_collector.record_completed(report)"));

    let drain = source
        .split("fn take_cpu_timing_reports")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(in crate::graphics) fn source_view")
                .next()
        })
        .expect("realtime IBL CPU timing drain implementation");
    assert!(drain.contains("synchronize_capture_epoch(profiling::capture_epoch_for_completion())"));
}

#[test]
fn realtime_ibl_pending_submission_reports_bound_graph_resources() {
    let _profile_capture_lock = profiling::test_capture_lock();
    profiling::reset_capture();
    let backend = RenderBackend::new_offscreen()
        .expect("offscreen backend required for realtime IBL graph preparation");
    let mut runtime = RealtimeIblRuntime::new();
    let prepared = runtime.prepare_frame(&backend.device, ProceduralSkyParams::default_gradient());
    let mut pipeline_cache = IblBakeWgpuPipelineCache::new(&backend.device);
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-realtime-ibl-graph-preparation-regression"),
        });

    let submission = runtime
        .record_prepared_frame(
            &backend.device,
            &mut encoder,
            false,
            &prepared,
            &mut pipeline_cache,
        )
        .expect("realtime IBL graph preparation should record")
        .expect("initial procedural frame should contain a realtime IBL batch");

    assert!(submission.graph_preparation.texture_view_binding_count > 0);
    assert_eq!(submission.graph_preparation.buffer_binding_count, 0);
    assert_eq!(
        submission.graph_preparation.execution_resource_cache_hits,
        0
    );
    assert_eq!(
        submission.graph_preparation.execution_resource_cache_misses,
        1
    );
    assert_eq!(
        submission
            .graph_preparation
            .execution_resource_cache_entry_count,
        1
    );
    assert!(
        submission
            .graph_preparation
            .execution_resource_cache_topology_capacity
            >= 1
    );
    assert_eq!(
        submission.graph_preparation.total_bound_resource_count,
        submission.graph_preparation.texture_view_binding_count
            + submission.graph_preparation.buffer_binding_count
    );
    assert_eq!(
        submission
            .graph_preparation
            .execution_resource_binding_micros,
        0,
        "normal realtime IBL recording must not start CPU measurement clocks"
    );
    assert_eq!(
        submission.graph_preparation.validation_micros, 0,
        "normal realtime IBL recording must not start CPU measurement clocks"
    );
    assert_eq!(submission.report.command_plan_creation_micros, 0);
    assert_eq!(submission.report.pipeline_ensure_micros, 0);
    assert_eq!(submission.report.binding_creation_micros, 0);
    assert_eq!(submission.report.capture_binding_creation_micros, 0);
    assert_eq!(submission.report.source_mip_binding_creation_micros, 0);
    drop(encoder.finish());
    runtime.complete_submission(submission, true);
    assert!(runtime.take_cpu_timing_reports().is_empty());
}

#[test]
#[ignore = "manual public CPU timing drain profile requires the profiling Cargo feature"]
fn profile_realtime_ibl_cpu_timing_reports_do_not_require_timestamp_recording() {
    let RenderBackend { device, queue, .. } = RenderBackend::new_offscreen()
        .expect("realtime IBL CPU timing profile requires an offscreen WGPU backend");
    let profile_capture = start_manual_cpu_profile_capture("realtime-ibl-cpu-timing-drain");
    let profile_frame =
        profiling::ProfileFrameScope::enter("realtime_ibl_cpu_profile", "cpu_timing_drain");
    let mut runtime = RealtimeIblRuntime::new();
    let prepared = runtime.prepare_frame(&device, ProceduralSkyParams::default_gradient());
    let mut pipeline_cache = IblBakeWgpuPipelineCache::new(&device);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-realtime-ibl-cpu-timing-drain-profile"),
    });
    let submission = {
        let _profile_scope = profiling::ProfileScope::enter(
            "realtime_ibl_cpu_profile",
            "recording",
            "submitted_ticket",
        );
        runtime
            .record_prepared_frame(&device, &mut encoder, false, &prepared, &mut pipeline_cache)
            .expect("realtime IBL CPU timing profile recording")
            .expect("initial procedural frame should contain a realtime IBL batch")
    };

    drop(profile_frame);
    profiling::stop_capture();
    queue.submit([encoder.finish()]);
    runtime.complete_submission(submission, true);
    let reports = runtime.take_cpu_timing_reports();

    assert_eq!(reports.len(), 1);
    assert!(reports[0].profile_capture_epoch > 0);
    assert_eq!(reports[0].pass_count, 1);
    assert_eq!(reports[0].dispatch_count, 1);
    assert!(reports[0].total_bound_resource_count > 0);
    profile_capture.finish_and_export();
}

#[test]
#[ignore = "manual CPU profile for realtime IBL graph resource preparation"]
fn profile_realtime_ibl_graph_resource_preparation() {
    const PROFILE_TICKET_COUNT: u64 = 256;
    const PASSES_PER_TICKET: u64 = 21;

    let RenderBackend {
        adapter, device, ..
    } = RenderBackend::new_offscreen()
        .expect("realtime IBL graph preparation profile requires an offscreen WGPU backend");
    let profile_capture = start_manual_cpu_profile_capture("realtime-ibl-graph-preparation");
    let profile_frame = profiling::ProfileFrameScope::enter(
        "realtime_ibl_cpu_profile",
        "graph_resource_preparation",
    );
    let adapter_info = adapter.get_info();
    let mut runtime = RealtimeIblRuntime::new();
    let mut pipeline_cache = IblBakeWgpuPipelineCache::new(&device);
    let mut active_batch_count = 0;
    let mut total = RealtimeIblGraphPreparationReport::default();
    let started = Instant::now();

    for source_revision in 1..=PROFILE_TICKET_COUNT {
        let mut sky = ProceduralSkyParams::default_gradient();
        sky.source_revision = source_revision;
        loop {
            let _profile_scope = profiling::ProfileScope::enter(
                "realtime_ibl_cpu_profile",
                "recording",
                "prepare_and_record_batch",
            );
            let prepared = runtime.prepare_frame(&device, sky);
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-realtime-ibl-graph-preparation-profile"),
            });
            let submission = runtime
                .record_prepared_frame(&device, &mut encoder, false, &prepared, &mut pipeline_cache)
                .expect("realtime IBL graph preparation profile recording")
                .expect("requested realtime IBL ticket must produce a batch");

            active_batch_count += 1;
            total.accumulate(submission.graph_preparation);
            drop(encoder.finish());
            runtime.complete_recording_without_submission(submission);

            if !runtime.scheduler.is_rebake_pending() {
                break;
            }
        }
    }

    let elapsed = started.elapsed();
    eprintln!(
        "realtime_ibl_graph_preparation_profile adapter_name={} adapter_backend={} adapter_vendor_id={} adapter_device_id={} adapter_type={:?} tickets={PROFILE_TICKET_COUNT} active_batches={active_batch_count} execution_resource_binding_micros={} validation_micros={} execution_resource_cache_hits={} execution_resource_cache_misses={} execution_resource_cache_entry_peak={} execution_resource_cache_topology_capacity={} texture_view_bindings={} buffer_bindings={} total_bound_resources={} total_elapsed_ms={:.3}",
        adapter_info.name,
        adapter_info.backend.to_str(),
        adapter_info.vendor,
        adapter_info.device,
        adapter_info.device_type,
        total.execution_resource_binding_micros,
        total.validation_micros,
        total.execution_resource_cache_hits,
        total.execution_resource_cache_misses,
        total.execution_resource_cache_entry_count,
        total.execution_resource_cache_topology_capacity,
        total.texture_view_binding_count,
        total.buffer_binding_count,
        total.total_bound_resource_count,
        elapsed.as_secs_f64() * 1000.0,
    );
    assert_eq!(active_batch_count, PROFILE_TICKET_COUNT * PASSES_PER_TICKET);
    assert_eq!(
        total.execution_resource_cache_hits + total.execution_resource_cache_misses,
        active_batch_count,
        "each recorded batch must identify one execution-resource cache outcome"
    );
    assert!(
        total.execution_resource_cache_misses > 0,
        "a fresh resource layout must materialize at least one execution-resource topology"
    );
    assert!(
        total.execution_resource_cache_hits > 0,
        "repeated realtime IBL topologies must reuse execution resources"
    );
    assert!(total.execution_resource_cache_topology_capacity > 0);
    assert!(
        total.execution_resource_cache_entry_count
            <= total.execution_resource_cache_topology_capacity,
        "execution-resource cache entries must stay within the scheduler topology capacity"
    );
    assert!(total.texture_view_binding_count > 0);
    assert!(
        total.texture_view_binding_count < active_batch_count as usize,
        "execution-resource cache hits must not count as new texture bindings"
    );
    assert!(total.buffer_binding_count > 0);
    assert!(
        total.buffer_binding_count < PROFILE_TICKET_COUNT as usize,
        "execution-resource cache hits must not count as new buffer bindings"
    );
    assert_eq!(
        total.total_bound_resource_count,
        total.texture_view_binding_count + total.buffer_binding_count
    );
    drop(profile_frame);
    profile_capture.finish_and_export();
}

#[test]
fn realtime_ibl_compiled_graph_cache_reuses_an_unchanged_topology() {
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(
        sky.ibl_bake_key(),
        SOURCE_CUBEMAP_PMREM_FACE_SIZE,
        SOURCE_CUBEMAP_PMREM_MIP_COUNT,
    );
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(request.pmrem_mip_count() as u8, 2)
            .expect("valid realtime IBL scheduler config"),
    );
    scheduler.request_rebake(sky.ibl_bake_key());
    let batch = scheduler
        .begin_frame(1)
        .expect("initial realtime IBL batch");
    let mut cache = RealtimeIblCompiledGraphCache::new();

    cache
        .resolve(&request, &batch)
        .expect("initial topology compiles");
    cache
        .resolve(&request, &batch)
        .expect("unchanged topology reuses the compiled artifact");
    let mut changed_sky = sky;
    changed_sky.sun_intensity += 1.0;
    let changed_request = IblBakeArtifactRequest::new(
        changed_sky.ibl_bake_key(),
        SOURCE_CUBEMAP_PMREM_FACE_SIZE,
        SOURCE_CUBEMAP_PMREM_MIP_COUNT,
    );
    cache
        .resolve(&changed_request, &batch)
        .expect("a sky-only bake key change reuses the compiled topology");

    assert_eq!(
        cache.stats(),
        super::RealtimeIblCompiledGraphCacheStats {
            cache_hit_count: 2,
            cache_miss_count: 1,
            compile_count: 1,
            eviction_count: 0,
            variant_count: 1,
        }
    );
}

#[test]
fn realtime_ibl_compiled_graph_cache_bounds_two_ticket_slot_topologies() {
    let first_sky = ProceduralSkyParams::default_gradient();
    let mut second_sky = first_sky;
    second_sky.source_revision = 2;
    let mut third_sky = second_sky;
    third_sky.source_revision = 3;
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(8, 2).expect("valid realtime IBL scheduler config"),
    );
    let mut cache = RealtimeIblCompiledGraphCache::new();
    let mut frame_number = 1;

    scheduler.request_rebake(first_sky.ibl_bake_key());
    resolve_pending_realtime_ibl_ticket(
        &mut cache,
        &mut scheduler,
        &IblBakeArtifactRequest::new(
            first_sky.ibl_bake_key(),
            SOURCE_CUBEMAP_PMREM_FACE_SIZE,
            SOURCE_CUBEMAP_PMREM_MIP_COUNT,
        ),
        &mut frame_number,
    );
    scheduler.request_rebake(second_sky.ibl_bake_key());
    resolve_pending_realtime_ibl_ticket(
        &mut cache,
        &mut scheduler,
        &IblBakeArtifactRequest::new(
            second_sky.ibl_bake_key(),
            SOURCE_CUBEMAP_PMREM_FACE_SIZE,
            SOURCE_CUBEMAP_PMREM_MIP_COUNT,
        ),
        &mut frame_number,
    );

    assert_eq!(cache.stats().variant_count, 42);
    assert_eq!(cache.stats().compile_count, 42);
    assert_eq!(cache.stats().eviction_count, 0);

    scheduler.request_rebake(third_sky.ibl_bake_key());
    let repeated = scheduler
        .begin_frame(frame_number)
        .expect("third ticket must restart at the first topology");
    cache
        .resolve(
            &IblBakeArtifactRequest::new(
                third_sky.ibl_bake_key(),
                SOURCE_CUBEMAP_PMREM_FACE_SIZE,
                SOURCE_CUBEMAP_PMREM_MIP_COUNT,
            ),
            &repeated,
        )
        .expect("completed ticket topology must remain cached");

    assert_eq!(cache.stats().variant_count, 42);
    assert_eq!(cache.stats().compile_count, 42);
    assert_eq!(cache.stats().cache_hit_count, 1);
    assert_eq!(cache.stats().eviction_count, 0);
}

#[test]
fn realtime_ibl_cached_recording_order_matches_the_compiled_graph() {
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(
        sky.ibl_bake_key(),
        SOURCE_CUBEMAP_PMREM_FACE_SIZE,
        SOURCE_CUBEMAP_PMREM_MIP_COUNT,
    );
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(request.pmrem_mip_count() as u8, 2)
            .expect("valid realtime IBL scheduler config"),
    );
    scheduler.request_rebake(sky.ibl_bake_key());
    let batch = scheduler
        .begin_frame(1)
        .expect("initial realtime IBL batch");
    let mut cache = RealtimeIblCompiledGraphCache::new();

    let artifact = cache
        .resolve(&request, &batch)
        .expect("initial topology compiles");

    assert_eq!(
        artifact
            .recording_passes()
            .iter()
            .map(|pass| pass.pass_id)
            .collect::<Vec<_>>(),
        artifact
            .graph()
            .passes()
            .iter()
            .map(|pass| pass.id)
            .collect::<Vec<_>>(),
    );
}

#[test]
fn realtime_ibl_compiled_graph_cache_distinguishes_operation_topologies() {
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 16, 5);
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(request.pmrem_mip_count() as u8, 2)
            .expect("valid realtime IBL scheduler config"),
    );
    scheduler.request_rebake(sky.ibl_bake_key());
    let initial = scheduler.begin_frame(1).expect("initial batch");
    scheduler.complete_frame(initial.token(), true);

    let mut changed_sky = sky;
    changed_sky.sun_intensity += 1.0;
    scheduler.request_rebake(changed_sky.ibl_bake_key());
    let first = scheduler.begin_frame(2).expect("first sliced batch");
    scheduler.complete_frame(first.token(), true);
    let second = scheduler.begin_frame(3).expect("second sliced batch");
    let changed_request = IblBakeArtifactRequest::new(changed_sky.ibl_bake_key(), 16, 5);
    let mut cache = RealtimeIblCompiledGraphCache::new();

    cache
        .resolve(&changed_request, &first)
        .expect("first topology compiles");
    cache
        .resolve(&changed_request, &second)
        .expect("second topology compiles");

    assert_ne!(first.operations(), second.operations());
    assert_eq!(first.ready_slot(), second.ready_slot());
    assert_eq!(first.work_slot(), second.work_slot());
    assert_eq!(cache.stats().variant_count, 2);
    assert_eq!(cache.stats().compile_count, 2);
}

#[test]
fn realtime_ibl_compiled_graph_cache_distinguishes_buffer_slot_topologies() {
    let sky = ProceduralSkyParams::default_gradient();
    let request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 16, 5);
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(request.pmrem_mip_count() as u8, 2)
            .expect("valid realtime IBL scheduler config"),
    );
    let mut frame_number = 1;
    scheduler.request_rebake(sky.ibl_bake_key());
    complete_pending_realtime_ibl_rebake(&mut scheduler, &mut frame_number);

    let mut first_sky = sky;
    first_sky.sun_intensity += 1.0;
    scheduler.request_rebake(first_sky.ibl_bake_key());
    let first = scheduler
        .begin_frame(frame_number)
        .expect("first capture topology");
    complete_pending_realtime_ibl_rebake(&mut scheduler, &mut frame_number);

    let mut second_sky = first_sky;
    second_sky.sun_intensity += 1.0;
    scheduler.request_rebake(second_sky.ibl_bake_key());
    let second = scheduler
        .begin_frame(frame_number)
        .expect("second capture topology");
    let first_request = IblBakeArtifactRequest::new(first_sky.ibl_bake_key(), 16, 5);
    let second_request = IblBakeArtifactRequest::new(second_sky.ibl_bake_key(), 16, 5);
    let mut cache = RealtimeIblCompiledGraphCache::new();

    cache
        .resolve(&first_request, &first)
        .expect("first buffer-slot topology compiles");
    cache
        .resolve(&second_request, &second)
        .expect("second buffer-slot topology compiles");

    assert_eq!(first.operations(), second.operations());
    assert_ne!(first.ready_slot(), second.ready_slot());
    assert_ne!(first.work_slot(), second.work_slot());
    assert_eq!(cache.stats().variant_count, 2);
    assert_eq!(cache.stats().compile_count, 2);
}

#[test]
fn realtime_ibl_compiled_graph_cache_invalidates_stale_request_geometry() {
    let sky = ProceduralSkyParams::default_gradient();
    let first_request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 16, 5);
    let second_request = IblBakeArtifactRequest::new(sky.ibl_bake_key(), 32, 5);
    let mut first_scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(first_request.pmrem_mip_count() as u8, 2)
            .expect("valid first realtime IBL scheduler config"),
    );
    let mut second_scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(second_request.pmrem_mip_count() as u8, 2)
            .expect("valid second realtime IBL scheduler config"),
    );
    first_scheduler.request_rebake(sky.ibl_bake_key());
    second_scheduler.request_rebake(sky.ibl_bake_key());
    let first = first_scheduler
        .begin_frame(1)
        .expect("first geometry batch");
    let second = second_scheduler
        .begin_frame(1)
        .expect("second geometry batch");
    let mut cache = RealtimeIblCompiledGraphCache::new();

    cache
        .resolve(&first_request, &first)
        .expect("first geometry compiles");
    cache
        .resolve(&second_request, &second)
        .expect("second geometry compiles");

    assert_eq!(first.operations(), second.operations());
    assert_eq!(first.ready_slot(), second.ready_slot());
    assert_eq!(first.work_slot(), second.work_slot());
    assert_eq!(cache.stats().variant_count, 1);
    assert_eq!(cache.stats().compile_count, 2);
}

fn complete_pending_realtime_ibl_rebake(
    scheduler: &mut RealtimeIblTimeSliceScheduler,
    frame_number: &mut u64,
) {
    while scheduler.is_rebake_pending() {
        let batch = scheduler
            .begin_frame(*frame_number)
            .expect("pending realtime IBL rebake must yield a batch");
        *frame_number += 1;
        scheduler.complete_frame(batch.token(), true);
    }
}

fn publish_pending_realtime_ibl_generation(
    runtime: &mut RealtimeIblRuntime,
    first_frame_number: u64,
) -> u64 {
    let mut frame_number = first_frame_number;
    for _ in 0..64 {
        runtime.frame_number = frame_number;
        let batch = runtime
            .scheduler
            .begin_frame(frame_number)
            .expect("pending realtime IBL generation must yield a batch");
        if runtime.complete_scheduler_frame(batch.token(), RealtimeIblSliceAttempt::Succeeded)
            == RealtimeIblCompletion::Published
        {
            return frame_number;
        }
        frame_number = frame_number.wrapping_add(1);
    }
    panic!("pending realtime IBL generation exceeded the bounded slice schedule");
}

fn resolve_pending_realtime_ibl_ticket(
    cache: &mut RealtimeIblCompiledGraphCache,
    scheduler: &mut RealtimeIblTimeSliceScheduler,
    request: &IblBakeArtifactRequest,
    frame_number: &mut u64,
) {
    while scheduler.is_rebake_pending() {
        let batch = scheduler
            .begin_frame(*frame_number)
            .expect("pending realtime IBL rebake must yield a batch");
        cache
            .resolve(request, &batch)
            .expect("ticket topology must compile or reuse");
        *frame_number += 1;
        scheduler.complete_frame(batch.token(), true);
    }
}
