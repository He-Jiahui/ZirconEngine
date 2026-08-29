use std::sync::Arc;

use zr_contracts::random::{
    RandomAlgorithmId, RandomEntityKey, RandomSeedReceipt, RandomServiceCheckpoint,
    RandomServiceState, RandomState, RandomStreamCheckpoint, RandomStreamKey, RandomWorldKey,
};

use super::authority::RandomAuthority;
use super::derivation::derive_stream;
use super::{RandomServiceError, RandomServiceLimits, RandomStreamLease};

const DEFAULT_MASTER_SEED: u64 = 0;

/// Engine-owned seed authority and unique deterministic-stream registry.
#[derive(Debug)]
pub struct RandomService {
    authority: Arc<RandomAuthority>,
}

impl Default for RandomService {
    fn default() -> Self {
        Self::new(DEFAULT_MASTER_SEED)
    }
}

impl RandomService {
    pub fn new(master_seed: u64) -> Self {
        Self::with_limits(master_seed, RandomServiceLimits::default())
    }

    pub fn with_limits(master_seed: u64, limits: RandomServiceLimits) -> Self {
        Self {
            authority: Arc::new(RandomAuthority::new(master_seed, limits)),
        }
    }

    pub fn algorithm(&self) -> RandomAlgorithmId {
        self.authority.algorithm()
    }

    pub fn master_seed(&self) -> u64 {
        self.authority.master_seed()
    }

    pub fn master_seed_generation(&self) -> u64 {
        self.authority.master_seed_generation()
    }

    /// Captures only the authority required to derive future unseen streams.
    pub fn snapshot(&self) -> RandomServiceState {
        self.authority.snapshot()
    }

    /// Restores seed authority with an empty stream registry.
    pub fn from_state(state: RandomServiceState) -> Self {
        Self::from_state_with_limits(state, RandomServiceLimits::default())
    }

    pub fn from_state_with_limits(state: RandomServiceState, limits: RandomServiceLimits) -> Self {
        Self {
            authority: Arc::new(RandomAuthority::from_state(state, limits)),
        }
    }

    /// Restores seed authority and every parked stream from a validated checkpoint.
    pub fn from_checkpoint(
        checkpoint: RandomServiceCheckpoint,
    ) -> Result<Self, RandomServiceError> {
        Self::from_checkpoint_with_limits(checkpoint, RandomServiceLimits::default())
    }

    pub fn from_checkpoint_with_limits(
        checkpoint: RandomServiceCheckpoint,
        limits: RandomServiceLimits,
    ) -> Result<Self, RandomServiceError> {
        let (state, streams) = checkpoint.into_parts();
        Ok(Self {
            authority: Arc::new(RandomAuthority::from_checkpoint(state, limits, streams)?),
        })
    }

    /// Acquires the sole mutable lease for a stable stream owner.
    pub fn acquire_stream(
        &self,
        key: RandomStreamKey,
    ) -> Result<RandomStreamLease, RandomServiceError> {
        let stream = self.authority.registry().acquire(key, || {
            let authority = self.authority.snapshot();
            derive_stream(
                authority.algorithm(),
                authority.master_seed(),
                authority.master_seed_generation(),
                key,
            )
        })?;
        Ok(RandomStreamLease::new(
            key,
            stream,
            Arc::clone(&self.authority),
        ))
    }

    /// Captures seed authority and all registered stream progress in canonical key order.
    pub fn checkpoint(&self) -> Result<RandomServiceCheckpoint, RandomServiceError> {
        self.checkpoint_with_stream_capture_hook(|| {})
    }

    fn checkpoint_with_stream_capture_hook(
        &self,
        after_stream_capture: impl FnOnce(),
    ) -> Result<RandomServiceCheckpoint, RandomServiceError> {
        let (authority, streams) = self
            .authority
            .registry()
            .checkpoint_with_authority_snapshot(|| {
                after_stream_capture();
                self.authority.snapshot()
            })
            .map_err(|active_leases| RandomServiceError::CheckpointBlocked { active_leases })?;
        Ok(RandomServiceCheckpoint::try_new(authority, streams)?)
    }

    /// Replaces the master seed after proving that no mutable stream is outstanding.
    pub fn reseed(&mut self, master_seed: u64) -> Result<RandomSeedReceipt, RandomServiceError> {
        self.authority.reseed(master_seed)
    }

    /// Explicitly removes parked progress so the next acquire re-derives the stream.
    pub fn evict_stream(
        &self,
        key: RandomStreamKey,
    ) -> Result<Option<RandomState>, RandomServiceError> {
        self.authority
            .registry()
            .evict(key)
            .map_err(|()| RandomServiceError::StreamEvictionBlocked { key })
    }

    /// Removes every parked stream owned by one exact World generation.
    pub fn evict_world(
        &self,
        world: RandomWorldKey,
    ) -> Result<Vec<RandomStreamCheckpoint>, RandomServiceError> {
        self.authority
            .registry()
            .evict_matching(|key| key.world() == world)
            .map_err(
                |active_leases| RandomServiceError::StreamScopeEvictionBlocked { active_leases },
            )
    }

    /// Removes every parked stream owned by one exact entity generation.
    pub fn evict_entity(
        &self,
        world: RandomWorldKey,
        entity: RandomEntityKey,
    ) -> Result<Vec<RandomStreamCheckpoint>, RandomServiceError> {
        self.authority
            .registry()
            .evict_matching(|key| key.world() == world && key.entity() == Some(entity))
            .map_err(
                |active_leases| RandomServiceError::StreamScopeEvictionBlocked { active_leases },
            )
    }

    pub fn registered_stream_count(&self) -> usize {
        self.authority.registry().registered_stream_count()
    }

    pub fn active_lease_count(&self) -> usize {
        self.authority.registry().active_lease_count()
    }
}

#[cfg(test)]
mod ownership_tests {
    use std::sync::{mpsc, Arc};
    use std::thread;

    use zr_contracts::random::{
        RandomPurposeKey, RandomStreamKey, RandomSystemKey, RandomWorldKey,
    };

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
        let (reseed_started_tx, reseed_started_rx) = mpsc::sync_channel(0);
        let reseed_worker = thread::spawn(move || {
            reseed_started_tx.send(()).expect("reseed start observer");
            reseed_authority.reseed(RESEEDED_SEED)
        });
        reseed_started_rx.recv().expect("reseed start signal");

        resume_tx.send(()).expect("checkpoint resume receiver");
        let checkpoint = checkpoint_worker
            .join()
            .expect("checkpoint worker should not panic")
            .expect("checkpoint should succeed");
        reseed_worker
            .join()
            .expect("reseed worker should not panic")
            .expect("idle service should reseed");

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
}
