use super::*;
use crate::core::framework::render::{
    EnvironmentExtract, FallbackSkyboxKind, PreviewEnvironmentExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, ViewportCameraSnapshot,
};
use crate::core::math::Vec4;

#[test]
fn queue_retains_at_most_one_pending_scene_snapshot() {
    let mut scheduler = EnvironmentCaptureScheduler::default();
    scheduler.request(scene(), request("a", 1)).unwrap();

    assert_eq!(
        scheduler.request(scene(), request("b", 1)),
        Err(RenderFrameworkError::EnvironmentCaptureQueueCapacityExceeded { limit: 1 })
    );
    assert_eq!(scheduler.telemetry().pending_capture_count, 1);
    assert_eq!(scheduler.telemetry().capacity_rejection_count, 1);
}

#[test]
fn duplicate_reuses_handle_and_new_generation_supersedes_queued_work() {
    let mut scheduler = EnvironmentCaptureScheduler::default();
    let first = scheduler.request(scene(), request("probe", 1)).unwrap();
    assert_eq!(
        scheduler.request(scene(), request("probe", 1)).unwrap(),
        first
    );

    let second = scheduler.request(scene(), request("probe", 2)).unwrap();
    assert_ne!(first, second);
    assert_eq!(
        scheduler.poll(first).unwrap().phase(),
        RenderEnvironmentCapturePhase::Superseded
    );
    assert_eq!(
        scheduler.poll(second).unwrap().phase(),
        RenderEnvironmentCapturePhase::Queued
    );
    assert_eq!(scheduler.telemetry().duplicate_request_count, 1);
    assert_eq!(scheduler.telemetry().superseded_capture_count, 1);
}

#[test]
fn report_tracks_pending_active_terminal_and_ready_payload_identities() {
    let mut scheduler = EnvironmentCaptureScheduler::default();
    let pending = scheduler.request(scene(), request("pending", 1)).unwrap();
    let pending_report = scheduler.report();
    assert_eq!(pending_report.pending_count, 1);
    assert_eq!(pending_report.pending_handle, Some(pending));
    assert_eq!(pending_report.pending_output_generation, Some(1));
    assert_eq!(
        pending_report.pending_bake_key,
        Some(request("pending", 1).ibl_bake_key())
    );
    assert_eq!(pending_report.active_count, 0);

    let work = scheduler.begin_next().unwrap();
    assert_eq!(work.handle(), pending);
    let active_report = scheduler.report();
    assert_eq!(active_report.pending_count, 0);
    assert_eq!(active_report.active_handle, Some(pending));
    assert_eq!(
        active_report.active_bake_key,
        Some(request("pending", 1).ibl_bake_key())
    );
    assert_eq!(
        active_report.active_phase,
        Some(RenderEnvironmentCapturePhase::Capturing)
    );
    assert_eq!(active_report.active_completed_work_items, 0);

    scheduler
        .advance_active(pending, RenderEnvironmentCapturePhase::Filtering, 6)
        .unwrap();
    assert_eq!(
        scheduler.report().active_phase,
        Some(RenderEnvironmentCapturePhase::Filtering)
    );
    assert_eq!(scheduler.report().active_completed_work_items, 6);
    finish_success(&mut scheduler, pending, None).unwrap();
    let terminal_report = scheduler.report();
    assert_eq!(terminal_report.active_count, 0);
    assert_eq!(terminal_report.terminal_status_count, 1);
    assert_eq!(terminal_report.succeeded_capture_count, 1);

    let persisted_request = persistence_request("persisted", 1);
    let persisted = scheduler
        .request(scene(), persisted_request.clone())
        .unwrap();
    scheduler.begin_next().unwrap();
    scheduler
        .advance_active(
            persisted,
            RenderEnvironmentCapturePhase::Persisting,
            RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
        )
        .unwrap();
    let source_payload = source_payload(persisted, &persisted_request);
    let source_payload_bytes = source_payload.source_rgba16f_bytes().len() as u64;
    finish_success(&mut scheduler, persisted, Some(source_payload)).unwrap();
    let payload_report = scheduler.report();
    assert_eq!(payload_report.ready_source_payload_handle, Some(persisted));
    assert_eq!(
        payload_report.ready_source_payload_bake_key,
        Some(persisted_request.ibl_bake_key())
    );
    assert_eq!(payload_report.source_payload_publish_count, 1);
    assert_eq!(
        payload_report.ready_source_payload_bytes,
        source_payload_bytes
    );
    assert_eq!(
        payload_report.peak_ready_source_payload_bytes,
        source_payload_bytes
    );
    assert_eq!(
        payload_report.cumulative_source_payload_bytes,
        source_payload_bytes
    );
    assert_eq!(
        scheduler
            .take_source_payload(persisted)
            .unwrap()
            .unwrap()
            .handle(),
        persisted
    );
    let consumed_report = scheduler.report();
    assert!(consumed_report.ready_source_payload_bake_key.is_none());
    assert_eq!(consumed_report.ready_source_payload_bytes, 0);
    assert_eq!(
        consumed_report.peak_ready_source_payload_bytes,
        source_payload_bytes
    );
    assert_eq!(
        consumed_report.cumulative_source_payload_bytes,
        source_payload_bytes
    );
}

