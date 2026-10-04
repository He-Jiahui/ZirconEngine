use std::sync::{mpsc, Arc};
use std::thread;

use zr_contracts::random::{RandomPurposeKey, RandomStreamKey, RandomSystemKey, RandomWorldKey};

use super::super::RandomStream;
use super::RandomService;

#[test]
fn release_and_reseed_do_not_depend_on_arc_drop_timing() {
    let mut service = RandomService::new(67);
    let key = RandomStreamKey::for_world(
        RandomWorldKey::new(1, 0),
        RandomSystemKey::new(2),
        RandomPurposeKey::new(3),
        4,
    );
    let lease = service.acquire_stream(key).expect("stream admission");
    let release_tail = Arc::clone(&service.authority);
    let (released_tx, released_rx) = mpsc::channel();
    let (finish_tx, finish_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        drop(lease);
        released_tx.send(()).expect("release signal receiver");
        let _ = finish_rx.recv();
        drop(release_tail);
    });

    released_rx.recv().expect("release signal sender");
    assert_eq!(service.active_lease_count(), 0);
    let reseed = service.reseed(71);
    let _ = finish_tx.send(());
    worker.join().expect("release worker should not panic");
    reseed.expect("reseed must use registry state, not Arc uniqueness");
    assert_eq!(service.master_seed(), 71);
    assert_eq!(service.master_seed_generation(), 1);
}

#[test]
fn checkpoint_prevents_reseed_from_crossing_the_captured_stream_era() {
    const INITIAL_SEED: u64 = 0x2200;
    const RESEEDED_SEED: u64 = 0x4400;

    let service = Arc::new(RandomService::new(INITIAL_SEED));
    let key = RandomStreamKey::for_world(
        RandomWorldKey::new(5, 1),
        RandomSystemKey::new(7),
        RandomPurposeKey::new(11),
        13,
    );
    let mut lease = service.acquire_stream(key).expect("stream admission");
    lease.try_next_u32().expect("advance parked stream");
    lease.release();

    let before = service.checkpoint().expect("baseline checkpoint");
    let before_service = before.service_state();
    let before_stream = before.streams()[0].state();

    let checkpoint_service = Arc::clone(&service);
    let (captured_tx, captured_rx) = mpsc::sync_channel(0);
    let (resume_tx, resume_rx) = mpsc::sync_channel(0);
    let checkpoint_worker = thread::spawn(move || {
        checkpoint_service.checkpoint_with_stream_capture_hook(|| {
            captured_tx.send(()).expect("capture observer");
            resume_rx.recv().expect("checkpoint resume signal");
        })
    });
    captured_rx.recv().expect("stream capture signal");
    assert!(
        service.authority.registry().lock_is_held_for_test(),
        "checkpoint must retain the registry lock after capturing stream entries"
    );

    let reseed_authority = Arc::clone(&service.authority);
    let (reseed_attempt_tx, reseed_attempt_rx) = mpsc::sync_channel(0);
    let (reseed_entered_tx, reseed_entered_rx) = mpsc::channel();
    let (reseed_done_tx, reseed_done_rx) = mpsc::channel();
    let reseed_worker = thread::spawn(move || {
        reseed_attempt_tx.send(()).expect("reseed attempt observer");
        let result = reseed_authority.reseed_with_test_observer(RESEEDED_SEED, || {
            reseed_entered_tx.send(()).expect("reseed entry observer");
        });
        reseed_done_tx.send(()).expect("reseed completion observer");
        result
    });
    reseed_attempt_rx.recv().expect("reseed attempt signal");
    assert!(
        matches!(reseed_entered_rx.try_recv(), Err(mpsc::TryRecvError::Empty)),
        "reseed must not enter while checkpoint capture is paused"
    );
    assert!(
        matches!(reseed_done_rx.try_recv(), Err(mpsc::TryRecvError::Empty)),
        "reseed must remain blocked while checkpoint capture is paused"
    );

    resume_tx.send(()).expect("checkpoint resume receiver");
    reseed_entered_rx
        .recv()
        .expect("reseed entry after checkpoint resume");
    let checkpoint = checkpoint_worker
        .join()
        .expect("checkpoint worker should not panic")
        .expect("checkpoint should succeed");
    reseed_worker
        .join()
        .expect("reseed worker should not panic")
        .expect("idle service should reseed");
    reseed_done_rx.recv().expect("reseed completion signal");

    assert_eq!(checkpoint.service_state(), before_service);
    assert_eq!(checkpoint.streams()[0].state(), before_stream);
    assert_eq!(service.master_seed(), RESEEDED_SEED);
    assert_eq!(service.master_seed_generation(), 1);
    assert_eq!(service.registered_stream_count(), 0);

    let mut expected = RandomStream::from_state(before_stream).expect("valid parked state");
    let expected_next = expected.try_next_u32();
    let expected_draw_index = expected.draw_index();
    let restored = RandomService::from_checkpoint(checkpoint).expect("checkpoint restore");
    let mut restored_lease = restored.acquire_stream(key).expect("restored stream lease");
    assert_eq!(restored_lease.try_next_u32(), expected_next);
    assert_eq!(restored_lease.draw_index(), expected_draw_index);
}
