use super::compiled_graph_cache::RealtimeIblCompiledGraphCache;
use super::*;
use crate::core::framework::render::{
    IblBakeArtifactRequest, SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};
use crate::graphics::backend::RenderBackend;

#[test]
fn realtime_runtime_defers_gpu_resource_creation_until_a_procedural_frame() {
    let runtime = RealtimeIblRuntime::new();

    assert!(!runtime.is_gpu_initialized());
    assert!(!runtime.gpu_timestamps_supported());
}

#[test]
fn first_procedural_frame_initializes_realtime_gpu_resources_and_starts_full_batch() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let mut runtime = RealtimeIblRuntime::new();

    let prepared = runtime.prepare_frame(&backend.device, ProceduralSkyParams::default_gradient());

    assert!(runtime.is_gpu_initialized());
    assert!(
        prepared
            .batch
            .as_ref()
            .is_some_and(|batch| batch.is_full_update())
    );
}

#[test]
fn initial_frame_samples_the_work_slot_written_before_scene_draws() {
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(8, 2).expect("valid test config"),
    );
    scheduler.request_rebake(ProceduralSkyParams::default_gradient().ibl_bake_key());
    let batch = scheduler.begin_frame(1).expect("initial batch");

    assert!(batch.is_full_update());
    assert_eq!(sampling_slot_for_batch(&batch), batch.work_slot());
}

#[test]
fn sliced_update_keeps_sampling_the_published_ready_slot() {
    let sky = ProceduralSkyParams::default_gradient();
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(8, 2).expect("valid test config"),
    );
    scheduler.request_rebake(sky.ibl_bake_key());
    let initial = scheduler.begin_frame(1).expect("initial batch");
    scheduler.complete_frame(initial.token(), true);
    let mut changed = sky;
    changed.horizon_color.x += 0.1;
    scheduler.request_rebake(changed.ibl_bake_key());
    let sliced = scheduler.begin_frame(2).expect("sliced batch");

    assert!(!sliced.is_full_update());
    assert_eq!(sampling_slot_for_batch(&sliced), sliced.ready_slot());
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
            RealtimeIblOperation::GenerateSourceMips,
            RealtimeIblOperation::ProjectDiffuseSh9,
        ]),
        "capture_sky+source_mips+diffuse_sh9"
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

    assert!(prepare.contains("let bake_key = runtime_bake_key(&sky);"));
    assert!(prepare.contains("request_for_key(bake_key)"));
    assert!(!prepare.contains("request_for_sky(&sky)"));
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

    assert_eq!(cache.variant_count(), 1);
    assert_eq!(cache.compile_count(), 1);
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
    assert_eq!(cache.variant_count(), 2);
    assert_eq!(cache.compile_count(), 2);
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
    assert_eq!(cache.variant_count(), 2);
    assert_eq!(cache.compile_count(), 2);
}

#[test]
fn realtime_ibl_compiled_graph_cache_distinguishes_request_geometry() {
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
    assert_eq!(cache.variant_count(), 2);
    assert_eq!(cache.compile_count(), 2);
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
