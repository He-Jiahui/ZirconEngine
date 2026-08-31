use std::collections::{BTreeMap, HashMap};
use std::mem;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use zircon_runtime::asset::{AssetUri, AssetUuid};

use crate::core::jobs::MutexGroup;

use super::flight::ImportFlight;
use super::{
    EditorAssetImportAdmissionLimits, EditorAssetImportReason, EditorAssetImportSubmitError,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ImportGenerationKey {
    uuid: AssetUuid,
    uri: Arc<AssetUri>,
    source_digest: Arc<str>,
}

impl ImportGenerationKey {
    pub(super) fn new(uuid: AssetUuid, uri: Arc<AssetUri>, source_digest: Arc<str>) -> Self {
        Self {
            uuid,
            uri,
            source_digest,
        }
    }

    fn estimated_bytes(&self) -> usize {
        mem::size_of::<Self>()
            .saturating_add(self.uri.path().len())
            .saturating_add(self.uri.label().map(str::len).unwrap_or_default())
            .saturating_add(self.source_digest.len())
    }

    fn uri(&self) -> &AssetUri {
        self.uri.as_ref()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FlightIdentity(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UuidLifecycleToken {
    uuid: AssetUuid,
    identity: u64,
}

impl UuidLifecycleToken {
    pub(super) fn uuid(self) -> AssetUuid {
        self.uuid
    }
}

pub(super) enum ImportReservation {
    Existing {
        flight: Arc<ImportFlight>,
    },
    New {
        key: ImportGenerationKey,
        flight_identity: FlightIdentity,
        mutex_group: MutexGroup,
        begin_uuid: Option<UuidLifecycleToken>,
        flight: Arc<ImportFlight>,
    },
}

pub(super) enum ReserveAttempt {
    Ready(ImportReservation),
    UuidTransitionPending,
}

struct FlightEntry {
    identity: FlightIdentity,
    flight: Arc<ImportFlight>,
    created_at: Instant,
    estimated_bytes: usize,
    active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UuidImportPhase {
    Starting,
    Ready,
    Clearing,
}

struct ActiveUuidImports {
    token: UuidLifecycleToken,
    mutex_group: MutexGroup,
    active_count: usize,
    phase: UuidImportPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ImportFinishAction {
    NoIndexTransition,
    ClearUuid(UuidLifecycleToken),
}

#[derive(Default)]
pub(super) struct ImportFlowState {
    flights: HashMap<ImportGenerationKey, FlightEntry>,
    active_by_uuid: HashMap<AssetUuid, ActiveUuidImports>,
    active_order: BTreeMap<Instant, Vec<ImportGenerationKey>>,
    completed_order: BTreeMap<Instant, Vec<ImportGenerationKey>>,
    admission_bytes: usize,
    next_mutex_group: u64,
    next_flight_identity: u64,
    next_uuid_lifecycle: u64,
}

impl ImportFlowState {
    pub(super) fn reserve(
        &mut self,
        key: ImportGenerationKey,
        reason: EditorAssetImportReason,
        now: Instant,
        limits: EditorAssetImportAdmissionLimits,
    ) -> Result<ReserveAttempt, EditorAssetImportSubmitError> {
        self.evict_expired_completed(now, limits.max_oldest_age);

        if let Some(entry) = self.flights.get(&key) {
            return Ok(ReserveAttempt::Ready(ImportReservation::Existing {
                flight: Arc::clone(&entry.flight),
            }));
        }

        if self
            .active_by_uuid
            .get(&key.uuid)
            .is_some_and(|active| active.phase != UuidImportPhase::Ready)
        {
            return Ok(ReserveAttempt::UuidTransitionPending);
        }

        let estimated_bytes = key
            .estimated_bytes()
            .saturating_add(mem::size_of::<FlightEntry>())
            .saturating_add(mem::size_of::<ImportFlight>())
            .saturating_add(mem::size_of::<EditorAssetImportReason>());
        self.evict_completed_until_within_limits(estimated_bytes, limits);

        if let Some((created_at, _)) = self.active_order.first_key_value() {
            if now.saturating_duration_since(*created_at) >= limits.max_oldest_age {
                return Err(EditorAssetImportSubmitError::OldestFlightAgeExceeded {
                    max_age_ms: duration_millis(limits.max_oldest_age),
                });
            }
        }
        if self.flights.len() >= limits.max_flights {
            return Err(EditorAssetImportSubmitError::FlightLimitReached {
                limit: limits.max_flights,
            });
        }
        if self.admission_bytes.saturating_add(estimated_bytes) > limits.max_estimated_bytes {
            return Err(EditorAssetImportSubmitError::ByteLimitExceeded {
                limit: limits.max_estimated_bytes,
                current: self.admission_bytes,
                requested: estimated_bytes,
            });
        }

        let (mutex_group, begin_uuid) = match self.active_by_uuid.get_mut(&key.uuid) {
            Some(active) => {
                active.active_count = active.active_count.saturating_add(1);
                (active.mutex_group.clone(), None)
            }
            None => {
                let mutex_group = self.allocate_mutex_group()?;
                let token = self.allocate_uuid_lifecycle(key.uuid);
                self.active_by_uuid.insert(
                    key.uuid,
                    ActiveUuidImports {
                        token,
                        mutex_group: mutex_group.clone(),
                        active_count: 1,
                        phase: UuidImportPhase::Starting,
                    },
                );
                (mutex_group, Some(token))
            }
        };
        let flight = Arc::new(ImportFlight::new(Arc::clone(&key.uri), reason));
        let flight_identity = self.allocate_flight_identity();
        self.admission_bytes = self.admission_bytes.saturating_add(estimated_bytes);
        self.active_order.entry(now).or_default().push(key.clone());
        self.flights.insert(
            key.clone(),
            FlightEntry {
                identity: flight_identity,
                flight: Arc::clone(&flight),
                created_at: now,
                estimated_bytes,
                active: true,
            },
        );
        Ok(ReserveAttempt::Ready(ImportReservation::New {
            key,
            flight_identity,
            mutex_group,
            begin_uuid,
            flight,
        }))
    }

    pub(super) fn mark_uuid_ready(&mut self, token: UuidLifecycleToken) -> bool {
        let Some(active) = self.active_by_uuid.get_mut(&token.uuid) else {
            return false;
        };
        if active.token != token || active.phase != UuidImportPhase::Starting {
            return false;
        }
        active.phase = UuidImportPhase::Ready;
        true
    }

    pub(super) fn abort_unsubmitted(
        &mut self,
        key: &ImportGenerationKey,
        identity: FlightIdentity,
    ) -> ImportFinishAction {
        self.finish_entry(
            key,
            identity,
            false,
            0,
            EditorAssetImportAdmissionLimits::new(usize::MAX, usize::MAX, Duration::MAX),
            Instant::now(),
        )
    }

    pub(super) fn finish(
        &mut self,
        key: &ImportGenerationKey,
        identity: FlightIdentity,
        successful: bool,
        completed_result_bytes: usize,
        limits: EditorAssetImportAdmissionLimits,
        now: Instant,
    ) -> ImportFinishAction {
        self.finish_entry(
            key,
            identity,
            successful,
            completed_result_bytes,
            limits,
            now,
        )
    }

    fn finish_entry(
        &mut self,
        key: &ImportGenerationKey,
        identity: FlightIdentity,
        retain_completed: bool,
        completed_result_bytes: usize,
        limits: EditorAssetImportAdmissionLimits,
        now: Instant,
    ) -> ImportFinishAction {
        let Some(entry) = self.flights.get_mut(key) else {
            return ImportFinishAction::NoIndexTransition;
        };
        if entry.identity != identity || !entry.active {
            return ImportFinishAction::NoIndexTransition;
        }
        entry.active = false;
        let created_at = entry.created_at;
        let base_bytes = entry.estimated_bytes;
        remove_order_entry(&mut self.active_order, created_at, key);

        let mut remove_uuid = false;
        let action = self
            .active_by_uuid
            .get_mut(&key.uuid)
            .map(|active| {
                active.active_count = active.active_count.saturating_sub(1);
                if active.active_count != 0 {
                    return ImportFinishAction::NoIndexTransition;
                }
                if active.phase == UuidImportPhase::Starting {
                    remove_uuid = true;
                    ImportFinishAction::NoIndexTransition
                } else {
                    active.phase = UuidImportPhase::Clearing;
                    ImportFinishAction::ClearUuid(active.token)
                }
            })
            .unwrap_or(ImportFinishAction::NoIndexTransition);
        if remove_uuid {
            self.active_by_uuid.remove(&key.uuid);
        }

        let retain_completed =
            retain_completed && self.make_room_for_completed_result(completed_result_bytes, limits);
        if retain_completed {
            if let Some(entry) = self.flights.get_mut(key) {
                entry.estimated_bytes =
                    entry.estimated_bytes.saturating_add(completed_result_bytes);
            }
            self.admission_bytes = self.admission_bytes.saturating_add(completed_result_bytes);
            self.completed_order
                .entry(now)
                .or_default()
                .push(key.clone());
        } else {
            self.flights.remove(key);
            self.admission_bytes = self.admission_bytes.saturating_sub(base_bytes);
        }
        action
    }

    pub(super) fn complete_uuid_clear(&mut self, token: UuidLifecycleToken) -> bool {
        let should_remove = self.active_by_uuid.get(&token.uuid).is_some_and(|active| {
            active.token == token
                && active.phase == UuidImportPhase::Clearing
                && active.active_count == 0
        });
        if should_remove {
            self.active_by_uuid.remove(&token.uuid);
        }
        should_remove
    }

    fn make_room_for_completed_result(
        &mut self,
        completed_result_bytes: usize,
        limits: EditorAssetImportAdmissionLimits,
    ) -> bool {
        while self.admission_bytes.saturating_add(completed_result_bytes)
            > limits.max_estimated_bytes
        {
            let Some((completed_at, key)) = oldest_entry(&self.completed_order) else {
                break;
            };
            self.remove_completed(completed_at, &key);
        }
        self.admission_bytes.saturating_add(completed_result_bytes) <= limits.max_estimated_bytes
    }

    fn evict_expired_completed(&mut self, now: Instant, max_oldest_age: Duration) {
        loop {
            let Some((completed_at, key)) = oldest_entry(&self.completed_order) else {
                break;
            };
            if now.saturating_duration_since(completed_at) < max_oldest_age {
                break;
            }
            self.remove_completed(completed_at, &key);
        }
    }

    fn evict_completed_until_within_limits(
        &mut self,
        requested_bytes: usize,
        limits: EditorAssetImportAdmissionLimits,
    ) {
        while self.flights.len() >= limits.max_flights
            || self.admission_bytes.saturating_add(requested_bytes) > limits.max_estimated_bytes
        {
            let Some((completed_at, key)) = oldest_entry(&self.completed_order) else {
                break;
            };
            self.remove_completed(completed_at, &key);
        }
    }

    fn remove_completed(&mut self, completed_at: Instant, key: &ImportGenerationKey) {
        remove_order_entry(&mut self.completed_order, completed_at, key);
        if let Some(entry) = self.flights.remove(key) {
            self.admission_bytes = self.admission_bytes.saturating_sub(entry.estimated_bytes);
        }
    }

    fn allocate_mutex_group(&mut self) -> Result<MutexGroup, EditorAssetImportSubmitError> {
        loop {
            let value = self.next_mutex_group;
            self.next_mutex_group = self.next_mutex_group.wrapping_add(1);
            // Keep a future format change inside the submit error path instead of crashing the
            // editor while starting an asset import.
            let candidate = MutexGroup::parse(format!("asset_import_{value:016x}"))?;
            if self
                .active_by_uuid
                .values()
                .all(|active| active.mutex_group != candidate)
            {
                return Ok(candidate);
            }
        }
    }

    fn allocate_flight_identity(&mut self) -> FlightIdentity {
        let identity = FlightIdentity(self.next_flight_identity);
        self.next_flight_identity = self.next_flight_identity.wrapping_add(1);
        identity
    }

    fn allocate_uuid_lifecycle(&mut self, uuid: AssetUuid) -> UuidLifecycleToken {
        let token = UuidLifecycleToken {
            uuid,
            identity: self.next_uuid_lifecycle,
        };
        self.next_uuid_lifecycle = self.next_uuid_lifecycle.wrapping_add(1);
        token
    }
}

#[derive(Default)]
pub(super) struct ImportFlowSharedState {
    state: Mutex<ImportFlowState>,
}

impl ImportFlowSharedState {
    pub(super) fn reserve(
        &self,
        key: ImportGenerationKey,
        reason: EditorAssetImportReason,
        limits: EditorAssetImportAdmissionLimits,
    ) -> Result<ImportReservation, EditorAssetImportSubmitError> {
        let uri = key.uri().clone();
        match self.lock().reserve(key, reason, Instant::now(), limits)? {
            ReserveAttempt::Ready(reservation) => Ok(reservation),
            ReserveAttempt::UuidTransitionPending => {
                Err(EditorAssetImportSubmitError::UuidLifecycleTransitionPending { uri })
            }
        }
    }

    pub(super) fn mark_uuid_ready(&self, token: UuidLifecycleToken) -> bool {
        self.lock().mark_uuid_ready(token)
    }

    pub(super) fn abort_unsubmitted(
        &self,
        key: &ImportGenerationKey,
        identity: FlightIdentity,
    ) -> ImportFinishAction {
        self.lock().abort_unsubmitted(key, identity)
    }

    pub(super) fn finish(
        &self,
        key: &ImportGenerationKey,
        identity: FlightIdentity,
        successful: bool,
        completed_result_bytes: usize,
        limits: EditorAssetImportAdmissionLimits,
    ) -> ImportFinishAction {
        self.lock().finish(
            key,
            identity,
            successful,
            completed_result_bytes,
            limits,
            Instant::now(),
        )
    }

    pub(super) fn complete_uuid_clear(&self, token: UuidLifecycleToken) -> bool {
        self.lock().complete_uuid_clear(token)
    }

    fn lock(&self) -> MutexGuard<'_, ImportFlowState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn duration_millis(duration: Duration) -> u64 {
    duration.as_millis().min(u64::MAX as u128) as u64
}

fn oldest_entry(
    order: &BTreeMap<Instant, Vec<ImportGenerationKey>>,
) -> Option<(Instant, ImportGenerationKey)> {
    order
        .first_key_value()
        .and_then(|(created_at, keys)| keys.first().map(|key| (*created_at, key.clone())))
}

fn remove_order_entry(
    order: &mut BTreeMap<Instant, Vec<ImportGenerationKey>>,
    created_at: Instant,
    key: &ImportGenerationKey,
) {
    let remove_bucket = order.get_mut(&created_at).is_some_and(|keys| {
        if let Some(position) = keys.iter().position(|candidate| candidate == key) {
            keys.swap_remove(position);
        }
        keys.is_empty()
    });
    if remove_bucket {
        order.remove(&created_at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_mutex_groups_are_distinct_valid_submission_values() {
        let mut state = ImportFlowState::default();
        let first = state.allocate_mutex_group().unwrap();
        let second = state.allocate_mutex_group().unwrap();

        assert_eq!(first.as_str(), "asset_import_0000000000000000");
        assert_eq!(second.as_str(), "asset_import_0000000000000001");
        assert_ne!(first, second);
    }
}
