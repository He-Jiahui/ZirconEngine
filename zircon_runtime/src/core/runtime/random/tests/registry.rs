use std::sync::{Arc, Barrier};
use std::thread;

use super::super::{RandomService, RandomServiceError};
use super::{key, keyed};

#[test]
fn same_key_has_one_mutable_authority_and_reacquire_resumes_progress() {
    let service = RandomService::new(31);
    let mut lease = service.acquire_stream(key()).expect("first admission");
    let first = lease.try_next_u32().expect("first draw");
    assert_eq!(service.active_lease_count(), 1);
    assert_eq!(
        service
            .acquire_stream(key())
            .expect_err("a second mutable authority must be rejected"),
        RandomServiceError::StreamAlreadyAcquired { key: key() }
    );
    lease.release();

    let mut resumed = service.acquire_stream(key()).expect("resumed admission");
    assert_ne!(resumed.try_next_u32().expect("second draw"), first);
    assert_eq!(resumed.draw_index(), 2);
}

#[test]
fn checkpoint_restore_reproduces_next_draws_in_canonical_key_order() {
    let service = RandomService::new(37);
    for (entity, draws) in [(8, 3), (2, 1), (5, 2)] {
        let mut lease = service
            .acquire_stream(keyed(entity))
            .expect("stream admission");
        for _ in 0..draws {
            lease.try_next_u32().expect("stream draw");
        }
    }
    let checkpoint = service.checkpoint().expect("idle registry checkpoint");
    let checkpoint_keys = checkpoint
        .streams()
        .iter()
        .map(|stream| stream.key())
        .collect::<Vec<_>>();
    assert!(checkpoint_keys.windows(2).all(|pair| pair[0] < pair[1]));

    let restored = RandomService::from_checkpoint(checkpoint)
        .expect("checkpoint within the default stream capacity");
    for entity in [2, 5, 8] {
        let mut original = service
            .acquire_stream(keyed(entity))
            .expect("original stream");
        let mut replay = restored
            .acquire_stream(keyed(entity))
            .expect("restored stream");
        assert_eq!(original.try_next_u32(), replay.try_next_u32());
        assert_eq!(original.draw_index(), replay.draw_index());
    }
}

#[test]
fn active_leases_block_checkpoint_reseed_and_target_eviction_without_mutation() {
    let mut service = RandomService::new(41);
    let mut lease = service.acquire_stream(key()).expect("stream admission");
    lease.try_next_u32().expect("stream draw");
    let authority_before = service.snapshot();

    assert_eq!(
        service.checkpoint(),
        Err(RandomServiceError::CheckpointBlocked { active_leases: 1 })
    );
    assert_eq!(
        service.reseed(99),
        Err(RandomServiceError::ReseedBlocked { active_leases: 1 })
    );
    assert_eq!(
        service.evict_stream(key()),
        Err(RandomServiceError::StreamEvictionBlocked { key: key() })
    );
    assert_eq!(service.snapshot(), authority_before);

    lease.release();
    let progressed = service
        .evict_stream(key())
        .expect("idle stream can be evicted")
        .expect("registered stream existed");
    assert_eq!(progressed.draw_index(), 1);
    assert_eq!(service.registered_stream_count(), 0);

    service.reseed(99).expect("idle registry can reseed");
    assert_eq!(service.master_seed_generation(), 1);
    assert_eq!(service.registered_stream_count(), 0);
}

#[test]
fn concurrent_same_key_admission_has_exactly_one_winner() {
    let service = Arc::new(RandomService::new(43));
    let start = Arc::new(Barrier::new(3));
    let finish = Arc::new(Barrier::new(3));
    let workers = (0..2)
        .map(|_| {
            let service = Arc::clone(&service);
            let start = Arc::clone(&start);
            let finish = Arc::clone(&finish);
            thread::spawn(move || {
                start.wait();
                let lease = service.acquire_stream(key());
                let admitted = lease.is_ok();
                finish.wait();
                drop(lease);
                admitted
            })
        })
        .collect::<Vec<_>>();
    start.wait();
    finish.wait();

    let winners = workers
        .into_iter()
        .map(|worker| worker.join().expect("worker should not panic"))
        .filter(|admitted| *admitted)
        .count();
    assert_eq!(winners, 1);
}

#[test]
fn large_registry_keeps_one_canonical_entry_per_stable_key() {
    const STREAM_COUNT: u64 = 4_096;
    let service = RandomService::new(47);
    for entity in (0..STREAM_COUNT).rev() {
        service
            .acquire_stream(keyed(entity))
            .expect("unique key admission")
            .release();
    }
    let checkpoint = service.checkpoint().expect("idle registry checkpoint");

    assert_eq!(service.registered_stream_count(), STREAM_COUNT as usize);
    assert_eq!(checkpoint.streams().len(), STREAM_COUNT as usize);
    assert!(checkpoint
        .streams()
        .windows(2)
        .all(|pair| pair[0].key() < pair[1].key()));
}
