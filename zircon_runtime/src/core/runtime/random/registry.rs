use std::collections::{BTreeMap, btree_map::Entry};
use std::sync::{Mutex, MutexGuard};

use zr_contracts::random::{
    RandomServiceState, RandomState, RandomStreamCheckpoint, RandomStreamKey,
};

use super::{RandomServiceError, RandomServiceLimits, RandomStream};

#[derive(Debug)]
enum RandomStreamEntry {
    Available(RandomStream),
    Leased,
}

#[derive(Debug, Default)]
struct RandomStreamRegistryState {
    streams: BTreeMap<RandomStreamKey, RandomStreamEntry>,
    active_leases: usize,
}

/// Canonical owner of parked stream state and stable-key admission.
#[derive(Debug, Default)]
pub(crate) struct RandomStreamRegistry {
    limits: RandomServiceLimits,
    state: Mutex<RandomStreamRegistryState>,
}

impl RandomStreamRegistry {
    pub(crate) fn new(limits: RandomServiceLimits) -> Self {
        Self {
            limits,
            state: Mutex::new(RandomStreamRegistryState::default()),
        }
    }

    pub(crate) fn from_checkpoints(
        limits: RandomServiceLimits,
        streams: Vec<RandomStreamCheckpoint>,
    ) -> Result<Self, RandomServiceError> {
        if streams.len() > limits.max_registered_streams() {
            return Err(RandomServiceError::StreamCapacityExceeded {
                capacity: limits.max_registered_streams(),
            });
        }
        let streams = streams
            .into_iter()
            .map(|checkpoint| {
                (
                    checkpoint.key(),
                    RandomStreamEntry::Available(RandomStream::from_valid_state(
                        checkpoint.state(),
                    )),
                )
            })
            .collect();
        Ok(Self {
            limits,
            state: Mutex::new(RandomStreamRegistryState {
                streams,
                active_leases: 0,
            }),
        })
    }

    pub(crate) fn acquire<F>(
        &self,
        key: RandomStreamKey,
        derive: F,
    ) -> Result<RandomStream, RandomServiceError>
    where
        F: FnOnce() -> RandomStream,
    {
        let mut state = self.lock();
        let at_capacity = state.streams.len() >= self.limits.max_registered_streams();
        let stream = match state.streams.entry(key) {
            Entry::Vacant(slot) => {
                if at_capacity {
                    return Err(RandomServiceError::StreamCapacityExceeded {
                        capacity: self.limits.max_registered_streams(),
                    });
                }
                let stream = derive();
                slot.insert(RandomStreamEntry::Leased);
                stream
            }
            Entry::Occupied(mut slot) => {
                match std::mem::replace(slot.get_mut(), RandomStreamEntry::Leased) {
                    RandomStreamEntry::Available(stream) => stream,
                    RandomStreamEntry::Leased => {
                        return Err(RandomServiceError::StreamAlreadyAcquired { key });
                    }
                }
            }
        };
        state.active_leases += 1;
        Ok(stream)
    }

    pub(crate) fn checkpoint_with_authority_snapshot(
        &self,
        capture_authority: impl FnOnce() -> RandomServiceState,
        after_stream_capture: impl FnOnce(),
    ) -> Result<(RandomServiceState, Vec<RandomStreamCheckpoint>), usize> {
        let state = self.lock();
        let active_leases = state.active_leases;
        if active_leases > 0 {
            return Err(active_leases);
        }
        // Keep the registry guard while entering the seed authority: reseed uses the same order.
        let authority = capture_authority();
        let master_seed_generation = authority.master_seed_generation();
        let streams = state
            .streams
            .iter()
            .filter_map(|(key, entry)| match entry {
                RandomStreamEntry::Available(stream) => Some(RandomStreamCheckpoint::new(
                    *key,
                    stream.snapshot(),
                    master_seed_generation,
                )),
                RandomStreamEntry::Leased => None,
            })
            .collect();
        after_stream_capture();
        Ok((authority, streams))
    }

    pub(crate) fn clear_if_idle_with<T, E>(
        &self,
        commit: impl FnOnce() -> Result<T, E>,
    ) -> Result<Result<T, E>, usize> {
        let mut state = self.lock();
        let active_leases = state.active_leases;
        if active_leases > 0 {
            return Err(active_leases);
        }
        let result = commit();
        if result.is_ok() {
            state.streams.clear();
        }
        Ok(result)
    }

    pub(crate) fn evict(&self, key: RandomStreamKey) -> Result<Option<RandomState>, ()> {
        let mut state = self.lock();
        match state.streams.get(&key) {
            Some(RandomStreamEntry::Leased) => Err(()),
            Some(RandomStreamEntry::Available(_)) => {
                let entry = state.streams.remove(&key);
                match entry {
                    Some(RandomStreamEntry::Available(stream)) => Ok(Some(stream.snapshot())),
                    _ => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    pub(crate) fn evict_matching(
        &self,
        matches: impl Fn(RandomStreamKey) -> bool,
        capture_master_seed_generation: impl FnOnce() -> u64,
    ) -> Result<Vec<RandomStreamCheckpoint>, usize> {
        let mut state = self.lock();
        let mut matching_streams = 0usize;
        let mut active_leases = 0usize;
        for (key, entry) in &state.streams {
            if matches(*key) {
                matching_streams = matching_streams.saturating_add(1);
                active_leases += usize::from(matches!(entry, RandomStreamEntry::Leased));
            }
        }
        if active_leases > 0 {
            return Err(active_leases);
        }
        if matching_streams == 0 {
            return Ok(Vec::new());
        }

        let master_seed_generation = capture_master_seed_generation();
        let mut checkpoints = Vec::with_capacity(matching_streams);
        state.streams.retain(|key, entry| {
            if !matches(*key) {
                return true;
            }
            let RandomStreamEntry::Available(stream) = entry else {
                unreachable!("matching active leases were rejected before scope eviction")
            };
            checkpoints.push(RandomStreamCheckpoint::new(
                *key,
                stream.snapshot(),
                master_seed_generation,
            ));
            false
        });
        Ok(checkpoints)
    }

    pub(crate) fn registered_stream_count(&self) -> usize {
        self.lock().streams.len()
    }

    pub(crate) fn active_lease_count(&self) -> usize {
        self.lock().active_leases
    }

    #[cfg(test)]
    pub(crate) fn lock_is_held_for_test(&self) -> bool {
        matches!(
            self.state.try_lock(),
            Err(std::sync::TryLockError::WouldBlock)
        )
    }

    pub(crate) fn release(&self, key: RandomStreamKey, stream: RandomStream) {
        let mut state = self.lock();
        let entry = state
            .streams
            .get_mut(&key)
            .expect("leased random stream key must remain registered");
        assert!(
            matches!(entry, RandomStreamEntry::Leased),
            "random stream release must replace a leased entry"
        );
        *entry = RandomStreamEntry::Available(stream);
        state.active_leases = state
            .active_leases
            .checked_sub(1)
            .expect("random stream active-lease count must not underflow");
    }

    fn lock(&self) -> MutexGuard<'_, RandomStreamRegistryState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
#[path = "registry/evict_matching_tests.rs"]
mod evict_matching_tests;
