use super::*;
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