#[test]
fn report_reads_precomputed_identity_without_query_time_hashing_or_payload_copy() {
    let source = include_str!("../environment_capture_scheduler/report.rs");
    assert!(!source.contains("ibl_bake_key()"));
    assert!(!source.contains("runtime_cache_artifact_request"));
    assert!(!source.contains("source_rgba16f_bytes"));
    assert!(!source.contains("clone()"));
}

#[test]
fn superseded_active_gpu_work_cannot_publish_output() {
    let mut scheduler = EnvironmentCaptureScheduler::default();
    let first = scheduler.request(scene(), request("probe", 1)).unwrap();
    let work = scheduler.begin_next().unwrap();
    assert_eq!(work.handle(), first);
    assert_eq!(work.request().output_generation(), 1);
    assert!(work.scene().scene.meshes.is_empty());
    scheduler
        .advance_active(first, RenderEnvironmentCapturePhase::Filtering, 6)
        .unwrap();

    let second = scheduler.request(scene(), request("probe", 2)).unwrap();
    assert_eq!(
        finish_success(&mut scheduler, first, None).unwrap(),
        EnvironmentCapturePublication::Discard
    );
    assert_eq!(
        scheduler.poll(first).unwrap().phase(),
        RenderEnvironmentCapturePhase::Superseded
    );
    assert!(scheduler.poll(first).unwrap().output().is_none());

    scheduler.begin_next().unwrap();
    scheduler
        .advance_active(second, RenderEnvironmentCapturePhase::Filtering, 6)
        .unwrap();
    assert_eq!(
        finish_success(&mut scheduler, second, None).unwrap(),
        EnvironmentCapturePublication::Publish
    );
    let status = scheduler.poll(second).unwrap();
    assert_eq!(status.phase(), RenderEnvironmentCapturePhase::Succeeded);
    assert_eq!(status.output().unwrap().output_generation(), 2);
}

#[test]
fn work_item_transfer_boundary_moves_scene_and_request_without_clone() {
    let source = include_str!("../environment_capture_scheduler.rs");
    let method = source
        .split("fn into_parts(")
        .nth(1)
        .and_then(|source| source.split("pub(in crate::graphics) fn scene").next())
        .expect("consuming work-item transfer method");

    assert!(method.contains("self.handle"));
    assert!(method.contains("self.scene"));
    assert!(method.contains("self.request"));
    assert!(!method.contains("clone()"));
}

#[test]
fn cancellation_is_idempotent_and_suppresses_active_success() {
    let mut scheduler = EnvironmentCaptureScheduler::default();
    let handle = scheduler.request(scene(), request("probe", 1)).unwrap();
    scheduler.begin_next().unwrap();
    scheduler.cancel(handle).unwrap();
    scheduler.cancel(handle).unwrap();
    assert_eq!(
        finish_success(&mut scheduler, handle, None).unwrap(),
        EnvironmentCapturePublication::Discard
    );
    scheduler.cancel(handle).unwrap();

    let status = scheduler.poll(handle).unwrap();
    assert_eq!(status.phase(), RenderEnvironmentCapturePhase::Cancelled);
    assert!(status.output().is_none());
    assert_eq!(scheduler.telemetry().cancellation_request_count, 1);
}

#[test]
fn live_generation_rejects_stale_requests_and_handle_overflow_is_typed() {
    let mut scheduler = EnvironmentCaptureScheduler::default();
    scheduler.request(scene(), request("probe", 2)).unwrap();
    assert_eq!(
        scheduler.request(scene(), request("probe", 1)),
        Err(RenderFrameworkError::EnvironmentCaptureGenerationNotNewer {
            capture_id: "probe".to_string(),
            requested_generation: 1,
            live_generation: 2,
        })
    );

    let mut exhausted = EnvironmentCaptureScheduler::default();
    exhausted.set_next_handle_for_tests(0);
    assert_eq!(
        exhausted.request(scene(), request("probe", 1)),
        Err(RenderFrameworkError::EnvironmentCaptureHandleSpaceExhausted)
    );
}

