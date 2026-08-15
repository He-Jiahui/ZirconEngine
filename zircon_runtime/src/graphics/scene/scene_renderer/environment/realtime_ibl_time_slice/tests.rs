use super::*;
use crate::core::framework::render::ProceduralSkyParams;

fn key_with_revision(revision: u64) -> IblBakeKey {
    let mut params = ProceduralSkyParams::default_gradient();
    params.source_revision = revision;
    params.ibl_bake_key()
}

fn scheduler() -> RealtimeIblTimeSliceScheduler {
    RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(8, 2).expect("valid test config"),
    )
}

#[test]
fn first_update_builds_and_publishes_a_complete_environment_in_one_frame() {
    let mut scheduler = scheduler();
    let key = key_with_revision(1);
    assert!(scheduler.request_rebake(key));

    let batch = scheduler.begin_frame(10).expect("initial full batch");
    assert!(batch.is_full_update());
    assert_eq!(batch.ready_slot(), IblRealtimeBufferSlot::A);
    assert_eq!(batch.work_slot(), IblRealtimeBufferSlot::B);
    assert_eq!(
        batch.operations(),
        &[
            RealtimeIblOperation::CaptureSky(CubeFaceRange::ALL),
            RealtimeIblOperation::CaptureCloud(CubeFaceRange::ALL),
            RealtimeIblOperation::GenerateSourceMips,
            RealtimeIblOperation::Prefilter {
                mips: CubeMipRange::new(0, 8),
                faces: CubeFaceRange::ALL,
            },
            RealtimeIblOperation::ProjectDiffuseSh9,
        ]
    );
    assert_eq!(scheduler.published_key(), None);

    assert_eq!(
        scheduler.complete_frame(batch.token(), true),
        RealtimeIblCompletion::Published
    );
    assert_eq!(scheduler.published_key(), Some(key));
    assert_eq!(scheduler.ready_slot(), IblRealtimeBufferSlot::B);
    assert!(!scheduler.is_rebake_pending());
}

#[test]
fn subsequent_update_matches_unreal_twelve_state_face_and_mip_schedule() {
    let mut scheduler = scheduler();
    publish_initial(&mut scheduler, key_with_revision(1));
    assert!(scheduler.request_rebake(key_with_revision(2)));

    let mut observed = Vec::new();
    for frame in 100..116 {
        let batch = scheduler.begin_frame(frame).expect("time-sliced batch");
        assert!(!batch.is_full_update());
        observed.push((batch.logical_state(), batch.operations().to_vec()));
        let expected_completion = if frame == 115 {
            RealtimeIblCompletion::Published
        } else {
            RealtimeIblCompletion::Advanced
        };
        assert_eq!(
            scheduler.complete_frame(batch.token(), true),
            expected_completion
        );
    }

    assert_eq!(
        observed.iter().map(|entry| entry.0).collect::<Vec<_>>(),
        vec![0, 0, 0, 1, 1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    );
    assert_eq!(
        observed[0].1,
        vec![RealtimeIblOperation::CaptureSky(CubeFaceRange::new(0, 2))]
    );
    assert_eq!(
        observed[2].1,
        vec![RealtimeIblOperation::CaptureSky(CubeFaceRange::new(4, 2))]
    );
    assert_eq!(
        observed[3].1,
        vec![RealtimeIblOperation::CaptureCloud(CubeFaceRange::new(0, 2))]
    );
    assert_eq!(
        observed[7].1,
        vec![RealtimeIblOperation::Prefilter {
            mips: CubeMipRange::new(0, 1),
            faces: CubeFaceRange::new(0, 2),
        }]
    );
    assert_eq!(
        observed[13].1,
        vec![RealtimeIblOperation::Prefilter {
            mips: CubeMipRange::new(4, 2),
            faces: CubeFaceRange::ALL,
        }]
    );
    assert_eq!(
        observed[14].1,
        vec![RealtimeIblOperation::Prefilter {
            mips: CubeMipRange::new(6, 2),
            faces: CubeFaceRange::ALL,
        }]
    );
    assert_eq!(
        observed[15].1,
        vec![RealtimeIblOperation::ProjectDiffuseSh9]
    );
}

#[test]
fn parameter_change_discards_old_work_without_replacing_the_ready_environment() {
    let mut scheduler = scheduler();
    let published = key_with_revision(1);
    publish_initial(&mut scheduler, published);
    scheduler.request_rebake(key_with_revision(2));
    let obsolete = scheduler.begin_frame(20).expect("old generation batch");

    let newest = key_with_revision(3);
    assert!(scheduler.request_rebake(newest));
    assert_eq!(scheduler.published_key(), Some(published));
    assert_eq!(
        scheduler.complete_frame(obsolete.token(), true),
        RealtimeIblCompletion::Stale
    );

    let restarted = scheduler.begin_frame(21).expect("restarted batch");
    assert_eq!(restarted.logical_state(), 0);
    assert_eq!(restarted.work_slot(), IblRealtimeBufferSlot::A);
    assert_eq!(scheduler.published_key(), Some(published));
    assert_eq!(scheduler.pending_key(), Some(newest));
}

#[test]
fn failed_gpu_slice_retries_the_same_work_and_same_frame_is_idempotent() {
    let mut scheduler = scheduler();
    publish_initial(&mut scheduler, key_with_revision(1));
    scheduler.request_rebake(key_with_revision(2));

    let first = scheduler.begin_frame(30).expect("first sliced batch");
    assert_eq!(scheduler.begin_frame(30), Some(first.clone()));
    assert_eq!(
        scheduler.complete_frame(first.token(), false),
        RealtimeIblCompletion::Retry
    );

    let retry = scheduler.begin_frame(31).expect("retry batch");
    assert_eq!(retry.logical_state(), first.logical_state());
    assert_eq!(retry.operations(), first.operations());
    assert_eq!(scheduler.published_key(), Some(key_with_revision(1)));
}

#[test]
fn unavailable_high_mip_states_advance_without_zero_length_gpu_operations() {
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(4, 2).expect("valid test config"),
    );
    publish_initial(&mut scheduler, key_with_revision(1));
    scheduler.request_rebake(key_with_revision(2));

    for frame in 100..116 {
        let batch = scheduler.begin_frame(frame).expect("time-sliced batch");
        if matches!(batch.logical_state(), 9 | 10) {
            assert!(batch.operations().is_empty());
        }
        scheduler.complete_frame(batch.token(), true);
    }

    assert_eq!(scheduler.published_key(), Some(key_with_revision(2)));
}

