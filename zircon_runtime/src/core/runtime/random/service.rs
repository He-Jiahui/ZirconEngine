use std::sync::Arc;

use zr_contracts::random::{
    RandomAlgorithmId, RandomEntityKey, RandomSeedReceipt, RandomServiceCheckpoint,
    RandomServiceState, RandomState, RandomStreamCheckpoint, RandomStreamKey, RandomWorldKey,
};

use super::authority::RandomAuthority;
use super::derivation::derive_stream;
use super::{RandomServiceError, RandomServiceLimits, RandomStreamLease};

const DEFAULT_MASTER_SEED: u64 = 0;

/// 一个 Runtime 的主种子权威与流租约入口；每个稳定键同一时刻只授予一个可变租约。
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

    /// 仅当没有活动流租约时才捕获种子代与全部进度，保证检查点可一致恢复。
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
            .checkpoint_with_authority_snapshot(|| self.authority.snapshot(), after_stream_capture)
            .map_err(|active_leases| RandomServiceError::CheckpointBlocked { active_leases })?;
        Ok(RandomServiceCheckpoint::try_new(authority, streams)?)
    }

    /// Replaces the master seed after proving that no mutable stream is outstanding.
    pub fn reseed(&mut self, master_seed: u64) -> Result<RandomSeedReceipt, RandomServiceError> {
        self.authority.reseed(master_seed)
    }

    /// Explicitly removes parked progress so the next acquire re-derives the stream.
    ///
    /// The returned state is an observation of the removed progress, not an independently
    /// restorable checkpoint. Use [`Self::checkpoint`] for replay persistence.
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
            .evict_matching(
                |key| key.world() == world,
                || self.authority.master_seed_generation(),
            )
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
            .evict_matching(
                |key| key.world() == world && key.entity() == Some(entity),
                || self.authority.master_seed_generation(),
            )
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
#[path = "tests/service_ownership_tests.rs"]
mod ownership_tests;