#[test]
fn terminal_generation_rejects_replayed_older_request() {
    let mut scheduler = EnvironmentCaptureScheduler::default();
    let handle = scheduler.request(scene(), request("probe", 2)).unwrap();
    scheduler.begin_next().unwrap();
    assert_eq!(
        finish_success(&mut scheduler, handle, None),
        Err(EnvironmentCaptureTransitionError::IncompleteSuccess {
            phase: RenderEnvironmentCapturePhase::Capturing,
            completed_work_items: 0,
        })
    );
    scheduler
        .advance_active(handle, RenderEnvironmentCapturePhase::Filtering, 6)
        .unwrap();
    assert_eq!(
        finish_success(&mut scheduler, handle, None).unwrap(),
        EnvironmentCapturePublication::Publish
    );

    assert_eq!(
        scheduler.request(scene(), request("probe", 1)),
        Err(RenderFrameworkError::EnvironmentCaptureGenerationNotNewer {
            capture_id: "probe".to_string(),
            requested_generation: 1,
            live_generation: 2,
        })
    );
}

#[test]
fn terminal_history_is_bounded_and_progress_cannot_regress() {
    let mut scheduler = EnvironmentCaptureScheduler::default();
    let first = scheduler.request(scene(), request("probe-0", 1)).unwrap();
    scheduler.begin_next().unwrap();
    scheduler
        .advance_active(first, RenderEnvironmentCapturePhase::Capturing, 2)
        .unwrap();
    assert_eq!(
        scheduler.advance_active(first, RenderEnvironmentCapturePhase::Capturing, 1),
        Err(EnvironmentCaptureTransitionError::ProgressRegression {
            previous: 2,
            next: 1,
        })
    );
    assert_eq!(
        scheduler.advance_active(first, RenderEnvironmentCapturePhase::Filtering, 2),
        Ok(())
    );
    assert_eq!(
        scheduler.advance_active(first, RenderEnvironmentCapturePhase::Capturing, 2),
        Err(EnvironmentCaptureTransitionError::PhaseRegression {
            previous: RenderEnvironmentCapturePhase::Filtering,
            next: RenderEnvironmentCapturePhase::Capturing,
        })
    );
    scheduler.finish_active_failure(first, "test").unwrap();

    let mut latest = first;
    for index in 1..=ENVIRONMENT_CAPTURE_TERMINAL_STATUS_CAPACITY {
        latest = scheduler
            .request(scene(), request(&format!("probe-{index}"), 1))
            .unwrap();
        scheduler.begin_next().unwrap();
        scheduler.finish_active_failure(latest, "test").unwrap();
    }

    assert_eq!(
        scheduler.poll(first),
        Err(RenderFrameworkError::UnknownEnvironmentCaptureHandle {
            handle: first.get(),
        })
    );
    assert_eq!(
        scheduler.poll(latest).unwrap().phase(),
        RenderEnvironmentCapturePhase::Failed
    );
    assert_eq!(
        scheduler.telemetry().terminal_status_count,
        ENVIRONMENT_CAPTURE_TERMINAL_STATUS_CAPACITY
    );
    assert_eq!(scheduler.telemetry().terminal_status_eviction_count, 1);
}

#[test]
fn framework_control_plane_does_not_drain_frames_or_lock_renderer_state() {
    let source = include_str!("../render_framework_trait_binding/wgpu_framework.rs");
    let control_plane = source
        .split("fn request_environment_capture")
        .nth(1)
        .and_then(|source| source.split("fn capture_frame_if_newer").next())
        .expect("environment capture request/poll/cancel trait binding");

    assert!(control_plane.contains(".environment_captures"));
    assert!(!control_plane.contains("finish_submission"));
    assert!(!control_plane.contains("lock_operation"));
    assert!(!control_plane.contains("lock_state"));
}

