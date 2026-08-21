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
fn initial_generation_is_time_sliced_and_publishes_only_after_sh9() {
    let mut scheduler = scheduler();
    let key = key_with_revision(1);
    assert!(scheduler.request_rebake(key));

    let mut observed = Vec::new();
    for frame in 10..40 {
        let batch = scheduler.begin_frame(frame).expect("generation batch");
        observed.push(batch.clone());
        let completion = scheduler.complete_frame(batch.token(), true);
        if completion == RealtimeIblCompletion::Published {
            break;
        }
        assert_eq!(completion, RealtimeIblCompletion::Advanced);
    }

    assert_eq!(observed.len(), 21);
    assert!(observed.iter().all(|batch| batch.operations().len() == 1));
    assert_eq!(
        observed
            .iter()
            .filter(|batch| matches!(batch.operations(), [RealtimeIblOperation::CaptureSky(_)]))
            .count(),
        3,
        "the ticket has exactly three two-face sky captures"
    );
    assert_eq!(observed[0].ready_slot(), IblRealtimeBufferSlot::A);
    assert_eq!(observed[0].work_slot(), IblRealtimeBufferSlot::B);
    assert_eq!(
        observed[0].operations(),
        &[RealtimeIblOperation::CaptureSky(CubeFaceRange::new(0, 2))]
    );
    assert_eq!(
        observed[2].operations(),
        &[RealtimeIblOperation::CaptureSky(CubeFaceRange::new(4, 2))]
    );
    assert_eq!(
        observed[3].operations(),
        &[RealtimeIblOperation::GenerateSourceMip { mip_level: 1 }]
    );
    assert_eq!(
        observed[9].operations(),
        &[RealtimeIblOperation::GenerateSourceMip { mip_level: 7 }]
    );
    assert_eq!(
        observed[10].operations(),
        &[RealtimeIblOperation::Prefilter {
            mips: CubeMipRange::new(0, 1),
            faces: CubeFaceRange::new(0, 2),
        }]
    );
    assert_eq!(
        observed[19].operations(),
        &[RealtimeIblOperation::Prefilter {
            mips: CubeMipRange::new(7, 1),
            faces: CubeFaceRange::ALL,
        }]
    );
    assert_eq!(
        observed[20].operations(),
        &[RealtimeIblOperation::ProjectDiffuseSh9]
    );
    assert_eq!(scheduler.published_key(), Some(key));
    assert_eq!(scheduler.ready_slot(), IblRealtimeBufferSlot::B);
    assert!(!scheduler.is_rebake_pending());
}

#[test]
fn generated_environment_uses_a_bounded_topology_cache_budget() {
    let mut scheduler = scheduler();
    scheduler.request_rebake(key_with_revision(1));
    let batch = scheduler.begin_frame(1).expect("first generation batch");

    // Twenty-one ticket stages across two ping-pong slots are the only
    // topologies this fixed configuration may retain at once.
    assert_eq!(batch.topology_cache_capacity(), 42);
}

#[test]
fn parameter_change_discards_old_work_without_replacing_the_ready_environment() {
    let mut scheduler = scheduler();
    let published = key_with_revision(1);
    publish_generation(&mut scheduler, published);
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
fn failed_gpu_slice_retries_the_same_ticket_operation_and_frame_is_idempotent() {
    let mut scheduler = scheduler();
    publish_generation(&mut scheduler, key_with_revision(1));
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
fn single_mip_generation_has_no_empty_gpu_operations() {
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(1, 2).expect("valid test config"),
    );
    let key = key_with_revision(1);
    scheduler.request_rebake(key);

    let mut completed = 0;
    for frame in 100..120 {
        let batch = scheduler.begin_frame(frame).expect("generation batch");
        assert_eq!(batch.operations().len(), 1);
        completed += 1;
        if scheduler.complete_frame(batch.token(), true) == RealtimeIblCompletion::Published {
            break;
        }
    }

    assert_eq!(completed, 7);
    assert_eq!(scheduler.published_key(), Some(key));
}

#[test]
fn requesting_the_published_key_cancels_obsolete_pending_work() {
    let mut scheduler = scheduler();
    let published = key_with_revision(1);
    publish_generation(&mut scheduler, published);
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
fn source_mips_complete_before_pmrem_face_slices() {
    let mut scheduler = scheduler();
    scheduler.request_rebake(key_with_revision(1));

    for frame in 100..110 {
        let batch = scheduler.begin_frame(frame).expect("scheduled batch");
        scheduler.complete_frame(batch.token(), true);
    }
    let mip_zero_faces_zero_and_one = scheduler.begin_frame(110).expect("PMREM slice");

    assert_eq!(
        mip_zero_faces_zero_and_one.prefilter_dispatch_slices(),
        vec![RealtimeIblPrefilterDispatchSlice {
            mip_level: 0,
            first_face: 0,
            face_count: 2,
        }]
    );
}

fn publish_generation(scheduler: &mut RealtimeIblTimeSliceScheduler, key: IblBakeKey) {
    scheduler.request_rebake(key);
    for frame in 1..64 {
        let batch = scheduler.begin_frame(frame).expect("generation batch");
        if scheduler.complete_frame(batch.token(), true) == RealtimeIblCompletion::Published {
            return;
        }
    }
    panic!("realtime IBL generation did not publish within the ticket budget");
}