#[test]
fn requesting_the_published_key_cancels_obsolete_pending_work() {
    let mut scheduler = scheduler();
    let published = key_with_revision(1);
    publish_initial(&mut scheduler, published);
    scheduler.request_rebake(key_with_revision(2));
    let obsolete = scheduler.begin_frame(40).expect("obsolete batch");

    assert!(scheduler.request_rebake(published));
    assert!(!scheduler.is_rebake_pending());
    assert_eq!(scheduler.begin_frame(41), None);
    assert_eq!(
        scheduler.complete_frame(obsolete.token(), true),
        RealtimeIblCompletion::Stale
    );
    assert_eq!(scheduler.published_key(), Some(published));
}

#[test]
fn frame_batches_expand_to_parameterized_prefilter_dispatch_slices() {
    let mut scheduler = scheduler();
    publish_initial(&mut scheduler, key_with_revision(1));
    scheduler.request_rebake(key_with_revision(2));

    for frame in 100..107 {
        let batch = scheduler.begin_frame(frame).expect("scheduled batch");
        scheduler.complete_frame(batch.token(), true);
    }
    let mip_zero_faces_zero_and_one = scheduler.begin_frame(107).expect("PMREM slice");

    assert_eq!(
        mip_zero_faces_zero_and_one.prefilter_dispatch_slices(),
        vec![RealtimeIblPrefilterDispatchSlice {
            mip_level: 0,
            first_face: 0,
            face_count: 2,
        }]
    );
}

#[test]
fn full_update_expands_every_pmrem_mip_over_all_cube_faces() {
    let mut scheduler = scheduler();
    scheduler.request_rebake(key_with_revision(1));
    let batch = scheduler.begin_frame(1).expect("full update");

    let slices = batch.prefilter_dispatch_slices();
    assert_eq!(slices.len(), 8);
    assert_eq!(slices.first().map(|slice| slice.mip_level), Some(0));
    assert_eq!(slices.last().map(|slice| slice.mip_level), Some(7));
    assert!(slices
        .iter()
        .all(|slice| slice.first_face == 0 && slice.face_count == 6));
}

fn publish_initial(scheduler: &mut RealtimeIblTimeSliceScheduler, key: IblBakeKey) {
    scheduler.request_rebake(key);
    let batch = scheduler.begin_frame(1).expect("initial batch");
    assert_eq!(
        scheduler.complete_frame(batch.token(), true),
        RealtimeIblCompletion::Published
    );
}