#[test]
fn persisted_source_payload_is_taken_once_and_backpressures_only_persistence_requests() {
    let mut scheduler = EnvironmentCaptureScheduler::default();
    let capture_request = persistence_request("atrium", 1);
    let handle = scheduler.request(scene(), capture_request.clone()).unwrap();
    scheduler.begin_next().unwrap();
    scheduler
        .advance_active(
            handle,
            RenderEnvironmentCapturePhase::Persisting,
            RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
        )
        .unwrap();

    assert_eq!(
        finish_success(&mut scheduler, handle, None),
        Err(EnvironmentCaptureTransitionError::PersistenceSourcePayloadRequired)
    );
    assert_eq!(
        finish_success(
            &mut scheduler,
            handle,
            Some(source_payload(handle, &capture_request)),
        )
        .unwrap(),
        EnvironmentCapturePublication::Publish
    );
    assert_eq!(
        scheduler.request(scene(), persistence_request("lobby", 1)),
        Err(RenderFrameworkError::EnvironmentCapturePersistenceResultCapacityExceeded { limit: 1 })
    );
    assert!(scheduler
        .request(scene(), request("runtime-only", 1))
        .is_ok());

    let payload = scheduler.take_source_payload(handle).unwrap().unwrap();
    assert_eq!(payload.handle(), handle);
    assert!(scheduler.take_source_payload(handle).unwrap().is_none());
    assert_eq!(scheduler.telemetry().source_payload_backpressure_count, 1);
    assert_eq!(scheduler.telemetry().source_payload_take_count, 1);
}

#[test]
fn runtime_only_success_preserves_an_unconsumed_persistence_payload() {
    let mut scheduler = EnvironmentCaptureScheduler::default();
    let persistence_request = persistence_request("atrium", 1);
    let persistence_handle = scheduler
        .request(scene(), persistence_request.clone())
        .unwrap();
    scheduler.begin_next().unwrap();
    scheduler
        .advance_active(
            persistence_handle,
            RenderEnvironmentCapturePhase::Persisting,
            RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
        )
        .unwrap();
    assert_eq!(
        finish_success(
            &mut scheduler,
            persistence_handle,
            Some(source_payload(persistence_handle, &persistence_request)),
        )
        .unwrap(),
        EnvironmentCapturePublication::Publish
    );

    let runtime_handle = scheduler
        .request(scene(), request("runtime-only", 1))
        .unwrap();
    scheduler.begin_next().unwrap();
    scheduler
        .advance_active(
            runtime_handle,
            RenderEnvironmentCapturePhase::Filtering,
            RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
        )
        .unwrap();
    assert_eq!(
        finish_success(&mut scheduler, runtime_handle, None).unwrap(),
        EnvironmentCapturePublication::Publish
    );

    let payload = scheduler
        .take_source_payload(persistence_handle)
        .unwrap()
        .expect("runtime-only success must not clear the persistence mailbox");
    assert_eq!(payload.handle(), persistence_handle);
}

#[test]
fn cancellation_discards_a_completed_source_payload_before_publication() {
    let mut scheduler = EnvironmentCaptureScheduler::default();
    let capture_request = persistence_request("atrium", 1);
    let handle = scheduler.request(scene(), capture_request.clone()).unwrap();
    scheduler.begin_next().unwrap();
    scheduler
        .advance_active(
            handle,
            RenderEnvironmentCapturePhase::Persisting,
            RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
        )
        .unwrap();
    scheduler.cancel(handle).unwrap();

    assert_eq!(
        finish_success(
            &mut scheduler,
            handle,
            Some(source_payload(handle, &capture_request)),
        )
        .unwrap(),
        EnvironmentCapturePublication::Discard
    );
    assert_eq!(
        scheduler.poll(handle).unwrap().phase(),
        RenderEnvironmentCapturePhase::Cancelled
    );
    assert!(scheduler.take_source_payload(handle).unwrap().is_none());
    let report = scheduler.report();
    assert_eq!(report.source_payload_publish_count, 0);
    assert_eq!(report.ready_source_payload_bytes, 0);
    assert_eq!(report.peak_ready_source_payload_bytes, 0);
    assert_eq!(report.cumulative_source_payload_bytes, 0);
}

#[test]
fn gpu_framework_work_item_entrypoint_only_takes_scheduler_lock() {
    let source = include_str!("../wgpu_render_framework/wgpu_render_framework.rs");
    let framework_impl = source
        .split("impl WgpuRenderFramework {")
        .nth(1)
        .expect("WGPU framework implementation");
    let method = framework_impl
        .split("fn begin_environment_capture_work_item(")
        .nth(1)
        .and_then(|source| source.split("/// Publishes capture progress").next())
        .expect("framework work-item entrypoint");

    assert!(method.contains("environment_captures"));
    assert!(method.contains("begin_next()"));
    assert!(!method.contains("lock_operation"));
    assert!(!method.contains("lock_state"));
    assert!(!method.contains("finish_submission"));
}

