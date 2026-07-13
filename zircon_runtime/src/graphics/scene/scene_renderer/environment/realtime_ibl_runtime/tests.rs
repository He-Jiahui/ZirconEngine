use super::*;

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