#[test]
fn gpu_framework_progress_entrypoints_only_touch_scheduler_state() {
    let source = include_str!("../wgpu_render_framework/wgpu_render_framework.rs");
    let framework_impl = source
        .split("impl WgpuRenderFramework {")
        .nth(1)
        .expect("WGPU framework implementation");
    let method = framework_impl
        .split("fn advance_environment_capture_work_item(")
        .nth(1)
        .and_then(|source| source.split("/// Publishes physical output").next())
        .expect("framework progress entrypoint");
    assert!(method.contains("environment_captures"));
    assert!(method.contains("advance_active"));
    assert!(!method.contains("lock_operation"));
    assert!(!method.contains("lock_state"));
    assert!(!method.contains("finish_submission"));

    let success = framework_impl
        .split("fn settle_environment_capture_work_item_success(")
        .nth(1)
        .and_then(|source| source.split("/// Records a recorder").next())
        .expect("framework success entrypoint");
    assert!(success.contains("environment_captures"));
    assert!(success.contains("finish_active_success_with_publication"));
    assert!(!success.contains("lock_operation"));
    assert!(!success.contains("lock_state"));

    let failure = framework_impl
        .split("fn finish_environment_capture_work_item_failure(")
        .nth(1)
        .and_then(|source| source.split("/// Compiles an existing Plan08").next())
        .expect("framework failure entrypoint");
    assert!(failure.contains("finish_active_failure"));
    assert!(!failure.contains("lock_operation"));
    assert!(!failure.contains("lock_state"));
}

fn request(capture_id: &str, output_generation: u64) -> RenderEnvironmentCaptureRequest {
    RenderEnvironmentCaptureRequest::with_revisions(
        capture_id,
        [0.0; 3],
        0.1,
        200.0,
        128,
        crate::core::framework::render::SourceCubemapPrefilterQuality::Normal,
        output_generation,
        output_generation,
        output_generation,
    )
    .unwrap()
}

fn persistence_request(
    capture_id: &str,
    output_generation: u64,
) -> RenderEnvironmentCaptureRequest {
    RenderEnvironmentCaptureRequest::with_revisions(
        capture_id,
        [0.0; 3],
        0.1,
        200.0,
        16,
        crate::core::framework::render::SourceCubemapPrefilterQuality::Normal,
        output_generation,
        output_generation,
        output_generation,
    )
    .unwrap()
    .with_persistence_output_uri(format!("res://probes/{capture_id}.zcube"))
    .unwrap()
}

fn source_payload(
    handle: RenderEnvironmentCaptureHandle,
    request: &RenderEnvironmentCaptureRequest,
) -> RenderEnvironmentCaptureSourcePayload {
    let mip_count = crate::core::framework::render::source_cubemap_mip_count(request.face_size());
    let byte_len =
        crate::core::framework::render::source_cubemap_sample_count(request.face_size(), mip_count)
            * crate::core::framework::render::RGBA16F_TEXEL_SIZE_BYTES;
    RenderEnvironmentCaptureSourcePayload::new(
        handle,
        RenderEnvironmentCaptureOutputIdentity::from_request(request),
        request.face_size(),
        mip_count,
        vec![0; byte_len],
    )
    .unwrap()
}

fn finish_success(
    scheduler: &mut EnvironmentCaptureScheduler,
    handle: RenderEnvironmentCaptureHandle,
    source_payload: Option<RenderEnvironmentCaptureSourcePayload>,
) -> Result<EnvironmentCapturePublication, EnvironmentCaptureTransitionError> {
    let mut publication = None;
    scheduler.finish_active_success_with_publication(
        handle,
        source_payload,
        |disposition, scheduler_before_publication| {
            let status = scheduler_before_publication.poll(handle).unwrap();
            assert!(!status.phase().is_terminal());
            assert!(!scheduler_before_publication
                .ready_source_payload
                .as_ref()
                .is_some_and(|payload| payload.handle() == handle));
            publication = Some(disposition);
        },
    )?;
    Ok(publication.expect("success settlement must publish or discard exactly once"))
}

fn scene() -> SceneViewportRenderPacket {
    SceneViewportRenderPacket {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        environment: EnvironmentExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    }
}
